use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::env;
use std::fs::File;

use serde_yaml;


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bindings: Vec<PathBinding>,
    pub variables: HashMap<String, String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathBinding {
    pub from: PathBuf,
    pub to: PathBuf,
}

pub struct ConfigFiles<'a> {
    pub bindings: &'a Path,
    pub variables: &'a Path
}


impl Config {
    pub fn new(config_files: &ConfigFiles) -> Result<Config, String>{
        let bindings_file = match File::open(config_files.bindings) {
            Ok(f) => f,
            Err(e) => return Err(format!(
                "Couldn't open {}: {}",
                config_files.bindings.display(),
                e.description()
            ))
        };

        let mut bindings: Vec<PathBinding>
            = match serde_yaml::from_reader(bindings_file) {
            Ok(c) => c,
            Err(e) => return Err(format!(
                "Couldn't parse {}: {}",
                config_files.bindings.display(),
                e.description()
            ))
        };

        let config_dir = config_files.bindings.parent();
        for binding in bindings.iter_mut() {
            binding.from = Config::resolve_path(&binding.from, config_dir);
            binding.to = Config::resolve_path(&binding.to, config_dir);
        }

        let variables_file = match File::open(config_files.variables) {
            Ok(f) => f,
            Err(e) => return Err(format!(
                "Couldn't open {}: {}",
                config_files.variables.display(),
                e.description()
            ))
        };

        let variables: HashMap<String, String>
            = match serde_yaml::from_reader(variables_file) {
            Ok(c) => c,
            Err(e) => return Err(format!(
                "Couldn't parse {}: {}",
                config_files.variables.display(),
                e.description()
            ))
        };


        Ok(Config {
            bindings,
            variables
        })
    }

    fn resolve_path(path: &Path, parent: Option<&Path>) -> PathBuf {

        if path.is_relative() {

            if path.starts_with("~") {
                match env::var("HOME") {
                    Ok(ref home) => {
                        let path = path.strip_prefix("~").unwrap();
                        return Path::new(home).join(path)
                    },
                    Err(e) => {
                        warn!("{}", e);
                    }
                };
            }

            else if let Some(parent) = parent {
                return parent.join(path);
            }
        }

        PathBuf::from(path)
    }
}
