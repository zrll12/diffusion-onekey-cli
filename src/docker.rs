use std::env;
use std::process::Command;
use bollard::Docker;
use bollard::image::ListImagesOptions;
use crate::{Distribution, run_command};
use crate::download::install;

pub async fn check_docker() -> bool {
    run_command(Command::new("docker").arg("images")).is_ok()
}

pub async fn check_image() -> bool {
    let mut flag = false;
    let docker = Docker::connect_with_local_defaults().unwrap();
    let version = docker.version().await.unwrap();
    println!("{}", version.version.unwrap());


    let images = &docker.list_images(Some(ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    })).await.unwrap();

    for image in images {
        if image.repo_tags[0] == "zrll/stable-diffution:latest" {//There is a typo with its name in docker hub. Will be fixed in the future.
            flag = true;
        }
    }

    flag
}

pub fn install_docker(distribution: &Distribution) {
    let url = match distribution {
        Distribution::Ubuntu2004 => { "https://download.docker.com/linux/ubuntu/dists/focal/pool/stable/amd64/containerd.io_1.6.20-1_amd64.deb" }
        Distribution::Ubuntu2204 => { "https://download.docker.com/linux/ubuntu/dists/jammy/pool/stable/amd64/containerd.io_1.6.20-1_amd64.deb" }
    };
    let md5 = match distribution {
        Distribution::Ubuntu2004 => { "99716cf5d655a35badbf5c578afbabfd" }
        Distribution::Ubuntu2204 => { "e415b88f5f0d926a9192e804f7d3b693" }
    };
    install(url,"containerd", md5, "containerd.io_1.6.20-1_amd64.deb");


    let url = match distribution {
        Distribution::Ubuntu2004 => { "https://download.docker.com/linux/ubuntu/dists/focal/pool/stable/amd64/docker-ce-cli_23.0.3-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "https://download.docker.com/linux/ubuntu/dists/jammy/pool/stable/amd64/docker-ce-cli_23.0.3-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    let md5 = match distribution {
        Distribution::Ubuntu2004 => { "e5a0c3de84a458a9d4fb711e7b00fca2" }
        Distribution::Ubuntu2204 => { "a913680a6ecfcffc25ee4b877fbfccab" }
    };
    let file_name = match distribution {
        Distribution::Ubuntu2004 => { "docker-ce-cli_23.0.3-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "docker-ce-cli_23.0.3-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    install(url, "docker-ce-cli", md5, file_name);


    let url = match distribution {
        Distribution::Ubuntu2004 => { "https://download.docker.com/linux/ubuntu/dists/focal/pool/stable/amd64/docker-ce_23.0.3-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "https://download.docker.com/linux/ubuntu/dists/jammy/pool/stable/amd64/docker-ce_23.0.3-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    let md5 = match distribution {
        Distribution::Ubuntu2004 => { "77e525d158f15e92bd2192bb0462e2cb" }
        Distribution::Ubuntu2204 => { "9e171fc3faa2c7a881a5831406d4ad80" }
    };
    let file_name = match distribution {
        Distribution::Ubuntu2004 => { "docker-ce_23.0.3-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "docker-ce_23.0.3-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    install(url, "docker-ce", md5, file_name);


    let url = match distribution {
        Distribution::Ubuntu2004 => { "https://download.docker.com/linux/ubuntu/dists/focal/pool/stable/amd64/docker-buildx-plugin_0.10.4-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "https://download.docker.com/linux/ubuntu/dists/jammy/pool/stable/amd64/docker-buildx-plugin_0.10.4-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    let md5 = match distribution {
        Distribution::Ubuntu2004 => { "f908e8c6a86033722f15b56041b35390" }
        Distribution::Ubuntu2204 => { "8302dcb5a863e0b1c40fbc86c0a23547" }
    };
    let file_name = match distribution {
        Distribution::Ubuntu2004 => { "docker-buildx-plugin_0.10.4-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "docker-buildx-plugin_0.10.4-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    install(url, "buildx-plugin", md5, file_name);


    let url = match distribution {
        Distribution::Ubuntu2004 => { "https://download.docker.com/linux/ubuntu/dists/focal/pool/stable/amd64/docker-compose-plugin_2.17.2-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "https://download.docker.com/linux/ubuntu/dists/jammy/pool/stable/amd64/docker-compose-plugin_2.17.2-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    let md5 = match distribution {
        Distribution::Ubuntu2004 => { "60abca4f505b3a91962935fa8fb39de9" }
        Distribution::Ubuntu2204 => { "5747b120df5b0c1d3c500a12bc81d1c6" }
    };
    let file_name = match distribution {
        Distribution::Ubuntu2004 => { "docker-compose-plugin_2.17.2-1~ubuntu.20.04~focal_amd64.deb" }
        Distribution::Ubuntu2204 => { "docker-compose-plugin_2.17.2-1~ubuntu.22.04~jammy_amd64.deb" }
    };
    install(url, "compose-plugin", md5, file_name);

    //add to user group
    run_command(Command::new("groupadd").arg("docker").arg("-f")).expect("Cannot run groupadd");
    run_command(Command::new("gpasswd").arg("-a").arg(env::var("USER").expect("Cannot get $USER")).arg("docker")).expect("Cannot run gpasswd");
    run_command(Command::new("newgrp").arg("docker")).expect("Cannot run newgrp.");
}