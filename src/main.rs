use std::{env, fs, io, result};
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::process::{Command, exit, Stdio};
use std::time::Duration;
use downloader::Downloader;
use tokio::main;
use crate::Distribution::{UBUNTU_2004, UBUNTU_2204};

#[derive(Debug)]
enum Distribution {
    UBUNTU_2004,
    UBUNTU_2204,
}

fn main() {
    sudo::with_env(&["HOME"]).expect("sudo failed");
    println!("Collecting system info...");
    //check if version compatible
    let linux_version = match check_distribute_version() {
        Ok(info) => { info }
        Err(err) => {
            println!("Cannot examine your linux linux_version, are you using linux?");
            panic!("{}", err);
        }
    };
    let distribution = match linux_version[0..2].to_string().as_str() {
        "Ub" => {
            match linux_version[7..14].to_string().as_str() {
                "22.04.2" => {
                    UBUNTU_2204
                }
                "20.04.5" => {
                    UBUNTU_2004
                }
                a => {
                    panic!("Unknown or unsupported distribution version: {}", a);
                }
            }
        }
        a => {
            panic!("Unknown distribution: {}", a);
        }
    };
    println!("Your distribution is: {:?}", distribution);

    //check is driver installed
    if let Err(e) = Command::new("rocm-smi").output() {
        //download
        if !Path::new(get_file_name(&distribution).as_str()).exists() {
            println!("It seems you have not install the drivers yet, downloading...");
            println!("If you believe this is a mistake, please provide these to admin:\n{}", e.to_string());
            download_driver(&distribution, 0);
        }
        //examine md5
        //calculate md5 for file
        let mut f = File::open(get_file_name(&distribution)).unwrap();
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer).unwrap();

        let mut digest = md5::compute(buffer);
        let mut times = 0;
        while format!("{:x}", digest) != "493c00c81e8dc166b3abde1bf1d04cda" { // file error, try again
            fs::remove_file(get_file_name(&distribution)).expect("Cannot remove file.");
            times = download_driver(&distribution, times);
        }
        println!("Download complete, installing...");
        // println!("If you believe this is a mistake, please provide these to admin:\n{}", e.to_string());

        //setup
        println!("Installing the installer...");
        let result = run_command(Command::new("dpkg").arg("-i").arg(get_file_name(&distribution)));
        match result {
            Ok(_) => {}
            Err(err) => {
                println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}", err);
                panic!("Cannot install driver");
            }
        }

        println!("Installer ready, installing...");
        let result = run_command(Command::new("amdgpu-install").arg("--usecase=rocm,hip,graphics").arg("--opencl=rocr"));
        match result {
            Ok(_) => {}
            Err(err) => {
                println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}", err);
                panic!("Cannot install driver");
            }
        }

        println!("Driver installed, cleaning up...");
        fs::remove_file(get_file_name(&distribution)).expect("Cannot remove file.");

        println!("Please reboot to continue installing.");
        pause();
        exit(0);
    };

    //check is driver installed
    println!("Driver checked, preparing docker now...");
    if let Err(e) = Command::new("docker").output() {
        //install docker
        // let architecture = String::from_utf8(Command::new("dpkg").arg("--print-architecture").output().unwrap().stdout).unwrap();
        if Path::new("docker.sh").exists() {
            println!("shell file exists, deleting...");
            fs::remove_file("docker.sh").expect("Cannot remove file.");
        }

        //TODO: change to rust code
        let shell_script = r#"#!/bin/bash

usermod -a -G render $LOGNAME
usermod -a -G video $LOGNAME

apt-get update

# Update the apt package index and install packages to allow apt to use a repository over HTTPS:
apt-get install \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Add Docker’s official GPG key:
mkdir -m 0755 -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# Use the following command to set up the repository:
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt-get update

#Your default umask may be incorrectly configured, preventing detection of the repository public key file. Try granting read permission for the Docker public key file before updating the package index:
chmod a+r /etc/apt/keyrings/docker.gpg
apt-get update

# To install the latest version, run:
apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
docker version
echo "docker installed"

#add user/docker to group
groupadd docker
echo "user add groupaad"
gpasswd -a $USER docker
echo "gpasswd"
newgrp docker"#;
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("docker.sh").unwrap();
        file.write_all(shell_script.as_bytes()).expect("Cannot write file.");

        let result = run_command(Command::new("sh").arg("docker.sh"));
        match result {
            Ok(_) => {}
            Err(err) => {
                println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}", err);
                panic!("Cannot install docker");
            }
        }

        println!("Docker installed, reboot to continue.");
        pause();
        exit(0);
    }

    //pull sd

    if Path::new("sd.sh").exists() {
        println!("shell file exists, deleting...");
        fs::remove_file("docker.sh").expect("Cannot remove file.");
    }

    //TODO: change to rust code
    let sd_script = r#"#!/bin/bash


rocm-smi

while true; do
  read -p "请确认在上方能看到您的显卡信息，y确认，N退出？[y/n] " yn
  case $yn in
    [Yy]* ) break;;
    [Nn]* ) exit;;
    * ) echo "请输入 y 或 n.";;
  esac
done

