use std::error::Error;
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use config::PathBinding;
use variables::Variables;

#[derive(Debug)]
pub struct Generator {
    variables: Variables,
}

impl Generator {
    pub fn new(variables: &HashMap<String, String>) -> Generator {
        Generator {
            variables: Variables::new(variables)
        }
    }

    pub fn process(&self, binding: &PathBinding) -> Result<u32, String> {

        let from = match File::open(&binding.from) {
            Ok(f) => BufReader::new(f),
            Err(e) => {
                return Err(format!(
                    "Couldn't open {}: {}",
                    binding.from.display(),
                    e.description()))
            }
        };

        let mut to = match File::create(&binding.to) {
            Ok(f) => BufWriter::new(f),
            Err(e) => {
                return Err(format!(
                    "Couldn't open {}: {}",
                    binding.to.display(),
                    e.description()))
            }
        };


        let mut replacements = 0;
        let mut output = String::new();

        for line in from.lines() {

            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    return Err(format!(
                        "Error reading from {}: {}",
                        binding.from.display(),
                        e.description()));
                }
            };

            output.truncate(0);
            replacements += self.process_line(&line, &mut output);
            match to.write(output.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!(
                        "Error writing to {}: {}",
                        binding.to.display(),
                        e.description()));
                }
            }
        }

        Ok(replacements)
    }

    fn process_line(&self, input: &String, output: &mut String) -> u32 {
        let mut remaining = &input[..];
        let mut replacements = 0;

        while let Some(start) = remaining.find("${{") {
            match remaining.find("}}") {
                Some(stop) => {
                    output.push_str(&remaining[..start]);
                    let key = &remaining[start + 3..stop];
                    match self.variables.get(key) {
                        Some(value) => {
                            output.push_str(value);
                            replacements += 1;
                        },
                        None => output.push_str(&remaining[start..stop + 2])
                    };
                    remaining = &remaining[stop + 2..];
                },
                None => break
            }
        };

        output.push_str(remaining);
        output.push('\n');

        replacements
    }
}
