mod utils;
use clap::{Arg, Command};
use futures_util::future::join_all;
use glob::{glob, GlobError};
use home_config::HomeConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tinypng::{TinyPng, REGISTER_URL};
use utils::format_size;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    key: String,
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

    let hc = HomeConfig::with_config_dir(env!("CARGO_PKG_NAME"), "config.toml");
    let mut config = hc.toml::<Config>().unwrap_or_default();

    // Set API KEY
    if let Some(key) = app.value_of("key") {
        if key.len() != 32 {
            exit!("Invalid API KEY");
        }
        config.key = key.to_string();
        hc.save_toml(&config).unwrap_or_else(|err| {
            exit!("{:#?}", err);
        });
        println!("Set API KEY successfully");
        return;
    }

    if config.key.len() != 32 {
        exit!("Invalid API KEY\n1. Register a KEY using your email at {REGISTER_URL}\n2. Use 'tinypng -k <KEY>' to set API_KEY");
    }

    let tiny = Arc::new(TinyPng::new(config.key));

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
