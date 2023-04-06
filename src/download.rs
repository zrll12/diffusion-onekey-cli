use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use downloader::Downloader;
use crate::{run_command, SimpleReporter};

pub fn install(url: &str, describe: &str, md5: &str, file_name: &str) {
    //download
    if !Path::new(file_name).exists() {
        println!("Downloading {} ...", describe);
        download_file(url, 0);
    }

    //examine md5
    //calculate md5 for file
    let mut f = File::open(file_name).unwrap();
    let mut buffer = Vec::new();
    // read the whole file
    f.read_to_end(&mut buffer).unwrap();

    let digest = md5::compute(buffer);
    let mut times = 0;
    while format!("{:x}", digest) != md5 { // file error, try again
        fs::remove_file(file_name).expect("Cannot remove file.");
        times = download_file(url, times);
    }

    //setup
    println!("Installing {} ...", describe);
    let result = run_command(Command::new("dpkg").arg("-i").arg(file_name));
    println!("Cleaning {} ...", describe);
    fs::remove_file(file_name).expect("Cannot remove file.");
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}", err);
            panic!("Cannot install {}", describe);
        }
    }
}

pub fn install_no_check(url: &str, describe: &str, file_name: &str) {
    //download
    if Path::new(url).exists() {
        fs::remove_file(file_name).expect("Cannot remove file.");
    }
    println!("Downloading {} ...", describe);
    download_file(url, 0);
    //setup
    println!("Installing {} ...", describe);
    let result = run_command(Command::new("dpkg").arg("-i").arg(file_name));
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}", err);
            panic!("Cannot install {}", describe);
        }
    }
}

fn download_file(url: &str, times: i32) -> i32 {
    if times >= 3 {//too many times
        println!("We have tried for at least 3 times, but unable to download the right file. Please download with this link and copy them to this directory: {}", url);
        panic!("Downloaded file error");
    }

    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("./"))
        .parallel_requests(1)
        .build()
        .unwrap();

    let dl = downloader::Download::new(url);
    let dl = dl.progress(SimpleReporter::create());

    let result = downloader.download(&[dl]).unwrap();
    match result.get(0).unwrap() {
        Err(e) => {
            println!("Download failed. If you need any help, please provide these to admin:\n{}", e.to_string());
            panic!("Download failed.");
        },
        Ok(_) => {},
    };

    times + 1
}
