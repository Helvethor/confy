use std::error::Error;
use std::io::{BufReader, BufWriter};
use std::io::prelude::*;
use std::fs::File;
use std::ops::Range;
use std::ops::Index;
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
                    let expression = &remaining[start + 3..stop];
                    match self.parse_expression(expression) {
                        Some(value) => {
                            output.push_str(&value);
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

    fn parse_expression(&self, expression: &str) -> Option<String> {

        let mut key = expression;
        let range_str = match expression.find("[") {
            Some(start) => {
                match expression.find("]") {
                    Some(stop) => {
                        key = &expression[..start];
                        Some(&expression[start + 1..stop])
                    },
                    None => return None
                }
            },
            None => None
        };
        
        let value = match self.variables.get(key) {
            Some(value) => match range_str {
                Some(range_str) => match self.parse_range(value, range_str) {
                    Some(range) => {
                        let value = value.index(range.clone());
                        debug!("{}[{:?}] = {}", key, range, value);
                        value
                    }
                    None => return None
                },
                None => value
            },
            None => return None
        };

        Some(value.to_string())
    }

    fn parse_range(&self, value: &str, range: &str) -> Option<Range<usize>> {
        match range.find("..") {
            Some(dot) => {
                let start = {
                    if dot == 0 {
                        0
                    }
                    else {
                        match range[..dot].parse::<usize>() {
                            Ok(start) => {
                                if start >= value.len() {
                                    value.len() - 1
                                }
                                else {
                                    start
                                }
                            },
                            Err(_) => return None
                        }
                    }
                };
                let end = {
                    if dot + 2 == range.len() {
                        value.len()
                    }
                    else {
                        debug!("end: {}", &range[dot + 2..]);
                        match range[dot + 2..].parse::<usize>() {
                            Ok(end) => {
                                if end > value.len() {
                                    value.len()
                                }
                                else {
                                    end
                                }
                            },
                            Err(_) => return None
                        }
                    }
                };
                
                Some(Range { start, end })
            },
            None => None
        }
    }
}
