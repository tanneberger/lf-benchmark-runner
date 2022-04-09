/*
 * @author Tassilo Tanneberger <tassilo.tanneberger@tu-dresden.de>
 * @description Data extractor for Lingua-Franca CPP Benchmarks
*/

extern crate derive_builder;

use clap::Parser;
use derive_builder::Builder;
use regex::Regex;

use std::fs::metadata;
use std::fs::{File, OpenOptions};
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(name = "Lingua-Franca benchmark runner")]
#[clap(author = "Tassilo Tanneberger <tassilo.tanneberger@tu-dresden.de>")]
#[clap(version = "1.0")]
#[clap(about = "Runns specified captures and extracts times.", long_about = None)]
struct Args {
    #[clap(short, long)]
    binary: String,

    #[clap(short, long)]
    target: String,

    #[clap(short, long)]
    file: String,

    #[clap(short, long, default_value_t = String::from(""))]
    name: String,

    #[clap(long, default_value_t = String::from("master"))]
    runtime_version: String,

    #[clap(long, default_value_t = 1)]
    number_of_runs: u32,

    #[clap(long, default_value_t = 1)]
    threads: u8,
}

fn format<T: std::fmt::Display>(option: &Option<T>) -> String {
    match option {
        Some(value) => {
            format!("{}", value)
        }
        None => {
            format!("")
        }
    }
}

#[derive(Default, Builder)]
struct Result {
    count: u32,
    benchmark_name: String,
    target: String,
    total_iterations: u32,
    threads: u8,

    #[builder(setter(into, strip_option), default)]
    pings: Option<u32>,

    runtime_version: String,
    min_time_ms: f32,
    max_time_ms: f32,
    median_time_ms: f32,
    mean_time_ms: f32,

    #[builder(setter(into), default)]
    pieces: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    workers: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    left: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    right: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    messages: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    actors: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    columns: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    simulations: Option<u32>,

    #[builder(setter(into, strip_option), default)]
    channels: Option<u32>,
}

impl Result {
    pub fn serialize(self: &Result) -> [String; 20] {
        return [
            self.count.to_string(),
            self.benchmark_name.clone(),
            self.target.clone(),
            self.total_iterations.to_string(),
            self.threads.to_string(),
            format(&self.pings),
            self.runtime_version.to_string(),
            self.min_time_ms.to_string(),
            self.max_time_ms.to_string(),
            self.median_time_ms.to_string(),
            self.mean_time_ms.to_string(),
            format(&self.pieces),
            format(&self.workers),
            format(&self.left),
            format(&self.right),
            format(&self.messages),
            format(&self.actors),
            format(&self.columns),
            format(&self.simulations),
            format(&self.channels),
        ];
    }
}

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
    let md = metadata(&args.binary).unwrap();
    if md.is_dir() {
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

    println!("Finsihed benchmarks and saved data into {}", &args.file);
}
