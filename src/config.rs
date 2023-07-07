use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use colored::*;

static CONFIG_FILE_PATH: &str = "config.yml";

#[derive(Serialize, Deserialize, Debug)]
pub struct OnedriveConfig {
    pub appid: String,
    pub secret: String,
    pub token_endpoint: String,
    pub ms_graph_scope: String,
    pub drive: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub onedrive: OnedriveConfig,
    pub files: Vec<String>
}

pub fn get_config() -> Config {
    if !Path::new(CONFIG_FILE_PATH).exists() {
        println!("{}", "❌ 配置文件config.yml不存在，请先创建配置文件".red());
        std::process::exit(0);
    } else {
        let config_file = File::open(CONFIG_FILE_PATH).unwrap();
        match serde_yaml::from_reader(config_file) {
            Ok(config @ Config { .. }) => {
                config
            }
            Err(e) => {
                eprintln!("{}", e);
                println!("{}", "❌ 配置文件config.yml格式错误，请检查配置文件".red());
                std::process::exit(0);
            }
        }
    }
}
