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


impl Config {
    pub fn new(config_file: &Path) -> Result<Config, String>{
        let file = match File::open(config_file) {
            Ok(f) => f,
            Err(e) => return Err(format!(
                "Couldn't open {}: {}",
                config_file.display(),
                e.description()
            ))
        };

        let mut config: Config = match serde_yaml::from_reader(file) {
            Ok(c) => c,
            Err(e) => return Err(format!(
                "Couldn't parse {}: {}",
                config_file.display(),
                e.description()
            ))
        };

        let config_dir = config_file.parent();
        for binding in config.bindings.iter_mut() {
            binding.from = Config::resolve_path(&binding.from, config_dir);
            binding.to = Config::resolve_path(&binding.to, config_dir);
        }

        Ok(config)
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
