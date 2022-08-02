/*
 * @author Tassilo Tanneberger <tassilo.tanneberger@tu-dresden.de>
 * @description Data extractor for Lingua-Franca CPP Benchmarks
*/

mod structs;

use clap::Parser;
use regex::Regex;

use std::fs::{File, OpenOptions, metadata};
use std::process::Command;
use serde_json::{Value};
use serde::{Serialize, Deserialize};
use charts::{Chart, VerticalBarView, ScaleBand, ScaleLinear, BarLabelPosition};

use structs::{Args, ResultBuilder};

#[derive(Serialize, Deserialize)]
struct ResultCollector {
    values: Vec<Value>
}

fn perform_benchmark(
    binary: &String,
    target: &String,
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
                    let re_best_time = Regex::new(r"Best Time: (\s+)(\d+).(\d+)").unwrap();
                    let re_worst_time = Regex::new(r"Worst Time: (\s+)(\d+).(\d+)").unwrap();
                    let re_median = Regex::new(r"Median: (\s+)(\d+).(\d+)").unwrap();
                    let re_iterations = Regex::new(r"numIterations = (\d+)").unwrap();
                    let re_benchmarks = Regex::new(r"Benchmark: (.+)").unwrap();

                    for cap in re_best_time.captures_iter(&output) {
                        //println!("|{}|{}|{}|", &cap[0], &cap[2], &cap[3]);
                        let after_comma = cap[3].parse::<i32>().unwrap() as f32;
                        best_time += (cap[2].parse::<i32>().unwrap() as f32)+ (1f32/10f32.powf(after_comma.log10().ceil())) * after_comma
                    }
                    for cap in re_worst_time.captures_iter(&output) {
                        //println!("|{}|{}|{}|", &cap[0], &cap[2], &cap[3]);
                        let after_comma = cap[3].parse::<i32>().unwrap() as f32;
                        worst_time += (cap[2].parse::<i32>().unwrap() as f32) + (1f32/10f32.powf(after_comma.log10().ceil())) * after_comma
                    }
                    for cap in re_median.captures_iter(&output) {
                        //println!("|{}|{}|{}|", &cap[0], &cap[2], &cap[3]);
                        let after_comma = cap[3].parse::<i32>().unwrap() as f32;
                        median += (cap[2].parse::<i32>().unwrap() as f32) + (1f32/10f32.powf(after_comma.log10().ceil())) * after_comma
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
            .target(target.to_string())
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

fn draw_image(file: &String, data: &Vec<[String; 20]>) {
    // Define chart related sizes.
    let width = 800;
    let height = 600;
    let (top, right, bottom, left) = (90, 40, 50, 60);

    let mut extracted_data = Vec::new();
    let mut benchmarks = Vec::new();

    for result in data.into_iter() {
        benchmarks.push(result[1].clone());
        extracted_data.push((result[1].clone(), result[9].parse::<f32>().unwrap(), result[2].clone()))
    }

    let x = ScaleBand::new()
        .set_domain(benchmarks)
        .set_range(vec![0, width - left - right]);

    let y = ScaleLinear::new()
        .set_domain(vec![0.0, 2000.0])
        .set_range(vec![height - top - bottom, 0]);

    let view = VerticalBarView::new()
        .set_x_scale(&x)
        .set_y_scale(&y)
        // .set_label_visibility(false)  // <-- uncomment this line to hide bar value labels
        .set_label_position(BarLabelPosition::Center)
        .load_data(&extracted_data).unwrap();

    Chart::new()
        .set_width(width)
        .set_height(height)
        .set_margins(top, right, bottom, left)
        .add_title(String::from("Stacked Bar Chart"))
        .add_view(&view)
        .add_axis_bottom(&x)
        .add_axis_left(&y)
        .add_left_axis_label("Units of Measurement")
        .add_bottom_axis_label("Categories")
        .save(file).unwrap();
}

fn collect_data(args: &Args, entry_count: u32) -> Vec<[String; 20]>{
    let mut results = Vec::new();
    let binary = args.binary.as_ref().unwrap();
    // check if the given path is a directoty or a file
    let md = metadata(&binary);
    match md {
        Err(_) => {
            println!("The given path to the binary doesn't exists !");
            return results;
        }
        Ok(metadata) => {
            if metadata.is_dir() {
                let paths = std::fs::read_dir(&binary).unwrap();
                let mut i = entry_count;
                for path in paths {
                    let display_binary = path.unwrap().path().display().to_string();
                    println!("Name: {}", &display_binary);
                    results.push(
                        perform_benchmark(
                            &display_binary,
                            &args.target.as_ref().unwrap(),
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
                        &args.binary.as_ref().unwrap(),
                        &args.target.as_ref().unwrap(),
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
    return results;
}


fn main() {
    let args = Args::parse();
    let entry_count: u32;
    
    match args.file {
        Some(ref file_name) => {
            // reads how many entries are already inside the csv file
            if std::path::Path::new(&file_name).exists() {
                let raw_content = std::fs::read(&file_name).unwrap();
                let buffer = String::from_utf8_lossy(&raw_content);
                entry_count = (buffer.split("\n").count() - 1) as u32;
            } else {
                entry_count = 0;
            }
        }
        None => {
            entry_count = 0;
        }
    }

    let results = if args.binary.is_some() && args.target.is_some() {
        collect_data(&args, entry_count)
    } else {
        Vec::new()
    };

    match args.file {
        Some(ref file_name) => {
            let file: File;
            if !std::path::Path::new(&file_name).exists() {
                file = File::create(&file_name).unwrap();
            } else {
                file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(&file_name)
                    .unwrap();
            }
            if entry_count == 0 {
                let mut wtr = csv::Writer::from_writer(&file);
                wtr.write_record(&[
                    "i",
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

            let mut wtr = csv::Writer::from_writer(&file);
            for record in results.clone().into_iter() {
                wtr.write_record(record).unwrap();
            }
            wtr.flush().unwrap();
            println!("Finsihed benchmarks and saved data into {}", &file_name);
        }
        None => {}
    }

    match args.image {
        Some(image_file) => {
            let image_data = match args.file {
                Some(ref file_name) => {
                    let mut parsed_data = Vec::new();
                    let file = OpenOptions::new()
                        .read(true)
                        .open(&file_name)
                        .unwrap();
                    
                    let mut rdr = csv::Reader::from_reader(file);
                    for result in rdr.deserialize() {
                        parsed_data.push(result.unwrap());
                    }

                    parsed_data
                }
                None => {
                    results
                }
            };

            draw_image(&image_file, &image_data);
        }
        None => {}
    }

}
