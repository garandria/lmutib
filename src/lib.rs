use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use rand::{thread_rng, Rng};
use std::process::Command;
use git2::{Index, IndexAddOption, Repository, Oid, Error};
use flate2::read::GzDecoder;
use tar::Archive;


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

pub fn kernel_download(version: &str) -> Result<String, ()> {

    let url = ["https://cdn.kernel.org/pub/linux/kernel/v",
               &version[..version.find('.').unwrap()],
               ".x/linux-", version, ".tar.gz"].join("");

    if !Command::new("wget").arg(&url).status().unwrap().success() {
        return Err(());
    }

    Ok(url[url.rfind('/').unwrap()+1..].to_string())
}

pub fn extract_tar(file: &str, dst: &str) -> Result<String, std::io::Error>{
    Archive::new(GzDecoder::new(fs::File::open(file)?)).unpack(dst)?;
    let mut sep = String::new();
    if dst.chars().last().unwrap() != '/' {
        sep.push_str("/");
    }
    Ok([dst, file.strip_suffix(".tar.gz").unwrap()].join(&sep))
}

pub struct KernelDir {
    pub git: String,
}

impl KernelDir {

    pub fn new(p: &str) -> Self {
        if !Path::new(&[p, ".git"].join("/")).is_dir(){
            let _ = Command::new("git")
                .arg("init")
                .current_dir(p)
                .output()
                .expect("git: failed init.");
        }
        let kd = Self {git: p.to_string()};
        kd.add_all();
        kd.save("source");
        kd
    }

    pub fn add_all(&self) {
        let _ = Command::new("git")
            .args(["add", "-f", "*"])
            .current_dir(&self.git)
            .output()
            .expect("git: failed add");
    }

    pub fn save(&self, msg: &str){
        let _ = Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(&self.git)
            .output()
            .expect("git: failed to commit");
    }

    pub fn create_new_branch(&self, from: Option<&str>, branch: &str) {
        if from.is_some() {
            self.checkout(from.unwrap());
        }

        let _ = Command::new("git")
            .args(["checkout", "-b", branch])
            .current_dir(&self.git)
            .output()
            .expect("git: failed to create branch");
    }

    pub fn checkout(&self, branch: &str) {
        let _ = Command::new("git")
            .args(["checkout", branch])
            .current_dir(&self.git)
            .output()
            .expect("git: failed to checkout");
    }

    pub fn get_workdir(&self) -> &str {
        &self.git
    }

    fn to_workdir(&self, dir: &str) -> String {
        [self.get_workdir(), dir].join("/").to_string()
    }

    pub fn randconfig(&self) {
        let _ = Command::new("make")
            .arg("randconfig")
            .current_dir(self.get_workdir())
            .output()
            .expect("make: failed to exectue randconfig process.");
    }

    pub fn defconfig(&self) {
        let _ = Command::new("make")
            .arg("defconfig")
            .current_dir(self.get_workdir())
            .output()
            .expect("make: failed to exectue randconfig process.");
    }

    pub fn build(&self) -> Result<(), ()>{
        let output = Command::new("/usr/bin/time")
            .args(["-p", "-a", "-o", "t+time", "--format=%e", "make", "-j16"])
            .current_dir(self.get_workdir())
            .output()
            .expect("/usr/bin/time: failed to execute build process.");

        let _ = fs::File::create(self.to_workdir("t+build"))
            .unwrap().write_all(&output.stdout);

        if !output.status.success() {
            let _ = fs::File::create(self.to_workdir("t+error")).unwrap()
                .write_all(&output.stderr);
            return Err(());
        }
        Ok(())
    }

    pub fn makeni_trace(&self) {
        let output = Command::new("make")
            .args(["-ni"])
            .current_dir(self.get_workdir())
            .output()
            .expect("make: failed to exectue randconfig process.");

        let _ = fs::File::create(self.to_workdir("t+makeni"))
            .unwrap().write_all(&output.stdout);
    }

    pub fn get_time(&self) -> Option<String>{
        fs::read_to_string(self.to_workdir("t+time")).ok()
    }
}
