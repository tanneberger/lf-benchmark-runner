extern crate derive_builder;

use clap::Parser;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[clap(name = "Lingua-Franca benchmark runner")]
#[clap(author = "Tassilo Tanneberger <tassilo.tanneberger@tu-dresden.de>")]
#[clap(version = "1.0")]
#[clap(about = "Runns specified captures and extracts times.", long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub binary: String,

    #[clap(short, long)]
    pub target: String,

    #[clap(short, long)]
    pub file: String,

    #[clap(short, long, default_value_t = String::from(""))]
    pub name: String,

    #[clap(long, default_value_t = String::from("master"))]
    pub runtime_version: String,

    #[clap(long, default_value_t = 1)]
    pub number_of_runs: u32,

    #[clap(long, default_value_t = 1)]
    pub threads: u8,

    #[clap(long)]
    pub json: bool
}

pub fn format<T: std::fmt::Display>(option: &Option<T>) -> String {
    match option {
        Some(value) => {
            format!("{}", value)
        }
        None => {
            format!("")
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonResult {
    name: String,
    units: String,
    value: f32,
    extra: String
}


#[derive(Default, Builder)]
pub struct Result {
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

    pub fn to_json(self: &Result) -> JsonResult {
        JsonResult {
            name: self.benchmark_name.clone(),
            units: String::from("ms"),
            value: self.mean_time_ms,
            extra: String::from("")
        }
    }
}

