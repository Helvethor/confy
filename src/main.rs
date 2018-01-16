mod generator;
mod watcher;
mod config;
mod variables;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate inotify;
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::process::exit;
use std::path::Path;
use std::env;
use std::io::Write;
use log::Level;
use env_logger::Color;
use clap::{App, Arg};

use watcher::{Watcher, Mode};
use generator::Generator;
use config::{Config, PathBinding};


fn main() {
    log_init();

    let matches = App::new("Generator")
        .version("0.1")
        .author("Vincent Pasquier")
        .about("Continuously make substitue key-value pairs accross multiple configuration files")
        .arg(Arg::with_name("config_file")
            .help("Configuration file (.yaml)")
            .short("c")
            .long("config")
            .value_name("FILE")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("watch_bindings")
            .help("Update to-part of a binding when from-part is modified")
            .short("B")
            .long("watch-bindings"))
        .arg(Arg::with_name("watch_config")
            .help("Update internal configuration when the config file is modified")
            .short("C")
            .long("watch-config"))
        .arg(Arg::with_name("verbose")
            .help("Output runtime informations"))
        .get_matches();

    let config_file = Path::new(matches.value_of("config_file").unwrap());
    let mode = Mode {
        bindings: matches.is_present("watch_bindings"),
        config: matches.is_present("watch_config")
    };

    if !mode.bindings && !mode.config {
        let config = match Config::new(config_file) {
            Ok(c) => c,
            Err(e) => {
                error!("{}", e);
                exit(1);
            }
        };

        let generator = Generator::new(&config.variables);
        for binding in config.bindings.iter() {
            match generator.process(binding) {
                Ok(n) => info!(
                    "{}: replaced {} key(s)",
                    binding.from.display(), n),
                Err(e) => error!("{}", e)
            };
        }
    }
    else {
        let mut watcher = match Watcher::new(config_file, mode) {
            Ok(w) => w,
            Err(e) => {
                error!("{}", e);
                exit(1);
            }
        };
        watcher.watch();
    }
}

fn log_init() {
    let mut builder = env_logger::Builder::new();
     
    builder.format(|buf, record| {
        let level = record.level();
        let mut level_style = buf.style();
        match level {
            Level::Trace => level_style.set_color(Color::White),
            Level::Debug => level_style.set_color(Color::Blue),
            Level::Info => level_style.set_color(Color::Green),
            Level::Warn => level_style.set_color(Color::Yellow),
            Level::Error => level_style.set_color(Color::Red).set_bold(true),
        };
        if let Some(module_path) = record.module_path() {
            writeln!(buf, "[{:>5}] {}: {}", level_style.value(level),
                module_path, record.args())
        }
        else {
            writeln!(buf, "[{:>5}] {}", level_style.value(level), record.args())
        }
    });

    if let Ok(rust_log) = env::var("RUST_LOG") {
       builder.parse(&rust_log);
    }

    builder.init();
}
