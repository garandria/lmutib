// use lmutib;
use std::fs;
// use std::io::{self, Write};
use std::path::Path;
mod git;
mod build;

fn main() {

    let folder  = "/home/linux-5.13";
    let configs = "/home/configs";
    println!("Creating Build folder...");
    let kernel = build::Build::new(folder);
    println!("Initializing...");
    kernel.init();
    let basec = [configs, "config"].join("/");
    let cc = Path::new(&basec);
    println!("Base configuration build...");
    kernel.clean_build(&cc);
    let b = kernel.config_name_from_path(&cc);
    for conf in fs::read_dir(Path::new(configs)).unwrap()
        .filter(|f| f.is_ok()).map(|f| f.unwrap().path()
                                   .to_str().unwrap().to_string())
        .filter(|f| !f.ends_with("config")){
            let curr = Path::new(&conf);
            println!("{}: Clean Build", conf);
            kernel.clean_build(&curr);
            println!("{}: Incremental Build", conf);
            kernel.incremental_build(&b, &curr);
        }
    println!("-- End, Reporting:");
    kernel.report()
}
