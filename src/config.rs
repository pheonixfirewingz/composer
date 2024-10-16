use std::fs;
use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config{
    pub site_root: String,
    pub music_dir: String,
    pub web_assets_dir: String,
    pub templates_dir: String
}

pub struct UploadConfig{
    pub web_assets_dir: String,
    pub music_dir: String
}

impl UploadConfig {
    // Constructor for UploadConfig
    pub fn new(web_assets_dir: String, music_dir: String) -> Self {
        UploadConfig {
            web_assets_dir,
            music_dir,
        }
    }
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