mod docker;
mod download;

use std::{fs, io};
use std::io::{BufRead, Read, Write};
use std::path::{Path};
use std::process::{Command, exit, Stdio};
use crate::Distribution::{Ubuntu2004, Ubuntu2204};
use crate::docker::{check_docker, install_docker};
use crate::download::install;

#[derive(Debug)]
pub enum Distribution {
    Ubuntu2004,
    Ubuntu2204,
}

fn main() {
    sudo::with_env(&["HOME", "USER"]).expect("sudo failed");
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
                    Ubuntu2204
                }
                "20.04.5" => {
                    Ubuntu2004
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
    if let Err(_) = Command::new("rocm-smi").output() {
        let url = match distribution {
            Ubuntu2004 => { "https://repo.radeon.com/amdgpu-install/22.40.3/ubuntu/focal/amdgpu-install_5.4.50403-1_all.deb" }
            Ubuntu2204 => { "https://repo.radeon.com/amdgpu-install/22.40.3/ubuntu/jammy/amdgpu-install_5.4.50403-1_all.deb" }
        };
        let md5 = match distribution {
            Ubuntu2004 => { "9f59f90b8e9cdd502892b1d052e909b1" }
            Ubuntu2204 => { "493c00c81e8dc166b3abde1bf1d04cda" }
        };
        install(url,"installer", md5, "amdgpu-install_5.4.50403-1_all.deb");

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
        fs::remove_file("amdgpu-install_5.4.50403-1_all.deb").expect("Cannot remove file.");

        println!("Please reboot to continue installing.");
        pause();
        exit(0);
    };

    //check is docker installed
    println!("Driver checked, preparing docker now...");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.spawn(check_docker());
    if !rt.block_on(result).unwrap() {
        //install docker
        install_docker(&distribution);
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

pub fn run_command(command: &mut Command) -> Result<(), String> {
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