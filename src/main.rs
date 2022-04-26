use lmutib;
use std::fs::{self};
use std::io::{self, Write};
use std::path::Path;

fn main() {

    let kernel = "/home/linux-5.13";
    println!("┌───────────────────────────┐");
    io::stdout().flush().unwrap();
    println!("· Experiment initialization ·");
    io::stdout().flush().unwrap();
    println!("└───────────────────────────┘");
    io::stdout().flush().unwrap();
    println!("  → Kernel directory: {}", kernel);
    io::stdout().flush().unwrap();
    let _ = fs::remove_file([kernel, ".gitignore"].join("/"));
    print!  ("  → Initializing git directory...");
    io::stdout().flush().unwrap();
    let git = lmutib::MyGit::new(kernel);
    println!(" ✓");
    io::stdout().flush().unwrap();
    print!  ("  → Local git configuration...");
    io::stdout().flush().unwrap();
    let _ = git.config("Tux", "None");
    println!(" ✓");
    io::stdout().flush().unwrap();
    print!  ("  → Adding source...");
    io::stdout().flush().unwrap();
    let add1 = match git.add_all() {
        Ok (oid)  => oid,
        Err(err)  => panic!("\t/!\\ Error adding all.\n\t{:?}", err),
    };
    println!(" ✓");
    io::stdout().flush().unwrap();
    print!  ("  → Committing sources...");
    io::stdout().flush().unwrap();
    let src_commit = match git.commit("source", add1) {
        Ok (oid) => oid,
        Err(err) => panic!("\t/!\\ Error commit.\n\t{:?}", err),
    };
    println!(" ✓");
    io::stdout().flush().unwrap();
    println!("┌───────────────────────────┐");
    io::stdout().flush().unwrap();
    println!("·   Starting build tasks    ·");
    io::stdout().flush().unwrap();
    println!("└───────────────────────────┘");
    io::stdout().flush().unwrap();
    let data_configs = Path::new("/home/data-configs");

    for folder in fs::read_dir(data_configs).unwrap().filter(|f| f.is_ok())
        .map(|f| f.unwrap()) {

            let dir_name = folder.file_name().into_string().unwrap();
            let base_config_path = [folder.path().to_str().unwrap(),
                                    "config"].join("/");
            let base_config_branch = [&dir_name, "base", "cb"]
                .join("-");

            // CLEAN BUILD OF THE BASE CONFIGURATION
            // -------------------------------------

            println!("  •  Folder: {}", &dir_name);
            io::stdout().flush().unwrap();
            println!("  ├─ Base configuration: {}", base_config_path);
            io::stdout().flush().unwrap();
            print!  ("  │ ├─ Creating new branch {}...", base_config_branch);
            io::stdout().flush().unwrap();
            git.create_branch(&base_config_branch, src_commit);
            println!(" ✓");
            io::stdout().flush().unwrap();
            print!  ("  │ ├─ Checkout to {}...", base_config_branch);
            io::stdout().flush().unwrap();
            git.checkout(&base_config_branch);
            println!(" ✓");
            io::stdout().flush().unwrap();
            print!  ("  │ ├─ Copying configuration...", );
            io::stdout().flush().unwrap();
            match fs::copy(base_config_path, [kernel, ".config"].join("/")) {
                Ok (_) => println!(" ✓"),
                Err(_) => panic!(" x"),
            };
            io::stdout().flush().unwrap();
            print!  ("  │ ├─ Clean build...");
            io::stdout().flush().unwrap();
            match lmutib::build(kernel) {
                Ok (_) => {
                    print!(" ✓");
                    println!(" {}s", fs::read_to_string([kernel, "t+time"]
                                                        .join("/"))
                             .unwrap().trim());
                },
                Err(_) => {
                    println!(" x");
                    println!("  │   ‗‗Trace‗‗\n  │   {:?}",
                             fs::read_to_string([kernel, "t+error"]
                                                .join("/")).unwrap().trim());
                }
            };
            io::stdout().flush().unwrap();
            print!  ("  │ ├─ Adding all...");
            io::stdout().flush().unwrap();
            let addbase = match git.add_all() {
                Ok (oid)  => {
                    println!(" ✓");
                    oid
                },
                Err(err)  => panic!(" x\n\t{:?}", err),
            };
            io::stdout().flush().unwrap();
            print!  ("  │ └─ Committing...");
            io::stdout().flush().unwrap();
            let base_cb_commit = match git.commit("clean build", addbase) {
                Ok (oid) => {
                    println!(" ✓");
                    oid
                },
                Err(err) => panic!(" x\n\t{:?}", err),
            };
            io::stdout().flush().unwrap();

            // BUILDS OF THE MUTANTS
            // ---------------------

            for config in fs::read_dir(folder.path()).unwrap().filter(|f| f.is_ok())
                .map(|f| f.unwrap()){
                    let file_name = config.file_name().into_string().unwrap();
                    if file_name.starts_with("___config") {
                        let config_path = [folder.path().to_str().unwrap(),
                                           &file_name].join("/");
                        let config_branch = [&dir_name, &file_name, "cb"]
                            .join("-");


                        // CLEAN BUILD
                        // ------------

                        println!("  ├─ Considering {}", file_name);
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Creating new branch {}...", config_branch);
                        io::stdout().flush().unwrap();
                        git.create_branch(&config_branch, src_commit);
                        println!(" ✓");
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Checkout to {}...", config_branch);
                        io::stdout().flush().unwrap();
                        git.checkout(&config_branch);
                        println!(" ✓");
                        print!  ("  │ ├─ Copying configuration...");
                        io::stdout().flush().unwrap();
                        match fs::copy(&config_path, [kernel, ".config"].join("/")) {
                            Ok (_) => println!(" ✓"),
                            Err(_) => panic!(" x"),
                        };
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Clean build...");
                        io::stdout().flush().unwrap();
                        match lmutib::build(kernel) {
                            Ok (_) => {
                                print!(" ✓");
                                println!(" {}s", fs::read_to_string([kernel, "t+time"]
                                                                    .join("/"))
                                         .unwrap().trim());
                            },
                            Err(_) => {
                                println!(" x");
                                println!("  │   ‗‗Trace‗‗\n  │   {:?}",
                                         fs::read_to_string([kernel, "t+error"]
                                                            .join("/")).unwrap().trim());
                            }
                        };
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Adding all...");
                        io::stdout().flush().unwrap();
                        let addcb = match git.add_all() {
                            Ok (oid)  => {
                                println!(" ✓");
                                oid
                            },
                            Err(err)  => panic!(" x\n\t{:?}", err),
                        };

                        print!  ("  │ ├─ Committing...");
                        io::stdout().flush().unwrap();
                        let _cb_commit = match git.commit("clean build", addcb) {
                            Ok (oid) => {
                                println!(" ✓");
                                oid
                            },
                            Err(err) => panic!(" x\n\t{:?}", err),
                        };

                        let config_branch_ib = [&dir_name, &file_name, "ib"]
                            .join("-");

                        // INCREMENTAL BUILD
                        // -----------------

                        print!  ("  │ ├─ Creating new branch {}...", config_branch_ib);
                        io::stdout().flush().unwrap();
                        git.create_branch(&config_branch_ib, base_cb_commit);
                        println!(" ✓");
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Checkout to {}...", config_branch_ib);
                        io::stdout().flush().unwrap();
                        git.checkout(&config_branch_ib);
                        println!(" ✓");
                        print!  ("  │ ├─ Copying configuration...");
                        io::stdout().flush().unwrap();
                        match fs::copy(&config_path, [kernel, ".config"].join("/")) {
                            Ok (_) => println!(" ✓"),
                            Err(_) => panic!(" x"),
                        };
                        print!  ("  │ ├─ Makefile trace...");
                        io::stdout().flush().unwrap();
                        lmutib::makeni_trace(kernel);
                        println!(" ✓");
                        io::stdout().flush().unwrap();
                        println!("  │ ├─ Total to do: {}",
                                 lmutib::mkf_ni_trace_total(
                                     lmutib::mkf_ni_trace(
                                         Path::new(&[kernel, "t+makeni"].join("/")))));
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Incremental build ({} → {})...",
                                 base_config_branch, config_branch_ib);
                        io::stdout().flush().unwrap();
                        match lmutib::build(kernel) {
                            Ok (_) => {
                                print!(" ✓");
                                println!(" {}s", fs::read_to_string([kernel, "t+time"]
                                                                    .join("/"))
                                         .unwrap().trim());
                            },
                            Err(_) => {
                                println!(" x");
                                println!("  │   ‗‗Trace‗‗\n  │   {:?}",
                                         fs::read_to_string([kernel, "t+error"]
                                                            .join("/")).unwrap().trim());
                            }
                        };
                        io::stdout().flush().unwrap();
                        print!  ("  │ ├─ Adding all...");
                        io::stdout().flush().unwrap();
                        let addib = match git.add_all() {
                            Ok (oid)  => {
                                println!(" ✓");
                                oid
                            },
                            Err(err)  => panic!(" x\n\t{:?}", err),
                        };
                        io::stdout().flush().unwrap();
                        print!  ("  │ └─ Committing...");
                        io::stdout().flush().unwrap();
                        let _ib_commit = match git.commit("incremental build", addib) {
                            Ok (oid) => {
                                println!(" ✓");
                                oid
                            },
                            Err(err) => panic!(" x\n\t{:?}", err),
                        };
                        io::stdout().flush().unwrap();
                    }
                }
            println!("  └───·");
        }
}
