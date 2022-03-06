mod lib;
mod utils;
use clap::{Arg, Command};
use futures_util::future::join_all;
use glob::{glob, GlobError};
use home_config::HomeConfig;
use lib::TinyPng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use utils::format_size;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    key: String,
}

#[macro_export]
macro_rules! exit {
    ($($arg:tt)*) => {
       {
            eprint!("Error: ");
            eprintln!($($arg)*);
            std::process::exit(1)
       }
    };
}

#[tokio::main]
async fn main() {
    let app = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("key")
                .short('k')
                .takes_value(true)
                .value_name("API_KEY")
                .help("Set TinyPNG API KEY"),
        )
        .arg(
            Arg::new("image")
                .conflicts_with("key")
                .required(true)
                .min_values(1)
                .help("Images to be compressed"),
        )
        .get_matches();

    let config = HomeConfig::new(env!("CARGO_PKG_NAME"), "config.json");
    let mut c = config.parse::<Config>().unwrap_or_default();

    if app.is_present("key") {
        let key = app.value_of("key").unwrap();
        if key.len() != 32 {
            exit!("Invalid API KEY");
        }
        c.key = key.to_string();
        config.save(&c).unwrap();
        println!("Set API KEY successfully");
        return;
    }

    if c.key.len() != 32 {
        exit!("Please use 'tinypng -k <KEY>' to set TinyPNG API_KEY\nLink: https://tinypng.com/developers");
    }

    let tiny = Arc::new(TinyPng::new(c.key));

    let paths = app
        .values_of("image")
        .unwrap()
        .flat_map(|val| {
            glob(val).unwrap_or_else(|err| {
                exit!("{:#?}", err);
            })
        })
        .collect::<Result<Vec<PathBuf>, GlobError>>()
        .unwrap_or_else(|err| {
            exit!("{:#?}", err);
        });

    let mut fus = Vec::with_capacity(paths.len());

    for p in paths {
        let tiny = tiny.clone();
        let f = async move {
            let rst = tiny.compress_file(&p, &p).await;
            match rst {
                Ok((input, output)) => {
                    let ratio = (1.0 - (output as f32 / input as f32)) * 100.0;
                    let (input, output) = (format_size(input), format_size(output));
                    println!(
                        "{}: Origin: {} Compressed: {}({:.1}%)",
                        p.display(),
                        input,
                        output,
                        ratio
                    );
                }
                Err(e) => {
                    eprintln!("{}: {:?}", p.display(), e);
                }
            };
        };
        fus.push(f);
    }

    join_all(fus).await;
}
