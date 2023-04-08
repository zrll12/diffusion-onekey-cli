use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use downloader::Downloader;
use crate::{run_command};

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

#[deprecated(
note = "Please use install instead."
)]
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
        .download_folder(Path::new("./"))
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


// Define a custom progress reporter:
pub struct SimpleReporterPrivate {
    last_update: std::time::Instant,
    max_progress: Option<u64>,
    message: String,
}

pub struct SimpleReporter {
    private: std::sync::Mutex<Option<SimpleReporterPrivate>>,
}

impl SimpleReporter {
    #[cfg(not(feature = "tui"))]
    fn create() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            private: std::sync::Mutex::new(None),
        })
    }
}

impl downloader::progress::Reporter for SimpleReporter {
    fn setup(&self, max_progress: Option<u64>, message: &str) {
        let private = SimpleReporterPrivate {
            last_update: std::time::Instant::now(),
            max_progress,
            message: message.to_owned(),
        };

        let mut guard = self.private.lock().unwrap();
        *guard = Some(private);
    }

    fn progress(&self, current: u64) {
        if let Some(p) = self.private.lock().unwrap().as_mut() {
            let max_bytes = match p.max_progress {
                Some(bytes) => format!("{:?}", bytes),
                None => "{unknown}".to_owned(),
            };
            if p.last_update.elapsed().as_millis() >= 1000 {
                println!(
                    "tdownloader: {} of {} bytes. [{}]",
                    current, max_bytes, p.message
                );
                p.last_update = std::time::Instant::now();
            }
        }
    }

    fn set_message(&self, message: &str) {
        println!("downloader: Message changed to: {}", message);
    }

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        *guard = None;
        println!("downloader: [DONE]");
    }
}