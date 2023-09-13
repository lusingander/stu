# STU

[![Crate Status](https://img.shields.io/crates/v/stu.svg)](https://crates.io/crates/stu)

S3 Terminal UI

## About

STU is the TUI application for AWS S3 written in Rust using [ratatui](https://github.com/ratatui-org/ratatui).

<img src="./img/screenshot.gif">

## Installation

`$ cargo install stu`

## Usage

```
STU - S3 Terminal UI

Usage: stu [OPTIONS]

Options:
  -r, --region <REGION>     AWS region
  -e, --endpoint-url <URL>  AWS endpoint url
  -p, --profile <NAME>      AWS profile name
  -h, --help                Print help
  -V, --version             Print version
```

Detailed operations on each view can be displayed by pressing `?` key.

### Config

Config is loaded from `~/.stu/config.toml`. If the file does not exist, it will be created automatically at startup.

The values that can be set are as follows:

- `download_dir`: _string_ - Directory to save when downloading objects (_default_: `~/.stu/download`)

## License

MIT
