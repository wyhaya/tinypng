
# tinypng [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/wyhaya/tinypng/Build?style=flat-square)](https://github.com/wyhaya/tinypng/actions) [![Crates.io](https://img.shields.io/crates/v/tinypng.svg?style=flat-square)](https://crates.io/crates/tinypng)

Command line tool for compressing images using the TinyPNG API

## Install

[Download](https://github.com/wyhaya/tinypng/releases) the binary from the release page

Or use `cargo` to install

```bash
cargo install tinypng
```

## Usage

1. Register a KEY using your email at [link](https://tinypng.com/developers)

2. Set TinyPNG API KEY

```sh
tinypng -k <KEY>
# Set API KEY successfully
# Your key is stored in ~/.config/tinypng/config.toml
```

3. Compress images

```sh
tinypng ./test.png
# test.png: Origin: 1004.7 KB Compressed: 245.4 KB(75.6%)

# Glob
tinypng ./images/*.png
# images/test.png: Origin: 1.4 MB Compressed: 174.5 KB(87.8%)
```
