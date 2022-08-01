use std::fs;
use std::io::{self, Write};
use std::path::Path;
mod utils;
mod git;
mod build;
use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, Command};

fn main() {
    let matches = Command::new("Lmutib")
        .version("1.0")
        .about("Experimenting IB")
        .arg(arg!(--src <VALUE>).required(true))
        .arg(arg!(--confs <VALUE>).required(true))
        .arg(arg!(--clean).action(ArgAction::SetTrue))
        .get_matches();

    println!(
        "Linux source: {:?}",
        matches.get_one::<String>("src").expect("required")
    );
    println!(
        "Configurations: {:?}",
        matches.get_one::<String>("confs").expect("required")
    );

    let all_clean = matches.get_one::<bool>("clean").unwrap();

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

                    if (!all_clean) {
			            println!("\t  - Incremental Build...");
			            io::stdout().flush().unwrap();
			            kernel.incremental_build(&[&branch, "cb"].join("-"), &curr);
		            }
	            }
        }

}

// fn main() {

    // let folder  = "/home/linuxtmp";
    // let configs = "/home/configs";
    // println!("Creating Build folder...");
    // let kernel = build::Build::new(folder);
    // println!("Initializing...");
    // kernel.init();
    // for metadir in fs::read_dir(Path::new(configs)).unwrap()
    //     .filter(|f|) f.is_ok()).map(|f| f.unwrap().path()
    //                                 .to_stri().unwrap().to_string()){}

    // let folder = "/tmp/test";
    // let mgit = git::Git::new(&folder);
    // fs::File::create("/tmp/test/master").unwrap();
    // let aoid = mgit.add_all().unwrap();
    // let _ = mgit.commit("source", aoid);
    // let last = mgit.get_last_commit().unwrap();
    // mgit.create_branch("foo", last);
    // mgit.checkout("foo");
    // fs::File::create("/tmp/test/foo-1").unwrap();
    // let oid = mgit.add_all().unwrap();
    // let _ = mgit.commit("foo-1", oid);

    // mgit.checkout("master");

    // let last = mgit.get_last_commit().unwrap();
    // mgit.create_branch("foo", last);
    // mgit.checkout("foo");
    // fs::File::create("/tmp/test/foo-2").unwrap();
    // let oid = mgit.add_all().unwrap();
    // let _ = mgit.commit("foo-2", oid);







    // let basec = [configs, "config"].join("/");
    // let cc = Path::new(&basec);
    // println!("Base configuration build...");
    // kernel.clean_build(&cc);
    // let b = kernel.config_name_from_path(&cc);
    // for conf in fs::read_dir(Path::new(configs)).unwrap()
    //     .filter(|f| f.is_ok()).map(|f| f.unwrap().path()
    //                                .to_str().unwrap().to_string())
    //     .filter(|f| !f.ends_with("config")){
    //         let curr = Path::new(&conf);
    //         println!("{}: Clean Build", conf);
    //         kernel.clean_build(&curr);
    //         println!("{}: Incremental Build", conf);
    //         kernel.incremental_build(&[&b, "cb"].join("-"), &curr);
    //     }
    // println!("-- End, Reporting:");
    // kernel.report()

    // let p = "/home/old/linux-cmut";
    // let mut total = 0;
    // let mut exist = 0;
    // let mut tt = 0;
    // let res =
    //     lmutib::mkf_ni_trace(Path::new("/home/old/linux-cmut/t+makeni"));
    // for (k, v) in &res {
    //     for (k2, v2) in v {
    //         let s = v2.to_string();
    //         let ss = s.replace(";", " ");
    //         let mut splitted = s.split(" ");
    //         for cmd in splitted
    //             .filter(|f| f.ends_with(".o")
    //                     || f.ends_with(".h")
    //                     || f.ends_with(".a")
    //                     || f.ends_with(".cmd")
    //                     || f.ends_with(".c")
    //                     || f.ends_with(".lds")
    //                     || f.ends_with(".s"))
    //             .filter(|f| !f.starts_with("'cmd_"))
    //         {
    //             total = total + 1;
    //             let rp = &[p, cmd].join("/").to_string();
    //             let pp = Path::new(&rp);
    //             if Path::exists(pp) {
    //                 exist = exist + 1;
    //                 tt = tt + fs::metadata(rp).unwrap().len();
    //             }
    //         }
    //     }
    // }
    // println!("{}/{} Total size: {}", exist, total, tt);
    // let res =
    //     lmutib::mkf_ni_trace(Path::new("/home/old/linux-cmut/t+makeni"));
    // for (k, v) in &res {
    //     for (k2, v2) in v {
    //         println!("{} {}", k, k2);
    //         println!("\t{}", v2);
    //     }
    // }

// }
