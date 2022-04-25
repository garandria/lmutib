use lmutib;
use reqwest;
use std::fs::{File, self};
use std::io::copy;
use std::env;
use std::path::Path;
use git2::{Index, IndexAddOption, Repository, Oid, Error, Config};

fn main() {

    let kernel = "";
    println!("┌───────────────────────────┐");
    println!("· Experiment initialization ·");
    println!("└───────────────────────────┘");

    println!("  → Kernel directory: {}", kernel);
    print!("  → Initializing git directory...");
    let git = lmutib::MyGit::new(kernel);
    println!(" ✓");

    print!("  → Local git configuration...");
    git.config("Tux", "None");
    println!(" ✓");

    print!("  → Adding source...");
    let add1 = match git.add_all() {
        Ok (oid)  => oid,
        Err(err)  => panic!("\t/!\\ Error adding all.\n\t{:?}", err),
    };
    println!(" ✓");

    print!("  → Committing sources...");
    let src_commit = match git.commit("source", add1) {
        Ok (oid) => oid,
        Err(err) => panic!("\t/!\\ Error commit.\n\t{:?}", err),
    };
    println!(" ✓");

    println!("┌───────────────────────────┐");
    println!("·   Starting build tasks    ·");
    println!("└───────────────────────────┘");

    let data_configs = Path::new("/tmp/data-configs");

    for folder in fs::read_dir(data_configs).unwrap().filter(|f| f.is_ok())
        .map(|f| f.unwrap()) {

            let dir_name = folder.file_name().into_string().unwrap();
            let base_config_path = [folder.path().to_str().unwrap(),
                                    "config"].join("/");
            let base_config_branch = [&dir_name, "base", "cb"]
                .join("-");

            println!("  •  Folder: {}", &dir_name);
            println!("  └─ Base configuration: {}", base_config_path);
            print!("      → Creating new branch {}...", base_config_branch);
            git.create_branch(&base_config_branch, src_commit);
            println!(" ✓");
            print!("      → Checkout to {}...", base_config_branch);
            git.checkout(&base_config_branch);
            print!("      → Copying configuration...", );
            match fs::copy(base_config_path, [kernel, ".config"].join("/")) {
                Ok (_) => println!(" ✓"),
                Err(_) => panic!(" x"),
            };
            println!(" ✓");
            print!("      → Clean build...");
            match lmutib::build(kernel) {
                Ok (_) => {
                    print!(" ✓");
                    println!(" {}s", fs::read_to_string([kernel, "t+time"]
                                                        .join("/"))
                             .unwrap());
                },
                Err(_) => {
                    println!(" x");
                    println!("      ‗‗Trace‗‗\n      {:?}",
                             fs::read_to_string([kernel, "t+time"]
                                                .join("/")).unwrap());
                }
            };
            print!("      → Adding all...");
            let addbase = match git.add_all() {
                Ok (oid)  => {
                    println!(" ✓");
                    oid
                },
                Err(err)  => panic!(" x\n\t{:?}", err),
            };

            print!("      → Committing...");
            let base_cb_commit = match git.commit("clean build", addbase) {
                Ok (oid) => {
                    println!(" ✓");
                    oid
                },
                Err(err) => panic!(" x\n\t{:?}", err),
            };

            for config in fs::read_dir(folder.path()).unwrap().filter(|f| f.is_ok())
                .map(|f| f.unwrap()){
                    let file_name = config.file_name().into_string().unwrap();
                    if file_name.starts_with("___config") {
                        let config_path = [folder.path().to_str().unwrap(),
                                           &file_name].join("/");
                        let config_branch = [&dir_name, &file_name, "cb"]
                            .join("-");

                        println!("    └─ Considering {}", file_name);
                        print!("          → Creating new branch {}...", config_branch);
                        git.create_branch(&config_branch, src_commit);
                        println!(" ✓");
                        print!("          → Checkout to {}...", config_branch);
                        git.checkout(&config_branch);
                        print!("          → Copying configuration...", );
                        match fs::copy(&config_path, [kernel, ".config"].join("/")) {
                            Ok (_) => println!(" ✓"),
                            Err(_) => panic!(" x"),
                        };
                        println!(" ✓");
                        print!("          → Clean build...");
                        match lmutib::build(kernel) {
                            Ok (_) => {
                                print!(" ✓");
                                println!(" {}s", fs::read_to_string([kernel, "t+time"]
                                                                    .join("/"))
                                         .unwrap());
                            },
                            Err(_) => {
                                println!(" x");
                                println!("      ‗‗Trace‗‗\n      {:?}",
                                         fs::read_to_string([kernel, "t+time"]
                                                            .join("/")).unwrap());
                            }
                        };
                        print!("          → Adding all...");
                        let addcb = match git.add_all() {
                            Ok (oid)  => {
                                println!(" ✓");
                                oid
                            },
                            Err(err)  => panic!(" x\n\t{:?}", err),
                        };

                        print!("          → Committing...");
                        let cb_commit = match git.commit("clean build", addcb) {
                            Ok (oid) => {
                                println!(" ✓");
                                oid
                            },
                            Err(err) => panic!(" x\n\t{:?}", err),
                        };

                        let config_branch_ib = [&dir_name, &file_name, "ib"]
                            .join("-");

                        print!("          → Creating new branch {}...", config_branch_ib);
                        git.create_branch(&config_branch_ib, cb_commit);
                        println!(" ✓");
                        print!("          → Checkout to {}...", config_branch);
                        git.checkout(&config_branch_ib);
                        print!("          → Copying configuration...", );
                        match fs::copy(&config_path, [kernel, ".config"].join("/")) {
                            Ok (_) => println!(" ✓"),
                            Err(_) => panic!(" x"),
                        };
                        println!(" ✓");
                        print!("          → Incremental build ({} → {})...",
                               base_config_branch, config_branch_ib);
                        match lmutib::build(kernel) {
                            Ok (_) => {
                                print!(" ✓");
                                println!(" {}s", fs::read_to_string([kernel, "t+time"]
                                                                    .join("/"))
                                         .unwrap());
                            },
                            Err(_) => {
                                println!(" x");
                                println!("      ‗‗Trace‗‗\n      {:?}",
                                         fs::read_to_string([kernel, "t+time"]
                                                            .join("/")).unwrap());
                            }
                        };
                        print!("          → Adding all...");
                        let addib = match git.add_all() {
                            Ok (oid)  => {
                                println!(" ✓");
                                oid
                            },
                            Err(err)  => panic!(" x\n\t{:?}", err),
                        };

                        print!("          → Committing...");
                        let ib_commit = match git.commit("clean build", addib) {
                            Ok (oid) => {
                                println!(" ✓");
                                oid
                            },
                            Err(err) => panic!(" x\n\t{:?}", err),
                        };
                    }
                }
        }
}
