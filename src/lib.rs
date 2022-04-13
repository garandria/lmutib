use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn readconfig(config: &Path) -> HashMap<String, String> {

    let file = match fs::File::open(config) {
        Ok(file) => file,
        Err(err) => panic!("Error: {:?} {}", config, err),
    };

    let mut configuration = HashMap::new();

    for line in BufReader::new(file).lines()
        .filter(|l| l.is_ok()).map(|l| l.unwrap()) {
            if line.starts_with('#') {
                if line.ends_with("is not set") {
                    let option = line
                        .strip_prefix("# CONFIG_").unwrap()
                        .strip_suffix(" is not set").unwrap();
                    configuration.insert(option.to_string(), "n".to_string());
                }
            }else {
                if !line.is_empty() {
                    let (option, value) = line.split_once('=').unwrap();
                    configuration.insert(option.to_string(), value.to_string());
                }
            }
        }
    configuration
}
