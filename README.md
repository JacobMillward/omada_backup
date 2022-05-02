[![GitHub](https://img.shields.io/github/license/jacobmillward/omada_backup?label=License)](https://github.com/JacobMillward/omada_backup/blob/main/LICENSE)
[![Build](https://github.com/JacobMillward/omada_backup/actions/workflows/build.yml/badge.svg)](https://github.com/JacobMillward/omada_backup/actions/workflows/build.yml) 
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/jacobmillward/omada_backup?label=Release)](https://github.com/JacobMillward/omada_backup/releases/latest)

# Omada Backup

A utility for downloading backups of a TP-Link Omada SDN Controller.

It logs into a given controller, and pulls down a backup file. The retention options are the same as given on the controllers maintenance page.

Tested and confirmed working with Omada Controller v`5.0.30`

## Usage
### Example
```sh
$ omada_backup -u admin -p mypassword -b https://10.0.0.100 -t
```
```
USAGE:
    omada_backup [OPTIONS] --username <USERNAME> --password <PASSWORD> --base-url <BASE_URL>

OPTIONS:
    -b, --base-url <BASE_URL>          Base URL for the Omada SDN Controller
    -h, --help                         Print help information
    -o, --output-file <OUTPUT_FILE>    Write to file instead of current directory
    -p, --password <PASSWORD>          Password for the User
    -q, --quiet                        Less output per occurrence
    -r, --retention <RETENTION>        Data retention period for the backup [default: settings-only]
                                       [possible values: settings-only, days7, days30, days60,
                                       days90, days180]
    -t, --trust-all-certificates       Enables trusting of invalid HTTPS certificates, including
                                       self-signed certificates
    -u, --username <USERNAME>          User to login to the Omada Controller
    -v, --verbose                      More output per occurrence
    -V, --version                      Print version information                 Print version information
```

## Building

Building this project requires the rust toolchain, which can be installed via [`rustup`](https://rustup.rs/). It can then be built with [`cargo`](https://doc.rust-lang.org/cargo/).

_N.B. On Windows the `msvc` toolchain is required, as it will not build under the `gnu` toolchain e.g. `stable-x86_64-pc-windows-msvc`_

```sh
$ cargo build
```

