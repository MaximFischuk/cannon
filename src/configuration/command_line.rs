use crate::configuration::constants::cargo_env::CARGO_PKG_NAME;
use clap::arg_enum;
use log::LevelFilter;
use std::path::PathBuf;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum LogLevel {
        Off, Error, Warn, Info, Debug, Trace,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = CARGO_PKG_NAME)]
pub struct Opt {
    /// Set a custom configuration file. Supported: YAML, JSON, TOML, HJSON
    #[structopt(parse(from_os_str))]
    pub file: PathBuf,

    /// Sets a logging level
    #[structopt(case_insensitive = true, long, short = "L", possible_values = &LogLevel::variants(), env = "LOG_LEVEL")]
    pub logging: Option<LogLevel>,

    /// FIle to which application will write logs
    #[structopt(long, short = "O", env = "LOG_OUTPUT_FILE")]
    pub log_output_file: Option<PathBuf>,

    /// Amount of parallel threads of worker groups
    #[structopt(long, short = "t")]
    pub threads: Option<usize>,

    /// Run only defined groups, any other will be ignored
    #[structopt(long, short = "g")]
    pub groups: Vec<String>,
}

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
