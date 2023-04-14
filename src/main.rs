mod docker;
mod download;
mod desktop;
mod gpu;

use std::{env, fs, io};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Read, Write};
use std::io::ErrorKind::AlreadyExists;
use std::path::{Path};
use std::process::{Command, exit, Stdio};
use bollard::container::StartContainerOptions;
use bollard::Docker;
use crate::desktop::{check_desktop, terminal_prefix};
use crate::Distribution::{Ubuntu2004, Ubuntu2204};
use crate::docker::{check_docker, check_image, install_docker};
use crate::download::install;
use crate::gpu::Args::Unknown;
use crate::gpu::{get_arg_string, get_args, get_args_ask};

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
        install(url, "installer", md5, "amdgpu-install_5.4.50403-1_all.deb");

        println!("Installer ready, installing...");
        
        let (cmd, arg) = terminal_prefix(check_desktop());
        let result = run_command(Command::new(cmd).args(arg.clone()).arg("amdgpu-install").arg("--usecase=rocm,hip,graphics").arg("--opencl=rocr").arg("-y"));
        match result {
            Ok(_) => {}
            Err(err) => {
                println!("It looks like we have met an error. If you need any help, please provide these to admin:\n{}, terminal: {} {:?}", err, cmd, arg);
                panic!("Cannot install driver");
            }
        }

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


    async fn start_container(name: &str) -> Result<(), String> {
        let docker = Docker::connect_with_local_defaults().unwrap();
        let a = docker.start_container(name, None::<StartContainerOptions<String>>).await;
        return match a {
            Ok(_) => {Ok(())}
            Err(err) => {Err(err.to_string())}
        }
    }

    //pull sd
    let result = rt.spawn(check_image());
    if !rt.block_on(result).unwrap() {
        println!("Pulling image...");
        let (cmd, arg) = terminal_prefix(check_desktop());
        Command::new(cmd).args(arg).args(["docker", "run", "-it", "--network=host", "--device=/dev/kfd", "--device=/dev/dri",
            "--group-add=video", "--ipc=host", "--cap-add=SYS_PTRACE", "--security-opt", "seccomp=unconfined", "--name=stable-diffusion", "-v",
            "$HOME/dockerx:/dockerx", "zrll/stable-diffution"]).output().unwrap();//There is a typo with its name in docker hub. Will be fixed in the future.

        let result = rt.spawn(start_container("stable-diffusion"));
        let result = rt.block_on(result).unwrap();
        if let Err(e) = result {
            println!("Cannot load image. If that window just flash through, please check your network");
            panic!("Cannot load image: {}", e.to_string());
        }

        println!("Image ready. Releasing file, this could take a minute.");

        let (cmd, arg) = terminal_prefix(check_desktop());
        Command::new(cmd).args(arg).args(["docker", "exec", "-it", "stable-diffusion",
            "rsync", "-a", "-P", "/sd_backup", "/dockerx"]).output().unwrap();
        println!("Creating shortcut...");
    }


    // let result = rt.spawn(start_container("stable-diffusion"));
    // let result = rt.block_on(result).unwrap();
    // if let Err(_) = result {
    //     println!("Pulling image...");
    //     let (cmd, arg) = terminal_prefix(check_desktop());
    //     Command::new(cmd).args(arg).args(["docker", "run", "-it", "--network=host", "--device=/dev/kfd", "--device=/dev/dri",
    //         "--group-add=video", "--ipc=host", "--cap-add=SYS_PTRACE", "--security-opt", "seccomp=unconfined", "--name=stable-diffusion", "-v",
    //         &(env::var("HOME").expect("Cannot get $HOME").to_string() + "/dockerx:/dockerx"), "zrll/stable-diffution", "&&", "exit"]).output().unwrap();
    //     //docker pull zrll/stable-diffution
    // }


    let result = rt.spawn(start_container("stable-diffusion"));
    let result = rt.block_on(result).unwrap();
    if let Err(e) = result {
        println!("Cannot load image. If that window just flash through, please check your network");
        panic!("Cannot load image: {}", e.to_string());
    }

    let mut launch_arg = get_args();
    if let Unknown = launch_arg {
        launch_arg = get_args_ask();
    }
    let arg_string = get_arg_string(launch_arg);
    let launch_string = "docker exec -it stable-diffusion bash -c \"cd /dockerx/stable-diffusion-webui && source venv/bin/activate && REQS_FILE='requirements.txt' ".to_string() + arg_string + "python launch.py\"";

    let home = env!("HOME");
    let path = home.to_string() + "/dockerx/sh/sd.sh";

    if Path::new(&path).exists() {
        fs::remove_file(&path).expect("Cannot remove file.");
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&path).unwrap();
    file.write_all(launch_string.as_bytes()).unwrap();

    //change owner
    run_command(Command::new("chown").args([env!("USER"), "-R", &(home.to_string() + "/dockerx")])).expect("Cannot change owner");
    run_command(Command::new("chmod").args(["+x", &path])).expect("Cannot change owner");

    match fs::create_dir("/usr/share/stable-diffusion") {
        Ok(_) => {}
        Err(e) => {
            if let AlreadyExists = e.kind() {} else {
                panic!("Cannot create dir: {}", e.to_string());
            }
        }
    };
    fs::copy(home.to_string() + "/dockerx/sh/oneclick_start.sh" , "/usr/share/stable-diffusion/oneclick_start.sh").unwrap();
    fs::copy(home.to_string() + "/dockerx/sh/sd.png" , "/usr/share/icons/sd.png").unwrap();
    fs::copy(home.to_string() + "/dockerx/sh/stable-diffusion.desktop" , home.to_string() + "/.local/share/applications/stable-diffusion.desktop").unwrap();

    println!("Install complete! If you have any problem, please contact staff.");

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
    let mut file = match File::open("/etc/issue.net") {
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
    let mut child = match command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(a) => a,
        Err(err) => { return Err(err.to_string()) }
    };
    let out = child.stdout.take().unwrap();
    let mut out = io::BufReader::new(out);
    let mut s = String::new();
    while let Ok(_) = out.read_line(&mut s) {
        if let Ok(Some(_)) = child.try_wait() { //finished
            break;
        }
        println!("{}", s);
    }
    let out = child.stderr.take().unwrap();
    let mut err_reader = io::BufReader::new(out);
    let mut err = String::new();
    err_reader.read_to_string(&mut err).unwrap();
    return if !err.is_empty() {
        Err(err)
    } else {
        Ok(())
    };
}

