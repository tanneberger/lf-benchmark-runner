/*
 * @author Tassilo Tanneberger <tassilo.tanneberger@tu-dresden.de>
 * @description Data extractor for Lingua-Franca CPP Benchmarks
*/

mod structs;

use clap::Parser;
use regex::Regex;

use std::fs::{File, OpenOptions, read_to_string, metadata};
use std::process::Command;
use serde_json::{Value, from_str, to_writer};

use structs::{Args, ResultBuilder};


fn perform_benchmark(
    binary: &String,
    runtime_version: &String,
    number_of_runs: u32,
    entry_count: u32,
    threads: u8,
) -> Option<[String; 20]> {
    let mut best_time: f32 = 0.0;
    let mut worst_time: f32 = 0.0;
    let mut median: f32 = 0.0;
    let mut iterations: u32 = 0;
    let mut benchmark: String = String::new();

    for _i in 0u32..number_of_runs {
        let command_run = Command::new(binary).output();

        match command_run {
            Err(_) => {
                println!("Execution of Command failed !");
                return None;
            }
            Ok(result) => {
                if result.status.success() {
                    let output = String::from_utf8_lossy(&result.stdout);
                    let re_best_time = Regex::new(r"Best Time:   (\d{3}).(\d{3})").unwrap();
                    let re_worst_time = Regex::new(r"Worst Time:   (\d{3}).(\d{3})").unwrap();
                    let re_median = Regex::new(r"Median:   (\d{3}).(\d{3})").unwrap();
                    let re_iterations = Regex::new(r"numIterations = (\d+)").unwrap();
                    let re_benchmarks = Regex::new(r"Benchmark: (.+)").unwrap();

                    for cap in re_best_time.captures_iter(&output) {
                        best_time += &cap[1].parse::<f32>().unwrap()
                            + 0.001 * &cap[2].parse::<f32>().unwrap();
                    }
                    for cap in re_worst_time.captures_iter(&output) {
                        worst_time += &cap[1].parse::<f32>().unwrap()
                            + 0.001 * &cap[2].parse::<f32>().unwrap();
                    }
                    for cap in re_median.captures_iter(&output) {
                        median += &cap[1].parse::<f32>().unwrap()
                            + 0.001 * &cap[2].parse::<f32>().unwrap();
                    }
                    for cap in re_iterations.captures_iter(&output) {
                        iterations += (&cap[1]).parse::<u32>().unwrap();
                    }
                    for cap in re_benchmarks.captures_iter(&output) {
                        benchmark = String::from(&cap[1]);
                    }
                    println!(
                        "Extracted Information: B:{} W:{} M:{} I:{} B: {}",
                        best_time, worst_time, median, iterations, benchmark
                    );
                }
            }
        }
    }
    Some(
        ResultBuilder::default()
            .count(entry_count)
            .benchmark_name(benchmark)
            .target(String::from("LF-Cpp"))
            .runtime_version(runtime_version.clone())
            .total_iterations(iterations)
            .threads(threads)
            .min_time_ms(best_time / (number_of_runs as f32))
            .max_time_ms(worst_time / (number_of_runs as f32))
            .mean_time_ms(median / (number_of_runs as f32))
            .median_time_ms(median / (number_of_runs as f32))
            .build()
            .unwrap()
            .serialize(),
    )
}

fn main() {
    let args = Args::parse();
    let mut results: Vec<[String; 20]> = Vec::new();
    let entry_count: u32;

    // reads how many entries are already inside the csv file
    if std::path::Path::new(&args.file).exists() {
        let raw_content = std::fs::read(&args.file).unwrap();
        let buffer = String::from_utf8_lossy(&raw_content);
        entry_count = (buffer.split("\n").count() - 1) as u32;
    } else {
        entry_count = 0;
    }

    // check if the given path is a directoty or a file
    let md = metadata(&args.binary);
    match md {
        Err(_) => {
            println!("The given path to the binary doesn't exists !");
            return
        }
        Ok(metadata) => {
            if metadata.is_dir() {
                let paths = std::fs::read_dir(&args.binary).unwrap();
                let mut i = entry_count;
                for path in paths {
                    let binary = path.unwrap().path().display().to_string();
                    println!("Name: {}", &binary);
                    results.push(
                        perform_benchmark(
                            &binary,
                            &args.runtime_version,
                            args.number_of_runs,
                            i,
                            args.threads,
                        )
                        .unwrap(),
                    );
                    i += 1;
                }
            } else {
                results.push(
                    perform_benchmark(
                        &args.binary,
                        &args.runtime_version,
                        args.number_of_runs,
                        entry_count,
                        args.threads,
                    )
                    .unwrap(),
                );
            }
        }
    } 
    

    let file: File;

    if !std::path::Path::new(&args.file).exists() {
        file = File::create(&args.file).unwrap();
    } else {
        file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&args.file)
            .unwrap();
    }
    
    if !args.json {
        if entry_count == 0 {
        {
            let mut wtr = csv::Writer::from_writer(&file);
            wtr.write_record(&[
                "",
                "benchmark",
                "target",
                "total_iterations",
                "threads",
                "pings",
                "runtime_version",
                "min_time_ms",
                "max_time_ms",
                "median_time_ms",
                "mean_time_ms",
                "pieces",
                "workers",
                "left",
                "right",
                "messages",
                "actors",
                "columns",
                "simulations",
                "channels",
            ])
            .unwrap();
        }
        }

        let mut wtr = csv::Writer::from_writer(&file);
        for record in results {
        wtr.write_record(&record).unwrap();
        }
        wtr.flush().unwrap();
    } else {
        let data = read_to_string(&args.file).expect("Unable to read file");
        let res: Value = from_str(&data).expect("Unable to parse");

        // res should be a list of json structs
    
        for record in results {
            res.push(record.to_json());
        }

        
        to_writer(&file, &res)?
    }
    
    println!("Finsihed benchmarks and saved data into {}", &args.file);
}
