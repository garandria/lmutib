use lmutib;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
mod git;
mod build;
use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, Command};

fn main() {
    let matches = Command::new("Lmutib")
        .version("1.0")
        .about("Experimenting IB")
        .arg(arg!(--src <VALUE>))
        .arg(arg!(--confs <VALUE>))
        .get_matches();

    println!(
        "Linux source: {:?}",
        matches.get_one::<String>("src").expect("required")
    );
    println!(
        "Configurations: {:?}",
        matches.get_one::<String>("confs").expect("required")
    );


    let source = matches.get_one::<String>("src").expect("required");
    let configs  = matches.get_one::<String>("confs").expect("required");
    println!("Creating Build folder...");
    io::stdout().flush().unwrap();
    let kernel = build::Build::new(source);
    println!("Initializing...");
    io::stdout().flush().unwrap();
    kernel.init();
    for metadir in fs::read_dir(Path::new(configs)).unwrap()
        .filter(|f| f.is_ok())
        .map(|f| f.unwrap().path().to_str().unwrap().to_string()) {
		    println!("-> {}", metadir);
		    io::stdout().flush().unwrap();
		    let mut basename = String::new();

		    for conf in fs::read_dir(Path::new(&metadir)).unwrap()
		        .filter(|f| f.is_ok()).map(|f| f.unwrap().path().to_str()
                                           .unwrap().to_string())
		        .filter(|f| f.ends_with("_base")) {
		 	        basename = conf.to_string();
		        }
		    let baseconf = [configs.to_string(), basename.to_string()]
                .join("/");
		    let baseconf_path = Path::new(&basename);
		    println!("\t~ Base: {}", basename);
		    io::stdout().flush().unwrap();
		    println!("\t  Clean build...");
		    io::stdout().flush().unwrap();
		    kernel.clean_build(&baseconf_path);
		    let branch = kernel.config_name_from_path(&baseconf_path);
		    for conf in fs::read_dir(Path::new(&metadir)).unwrap()
		        .filter(|f| f.is_ok()).map(|f| f.unwrap().path().to_str()
                                           .unwrap().to_string())
		        .filter(|f| !f.ends_with("_base")){
		 	        println!("\t* {}", conf);
			        io::stdout().flush().unwrap();
			        let curr = Path::new(&conf);
			        println!("\t  - Clean Build...");
			        io::stdout().flush().unwrap();
			        kernel.clean_build(&curr);
			        println!("\t  - Incremental Build...");
			        io::stdout().flush().unwrap();
			        kernel.incremental_build(&[&branch, "cb"].join("-"), &curr);
		        }
	    }

}
