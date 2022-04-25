use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use rand::{thread_rng, Rng};
use std::process::Command;
use git2::{Index, IndexAddOption, Repository, Oid, Config};
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

pub fn mkf_ni_trace_total(table: HashMap<String, HashMap<String, String>>)
                          -> usize {
    let mut total = 0;
    for (_, v) in &table {
        total += v.len();
    }
    total
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


pub struct MyGit {
    pub repo: Repository,
}

impl MyGit {

    pub fn new(folder: &str) -> Self {
        Self {repo: Repository::init(folder).unwrap()}
    }

    pub fn config(&self, user_name: &str, user_email: &str)
                  -> Result <Config, git2::Error>{

        let mut conf = Config::new().unwrap();
        let path_str = [&self.repo.path().to_str().unwrap(),
                        "config"].join("/");
        conf.add_file(Path::new(&path_str),
                      git2::ConfigLevel::Local,
                      true)?;
        conf.set_str("user.name", user_name)?;
        conf.set_str("user.email", user_email)?;
        Ok(conf)
    }

    pub fn add_all(&self) -> Result<Oid, git2::Error>{

        let cb = &mut |path: &Path, _matched_spec: &[u8]| -> i32 {
            let status = self.repo.status_file(path).unwrap();
            if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::WT_NEW){
                    0
                }else {
                    1
                }
        };

        let mut index = self.repo.index()
            .expect("MyGit: cannot get the Index file.");
        index.add_all(["*"].iter(),
                      IndexAddOption::DEFAULT,
                      Some(cb as &mut git2::IndexMatchedPath))
            .expect("MyGit: failed to add -f *");
        index.write().expect("MyGit: failed to write modification.");
        index.write_tree()
    }

    pub fn commit(&self, msg: &str, tree_id: Oid) -> Result<Oid, git2::Error>  {
        let signature = self.repo.signature().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();
        let head = self.repo.head();
        let ret: Result<Oid, git2::Error>;
        match head {
            Ok(h) => {
                ret = self.repo.commit(Some("HEAD"),
                                        &signature,
                                        &signature,
                                        msg,
                                        &tree,
                                        &[&h.peel_to_commit().unwrap()]
                );
            }
            Err(_)   => {
                ret = self.repo.commit(Some("HEAD"),
                                        &signature,
                                        &signature,
                                        msg,
                                        &tree,
                                        &[]
                );
            }
        }
        ret
    }

    pub fn create_branch(&self, branch_name: &str, src_commit: Oid) {
        let srccommit = self.repo.find_commit(src_commit).unwrap();
        let _ = self.repo.branch(branch_name, &srccommit, false);
    }

    pub fn checkout(&self, branch: &str) {
        let (object, reference) = self.repo.revparse_ext(branch).expect("Object not found");

        self.repo.checkout_tree(&object, None)
            .expect("Failed to checkout");

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => self.repo.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => self.repo.set_head_detached(object.id()),
        }
        .expect("Failed to set HEAD");
    }

    pub fn get_workdir(&self) -> &str {
        &self.repo.workdir().unwrap().to_str().unwrap()
    }
}
