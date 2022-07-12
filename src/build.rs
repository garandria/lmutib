use crate::git::Git;
use lmutib;
use std::process::Command;
use std::io::Write;
use std::path::Path;
use std::fs;
use std::collections::HashMap;



pub struct Build {
    git: Git
}

impl Build {

    pub fn new(folder: &str) -> Self {
        Self { git: Git::new(folder) }
    }

    pub fn init(&self) {
        let _ = self.git.config("Tux", "Tux");
        let aoid = self.git.add_all().unwrap();
        let _ = self.git.commit("source", aoid);
    }

    pub fn get_workdir(&self) -> &str {
        self.git.get_workdir()
    }

    // 1.
    // config_name_from_path("/home/configs/___configOPT_XXX_y-0")
    //         ->  home|configs|OPT_XXX_y
    // 2.
    // config_name_from_path("/home/configs/rand")
    //         -> home|configs|rand
    //
    pub fn config_name_from_path(&self, config: &Path) -> String {
        let configprefix = "___config";
        let pref = config.strip_prefix("/").unwrap().parent().unwrap()
            .to_str().unwrap().replace("/", "|");
        let f = config.file_name().unwrap().to_str().unwrap();
        if f.starts_with(configprefix) {
            let extr = f.strip_prefix(configprefix)
                .unwrap().split("-").next().unwrap();
            [pref, extr.to_string()].join("|").to_string()
        }else {
            [pref, f.to_string()].join("|").to_string()
        }
    }

    pub fn build_ok(&self) -> bool {
        return !Path::exists(
            Path::new(&[self.get_workdir(), "t+error"].join("/")))
    }

    pub fn build_time(&self) -> String {
        fs::read_to_string([self.get_workdir(), "t+time"].join("/"))
            .unwrap().trim().to_string()
    }

    pub fn extract_parent_from_bname(&self, name: &str) -> String {
        if name.ends_with("cb") {
            "--".to_string()
        }else {
            let mut splitted = name.split("-");
            splitted.next().unwrap().to_string()
        }
    }

    pub fn extract_config_from_bname(&self, name: &str) -> String {
        if name.ends_with("cb") {
            name.strip_suffix("-cb").unwrap().to_string()
        }else {
            let mut splitted = name.split("-");
            splitted.next(); splitted.next(); splitted.next().unwrap().to_string()
        }
    }

    pub fn report(&self) {
        let mut results: HashMap<String, String> = HashMap::new();
        for b in self.git.branches().unwrap()
            .map(|br| br.unwrap().0.name().unwrap().unwrap().to_string())
            .filter(|bn| bn != "master"){
                self.git.checkout(&b);
                if self.build_ok() {
                    results.insert(b, self.build_time());
                }else {
                    results.insert(b, "--".to_string());
                }
            }
        println!("name,parent,clean,incremental,add,remove,change");
        for (k, v) in &results {
            if k.ends_with("ib") {
                self.git.checkout(&k);
                let parent = self.extract_parent_from_bname(&k);
                let name = self.extract_config_from_bname(&k);
                let diff = lmutib::diffconfig(Path::new(&[self.get_workdir(), ".config.old"].join("/")),
                                              Path::new(&[self.get_workdir(), ".config"].join("/")));
                println!("{},{},{},{},{},{},{}", name, &parent,
                         results.get(&[parent.to_string(), "cb".to_string()].join("-")).unwrap(), v,
                         diff.get("+").unwrap().len(), diff.get("-").unwrap().len(), diff.get("~").unwrap().len());

            }else {
                let name = self.extract_config_from_bname(&k);
                println!("{},--,{},--,--,--,--", name, v);
            }
        }
    }

    pub fn build(&self) -> Result<(), ()>{
        let output = Command::new("/usr/bin/time")
            .args(["-p", "-o", "t+time", "--format=%e", "make", "-j16"])
            .current_dir(self.git.get_workdir())
            .output()
            .expect("/usr/bin/time: failed to execute build process.");

        let _ = fs::File::create([self.git.get_workdir(), "t+build"]
                                 .join("/"))
            .unwrap().write_all(&output.stdout);

        if !output.status.success() {
            let _ = fs::File::create([self.git.get_workdir(), "t+error"]
                                     .join("/")).unwrap()
                .write_all(&output.stderr);
            return Err(());
        }
        Ok(())
    }

    pub fn build_from(&self, branch: &str, config: &Path) {
        // checkout to branch
        self.git.checkout(branch);
        // new branch from master with the given config
        let last = self.git.get_last_commit().unwrap();
        let new_branch_name: String;
        if branch == "master" {
            new_branch_name =
                [self.config_name_from_path(config), "cb".to_string()].join("-")
                .to_string();
        }else {
            new_branch_name =
                [branch, &self.config_name_from_path(config),
                 "ib"].join("-").to_string();
        }
        self.git.create_branch(&new_branch_name, last);
        self.git.checkout(&new_branch_name);
        // move config here
        if branch != "master" {
            let _ = fs::copy([self.get_workdir(), ".config"].join("/"),
                             [self.get_workdir(), ".config.old"].join("/"));
        }
        let _ = fs::copy(config.to_str().unwrap(),
                         [self.get_workdir(), ".config"].join("/"));
        // build
        let _ = self.build();
        // add and commit
        let oid = self.git.add_all().unwrap();
        let _ = self.git.commit("Build", oid);
    }

    pub fn clean_build(&self, config: &Path) {
        self.build_from("master", config);
    }

    pub fn incremental_build(&self, src_branch: &str, dst_config: &Path) {
        self.build_from(src_branch, dst_config);
    }

}
