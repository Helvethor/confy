mod confy;
mod watcher;
mod config;
mod variables;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate inotify;
extern crate clap;

use std::process::exit;
use std::path::{Path, PathBuf};
use clap::{App, Arg};

use watcher::{Watcher, Mode};
use confy::Confy;
use config::Config;


#[derive(Debug, Serialize, Deserialize)]
pub struct PathBinding {
    pub from: PathBuf,
    pub to: PathBuf,
}


fn main() {
    let matches = App::new("Confy")
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
                eprint!("{}\n", e);
                exit(1);
            }
        };

        let confy = Confy::new(&config.variables);
        for binding in config.bindings.iter() {
            match confy.process(binding) {
                Ok(n) => print!(
                    "{}: replaced {} key(s)\n",
                    binding.from.display(), n),
                Err(e) => eprint!("{}\n", e)
            };
        }
    }
    else {
        let mut watcher = match Watcher::new(config_file, mode) {
            Ok(w) => w,
            Err(e) => {
                eprint!("{}\n", e);
                exit(1);
            }
        };
        watcher.watch();
    }
}

