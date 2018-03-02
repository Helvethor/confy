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
use config::{Config, ConfigFiles, PathBinding};


fn main() {
    log_init();

    let matches = App::new("Confy")
        .version("0.2.2")
        .author("Vincent Pasquier")
        .about("Continuously substitute key-value pairs accross multiple configuration files")
        .arg(Arg::with_name("bindings_file")
            .help("Bindings file (.yaml)")
            .short("b")
            .long("bindings")
            .value_name("BINDINGS_FILE")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("variables_file")
            .help("Variables file (.yaml)")
            .short("v")
            .long("variables")
            .value_name("BINDINGS_FILE")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("watch_files")
            .help("Update to-part of a binding when from-part is modified")
            .short("F")
            .long("watch-files"))
        .arg(Arg::with_name("watch_bindings")
            .help("Update internal configuration when the bindings file is modified")
            .short("B")
            .long("watch-bindings"))
        .arg(Arg::with_name("watch_variables")
            .help("Update internal configuration when the variables file is modified")
            .short("V")
            .long("watch-variables"))
        .get_matches();

    debug!("{:?}", matches);

    let config_files = ConfigFiles {
        bindings: Path::new(matches.value_of("bindings_file").unwrap()),
        variables: Path::new(matches.value_of("variables_file").unwrap()),
    };
    let mode = Mode {
        files: matches.is_present("watch_files"),
        bindings: matches.is_present("watch_bindings"),
        variables: matches.is_present("watch_variables")
    };

    if !mode.bindings && !mode.variables && !mode.files{
        let config = match Config::new(&config_files) {
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
        let mut watcher = match Watcher::new(config_files, mode) {
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
