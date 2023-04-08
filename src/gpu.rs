use std::io::stdin;
use std::process::Command;

pub enum Args {
    HsaOverride,
    ROCMArch,
    None,
    Unknown
}

pub fn get_args() -> Args {
    let output = Command::new("lspci").output().unwrap();
    return if let Some(_) = String::from_utf8(output.clone().stdout).unwrap().find("RX 6800") {
        Args::None
    } else if let Some(_) = String::from_utf8(output.clone().stdout).unwrap().find("RX 6700") {
        Args::HsaOverride
    } else if let Some(_) = String::from_utf8(output.clone().stdout).unwrap().find("RX 6600") {
        Args::HsaOverride
    } else if let Some(_) = String::from_utf8(output.stdout).unwrap().find("RX 5") {
        Args::HsaOverride
    } else {
        Args::Unknown
    }
}

pub fn get_args_ask() -> Args {
    let mut buffer = "".to_string();
    println!("Please choose your GPU\n1) RX 6800 series/RX 6900 series\n2) RX 6600 series/RX 6700 series/RX 5000 series\n3) RX Vega series\nInput your answer and press enter.");

    stdin().read_line(&mut buffer).expect("Cannot read from stdin...");
    let input;

    if buffer.chars().nth(0).is_none() {
        input = "1".to_string();
    } else {
        input = buffer.chars().nth(0).unwrap().to_string().trim_end().to_string();
    }
    buffer.clear();

    return match input.as_str() {
        "2" => Args::HsaOverride,
        "3" => Args::ROCMArch,
        _ => Args::None,
    };
}

pub fn get_arg_string(arg: Args) -> &'static str {
    return match arg {
        Args::HsaOverride => {
            "HSA_OVERRIDE_GFX_VERSION=10.3.0"
        },
        Args::ROCMArch => {
            "PYTORCH_ROCM_ARCH=gfx906 HCC_AMDGPU_TARGET=gfx906"
        },
        Args::None => {
            ""
        },
        Args::Unknown => {
            "HSA_OVERRIDE_GFX_VERSION=10.3.0"
        }
    }
}