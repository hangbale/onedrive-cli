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
    pub files: Vec<String>,
    pub parsed_files: Option<Vec<String>>
}

fn is_dir(path: &str) -> bool {
    let path = Path::new(path);
    path.is_dir()
}

fn traverse_dir(path: &str) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    if is_dir(path) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut traverse_dir(path.to_str().unwrap()));
            } else {
                files.push(path.to_str().unwrap().to_string());
            }
        }
    } else {
        files.push(path.to_string());
    }
    files
}

pub fn get_config() -> Config {
    if !Path::new(CONFIG_FILE_PATH).exists() {
        println!("{}", "❌ 配置文件config.yml不存在，请先创建配置文件".red());
        std::process::exit(0);
    } else {
        let config_file = File::open(CONFIG_FILE_PATH).unwrap();
        match serde_yaml::from_reader(config_file) {
            Ok(mut config @ Config { .. }) => {
                let mut files: Vec<String> = Vec::new();
                for file in &config.files {
                    files.append(&mut traverse_dir(&file));
                }
                config.parsed_files = Some(files);
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
