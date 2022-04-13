use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use rand::{thread_rng, Rng};


pub fn mkf_ni_trace(trace: &Path) -> HashMap<String, HashMap<String, String>> {

    let file = match fs::File::open(trace) {
        Ok(file) => file,
        Err(err) => panic!("Error: {:?} {}", trace, err),
    };

    let mut tasks: HashMap<String, HashMap<String, String>> = HashMap::new();

    for line in BufReader::new(file).lines()
        .filter(|l| l.is_ok()).map(|l| l.unwrap()) {
            if line.starts_with("set -e;  echo '  ") {
                let echoed = line.strip_prefix("set -e;  echo '  ")
                    .unwrap().trim();
                let mut splitted = echoed[..echoed.find('\'').unwrap()]
                    .split_whitespace();
                let rule = splitted.nth(0).unwrap().to_string();
                let target = splitted.nth(0).unwrap().to_string();
                let cmd = echoed[echoed.find(';').unwrap()+1..]
                    .trim().to_string();

                if !tasks.contains_key(&rule) {
                    tasks.insert(String::from(&rule), HashMap::new());
                }
                if let Some(table) = tasks.get_mut(&rule) {
                    table.insert(String::from(&target), String::from(&cmd));
                }
            }
        }

    tasks
}


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

pub fn diffconfig(config1: &Path, config2: &Path)
              -> HashMap<String, HashMap<String, String>> {

    let c1 = readconfig(config1);
    let c2 = readconfig(config2);

    let mut
        comparison: HashMap<String, HashMap<String, String>> = HashMap::new();
    comparison.insert("=".to_string(), HashMap::new());
    comparison.insert("+".to_string(), HashMap::new());
    comparison.insert("-".to_string(), HashMap::new());
    comparison.insert("~".to_string(), HashMap::new());

    for (k, v) in c1.iter() {
        if c2.contains_key(k) {
            if c1.get(k) == c2.get(k) {
                comparison.get_mut("=").unwrap()
                    .insert(k.to_string(), v.to_string());
            }else {
                comparison.get_mut("~").unwrap()
                    .insert(k.to_string(),
                            format!("{} -> {}", c1.get(k).unwrap().to_string(),
                                    c2.get(k).unwrap().to_string()));
            }
        }else {
            comparison.get_mut("-").unwrap()
                .insert(k.to_string(), v.to_string());
        }
    }
    for (k, v) in c2.iter() {
        if !c1.contains_key(k) {
            comparison.get_mut("+").unwrap()
                .insert(k.to_string(), v.to_string());
        }
    }
    comparison
}
