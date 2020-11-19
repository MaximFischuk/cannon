// #![forbid(unsafe_code)]
// #![deny(non_upper_case_globals)]
// #![deny(non_camel_case_types)]
// #![deny(non_snake_case)]
// #![deny(unused_mut)]
// #![deny(unused_variables)]
// #![deny(dead_code)]
// #![deny(unused_imports)]
//#![deny(missing_docs)]
//#![deny(warnings)]

extern crate chrono;
extern crate derivative;
extern crate lazy_static;
extern crate reqwest;
extern crate serde_derive;
extern crate uuid;

#[macro_use]
extern crate log;

#[macro_use]
extern crate derive_builder;

mod app;
mod configuration;
// mod reporter;
mod time;

use log::LevelFilter;
use signal_hook::{iterator::Signals, SIGINT};
use std::{path::PathBuf, process::exit, thread};
use structopt::StructOpt;

use self::app::App;
use self::{
    configuration::command_line::{LogLevel, Opt},
    configuration::manifest::Manifest,
};

fn main() {
    let options = Opt::from_args();
    let signals = Signals::new(&[SIGINT]).unwrap();

    thread::spawn(move || {
        for sig in signals.forever() {
            info!("Received signal {:?}, stopping", sig);
            exit(0);
        }
    });

    let manifest = Manifest::from(options.file);

    init_logging(
        options.logging.unwrap_or(LogLevel::Info).into(),
        &options.log_output_file,
    );

    match manifest {
        Ok(manifest) => {
            debug!("Initiated configuration {:#?}", manifest);
            let app = App::new(manifest);
            app.run();
        }
        Err(e) => error!("Failed to load manifest file configuration {}", e),
    }
}

fn init_logging(level: LevelFilter, output: &Option<PathBuf>) {
    let mut dispatcher = fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}:{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record
                    .line()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "".to_owned()),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout());

    if let Some(log_file) = output {
        dispatcher = dispatcher.chain(fern::log_file(log_file).unwrap())
    }
    dispatcher.apply().unwrap();
    info!("Logging level {} enabled", level);
}
