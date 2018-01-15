use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::fs::File;

use serde_yaml;

use ::PathBinding;


#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bindings: Vec<PathBinding>,
    pub variables: HashMap<String, String>
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

        if let Some(config_dir) = config_file.parent() {
            for binding in config.bindings.iter_mut() {
                if binding.from.is_relative() {
                    binding.from = config_dir.join(&binding.from);
                }
                if binding.to.is_relative() {
                    binding.to = config_dir.join(&binding.to);
                }
            }
        }

        Ok(config)
    }
}
