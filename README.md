# STU

[![Crate Status](https://img.shields.io/crates/v/stu.svg)](https://crates.io/crates/stu)

S3 Terminal UI

## About

STU is the TUI application for AWS S3 written in Rust using [ratatui](https://github.com/ratatui-org/ratatui).

<img src="./img/demo.gif">

## Installation

### Cargo

```
$ cargo install stu
```

### Homebrew (macOS)

```
$ brew install lusingander/tap/stu
```

### AUR (Arch Linux)

```
$ paru -S stu
```

### Binary

You can download binaries from [releases](https://github.com/lusingander/stu/releases)

## Usage

```
STU - S3 Terminal UI

Usage: stu [OPTIONS]

Options:
  -r, --region <REGION>     AWS region
  -e, --endpoint-url <URL>  AWS endpoint url
  -p, --profile <NAME>      AWS profile name
  -b, --bucket <NAME>       Target bucket name
  -h, --help                Print help
  -V, --version             Print version
```

You can also use each environment variable in the same way as [when using the AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html).

Detailed operations on each view can be displayed by pressing `?` key.

Or refer to the `***-help.png` screenshots in the [./img directory](./img).

### Config

Config is loaded from `$STU_ROOT_DIR/config.toml`.

- If `STU_ROOT_DIR` environment variable is not set, `~/.stu` is used by default.
- If the file does not exist, it will be created automatically at startup.

The values that can be set are as follows:

- `download_dir`: _string_ - Directory to save when downloading objects (_default_: `$STU_ROOT_DIR/download`)

## Features / Screenshots

### Bucket list

- Show list of buckets

<img src="./img/bucket-list.png" width=500>

### Object list

- Show list of objects in a hierarchy

<img src="./img/object-list-simple.png" width=500>
<img src="./img/object-list-hierarchy.png" width=500>
<img src="./img/object-list-many.png" width=500>

### Object detail

- Show object details
- Download object
- Preview object (text file only)
- Copy resource name to clipboard

<img src="./img/object-detail.png" width=500>
<img src="./img/object-version.png" width=500>
<img src="./img/object-download.png" width=500>
<img src="./img/object-preview.png" width=500>
<img src="./img/object-details-copy.png" width=500>

## License

MIT
