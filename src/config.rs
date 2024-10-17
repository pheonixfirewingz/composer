use std::fs;
use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config{
    pub root_dir: String,
    pub music_dir: String,
}

#[derive(Parser)]
struct Args {
    config: String,
}

fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}

pub fn setup_config() -> Option<Config> {
    let args = Args::parse();
    match read_config(&args.config) {
        Ok(config) => {
            println!("Configuration loaded: {:?}", config);
            Option::from(config)
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            None
        }
    }
}