#Pull the latest k7212519/stable-diffusion-webui Docker image, start the image and attach to the container
gnome-terminal -- docker run -it --network=host --device=/dev/kfd --device=/dev/dri --group-add=video --ipc=host --cap-add=SYS_PTRACE --security-opt seccomp=unconfined --name=stable-diffusion -v $HOME/dockerx:/dockerx k7212519/stable-diffusion-webui

# 等待容器启动
until [ "$(docker inspect -f '{{.State.Status}}' stable-diffusion)" = "running" ]; do
    sleep 10
done

echo "docker is running..."

echo "正在释放文件，请稍等......"
# copy sd files from /sd_backup to /dockerx/
docker exec -it stable-diffusion bash -c "cp -a /sd_backup/. /dockerx/ && exit"


##########显卡型号选择##########

valid_choice=false

while [ "$valid_choice" == false ]
do
  echo "请选择您的显卡型号："
  echo "1. RX 6800系列"
  echo "2. RX 6700系列"
  echo "3. RX 6600系列"
  echo "4. RX 5000系列"
  echo "5. RX Vega系列"

  read -p "请输入选项编号： " choice

  case $choice in
    1)
      echo "您选择了 RX 6800"
      sudo cp -f GPU/rx6800.sh  $HOME/dockerx/sh/sd.sh
      valid_choice=true
      ;;
    2)
      echo "您选择了 RX 6700"
      sudo cp -f GPU/rx6700.sh  $HOME/dockerx/sh/sd.sh
      valid_choice=true
      ;;
    3)
      echo "您选择了 RX 6600"
      sudo cp -f GPU/rx6600.sh  $HOME/dockerx/sh/sd.sh
      valid_choice=true
      ;;
    4)
      echo "您选择了 RX 5000"
      sudo cp -f GPU/rx5000.sh  $HOME/dockerx/sh/sd.sh
      valid_choice=true
      ;;
     5)
      echo "您选择了 RX Vega"
      sudo cp -f GPU/rx_vega.sh  $HOME/dockerx/sh/sd.sh
      valid_choice=true
      ;;
    *)
      echo "无效的选项编号，请重新输入。"
      ;;
  esac
done
# 创建目录和拷贝文件
sudo mkdir /usr/share/stable-diffusion
sudo cp $HOME/dockerx/sh/oneclick_start.sh /usr/share/stable-diffusion/
sudo cp $HOME/dockerx/sh/sd.png /usr/share/icons/
sudo cp $HOME/dockerx/sh/stable-diffusion.desktop $HOME/.local/share/applications/
sudo chmod -R 777 $HOME/dockerx"#;
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("sd.sh").unwrap();
    file.write_all(sd_script.as_bytes()).expect("Cannot write file.");

    let result = run_command(Command::new("sh").arg("sd.sh"));
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}", err);
            panic!("Cannot install docker");
        }
    }

    println!("Docker installed, reboot to continue.");
    pause();
    exit(0);
}

fn pause() {
    println!("Press any key to continue...");
    let stdin = io::stdin();
    let mut _buffer = "".to_string();
    stdin.read_line(&mut _buffer).expect("Cannot read from stdin...");
}

fn check_distribute_version() -> Result<String, String> {
    let mut system: String = Default::default();
    let mut file = match std::fs::File::open("/etc/issue.net") {
        Ok(f) => { f }
        Err(e) => { return Err(e.to_string()); }
    };
    match file.read_to_string(&mut system) {
        Ok(_) => {}
        Err(e) => { return Err(e.to_string()); }
    };
    Ok(system)
}

fn get_file_name(distribution: &Distribution) -> String {
    match distribution {
        UBUNTU_2004 => { "amdgpu-install_5.4.50403-1_all.deb".to_string() }
        UBUNTU_2204 => { "amdgpu-install_5.4.50403-1_all.deb".to_string() }
    }
}

fn download_driver(distribution: &Distribution, times: i32) -> i32 {
    let url = match distribution {
        UBUNTU_2004 => { "https://repo.radeon.com/amdgpu-install/22.40.3/ubuntu/focal/amdgpu-install_5.4.50403-1_all.deb" }
        UBUNTU_2204 => { "https://repo.radeon.com/amdgpu-install/22.40.3/ubuntu/jammy/amdgpu-install_5.4.50403-1_all.deb" }
    };

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

fn run_command(command: &mut Command) -> Result<(), String> {
    let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let out = child.stdout.take().unwrap();
    let mut out = std::io::BufReader::new(out);
    let mut s = String::new();
    while let Ok(_) = out.read_line(&mut s) {
        if let Ok(Some(_)) = child.try_wait() { //finished
            break;
        }
        println!("{}", s);
    }
    let out = child.stderr.take().unwrap();
    let mut err_reader = std::io::BufReader::new(out);
    let mut err = String::new();
    err_reader.read_to_string(&mut err).unwrap();
    return if !err.is_empty() {
        Err(err)
    } else {
        Ok(())
    }
}


// Define a custom progress reporter:
struct SimpleReporterPrivate {
    last_update: std::time::Instant,
    max_progress: Option<u64>,
    message: String,
}
struct SimpleReporter {
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