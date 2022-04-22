use lmutib;
use reqwest;
use std::fs::{File, self};
use std::io::copy;
use std::env;
use std::path::Path;

fn main() {
    env::set_current_dir(Path::new("/home"));
    lmutib::kernel_download("5.13");
    lmutib::extract_tar("linux-5.13.tar.gz", "linux-5.13");
    let kernel = lmutib:: KernelDir::new("linux-5.13");
    let cfdir = |cf: &str| {["configs", cf].join("/")};
    let cfker = |cf: &str| {["linux-5.13", cf].join("/")};
    kernel.create_new_branch(None, "base-cb");
    fs::copy(cfdir("config-base"), cfker(".config"));
    kernel.build();
    kernel.add_all();
    kernel.save("clean build");
    for config in fs::read_dir(Path::new("configs")).unwrap() {
        let fname = config.as_ref()
            .unwrap().file_name().into_string().unwrap();
        if fname.starts_with("___config") {
            // Clean build
            kernel.create_new_branch(
                Some("master"), &[&fname, "cb"].join("-"));
            fs::copy(cfdir(&fname), cfker(".config"));
            kernel.build();
            kernel.add_all();
            kernel.save("clean build");
            // Incremental build
            kernel.create_new_branch(
                Some("base-cb"), &[&fname, "ib"].join("-"));
            fs::copy(cfdir(&fname), cfker(".config"));
            kernel.makeni_trace();
            kernel.build();
            kernel.add_all();
            kernel.save("incremental build");
        }
    }
}
