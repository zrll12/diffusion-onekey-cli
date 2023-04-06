# Diffusion-onekey-cli

A program aiming to install stable diffusion webui for amd in linux in just a few clicks!<br>
Thanks to https://github.com/k7212519/stable-diffusion-webui-AMD-onekey-deploy for providing me the script.

## Warning: This project is currently wok in progress and is not promised to work

## Platform supported

 - [x] Ubuntu 20.04.5
 - [x] Ubuntu 22.04.5
 - [ ] RHEL 7.9
 - [ ] RHEL 8.7
 - [ ] RHEL 9.1
 - [ ] SUSE 15 SP4

Please note that only amd64 CPUs are supported.<br>
More platform support will be added once amd release its driver.

## GPU supported
 - RX 5000 series
 - RX 6600 series
 - RX 6700 series
 - RX 6800 series
 - RX 6900 series
 - RX VEGA

More GPU support will be added once amd release its driver.

## Building

Need rust(cargo) beta version installed<br>
Need libssl-dev installed
```
git clone https://github.com/zrll12/diffusion-onekey-cli.git
cd diffusion-onekey-cli
cargo build --release
```

## Contributing

You can pull the repository, make changes, and then make push requests.<br>
Thank you for contributing to this project!