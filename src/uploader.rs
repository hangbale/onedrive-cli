use std::path::Path;
use serde_json::json;
use colored::*;
use crate::config::Config;

use indicatif::ProgressBar;
use std::io::{ Read };
use std::fs::File;
use reqwest::header::{
    HeaderMap,
    HeaderValue,
    CONTENT_LENGTH,
    CONTENT_RANGE,
};
use reqwest::StatusCode;
use crate::request::{Request, B};

static SLICE_SIZE: u32 = 5 * 1024 * 1024;
static MSAPI: &str = "https://graph.microsoft.com/v1.0/";

pub struct Uploader<'a> {
    client: Request<'a>,
    config: &'a Config,
}

impl<'a> Uploader<'a> {
    pub async fn new(config: &'a Config) -> Uploader<'a> {
        let client = Request::new(&config).await;
  
        Self {
            client,
            config
        }
    }
    pub async fn upload_files(&mut self) {
        for file in &self.config.files {
            self.read_file(file).await;
        }
    }
    async fn read_file(&mut self, file_path: &str){
        let file_name = Path::new(file_path).file_name();
        if let Some(file_name) = file_name {
            match file_name.to_str() {
                Some(name) => {
                    self.upload(file_path, name).await;
                }
                None => {
                    eprint!("❌ 读取文件失败：{}", file_path);
                }
            }
        } else {
            eprint!("❌ 读取文件失败：{}", file_path);
        }
    }
    async fn create_session(&mut self, file_name: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}{}{}:/createUploadSession",
            MSAPI,
            &self.config.onedrive.drive,
            &self.config.onedrive.folder,
            file_name
        );
        
        let body = json!(
            {
                "@microsoft.graph.conflictBehavior": "rename"
            }
        );
        self.client.request(
            Some("post"),
            &url,
            B::String(body.to_string()),
            None).await
    }
    async fn upload(&mut self, file_path: &str, file_name: &str) {
        let file_size = Path::new(file_path).metadata().unwrap().len();
        let slice_count = file_size as f64 / SLICE_SIZE as f64;
        let slice_count = slice_count.ceil() as u64;
        
        
        
        println!("{} {}", "🚀 开始上传".green(), file_name);
        let mut session_ret = self.create_session(file_name).await;
        if let Ok(session_data) = &session_ret {
            match session_data.status() {
                StatusCode::UNAUTHORIZED => {
                    eprintln!("{} {}", "❌ token失效，正在重试".red(), file_name);
                    session_ret = self.create_session(file_name).await;
                }
                _ => {}
            }
        }
        match session_ret {
            Ok(session_data) => {
                match session_data.status() {
                    StatusCode::OK => {
                        println!("{} {}", "✅ 创建上传会话成功".green(), file_name);
                        let body = session_data.text().await.unwrap();
                        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
                        let upload_url = json["uploadUrl"].as_str().unwrap();
                        
                        let mut file = File::open(file_path).unwrap();
                        // 每次读取 5M
                        let mut file_buffer = vec![0; SLICE_SIZE as usize];
                        let mut index = 0;
                        
                        let p_bar = ProgressBar::new(slice_count);
                        
                        while index < slice_count {
                            let start = index * SLICE_SIZE as u64;
                            let mut end = (index + 1) * SLICE_SIZE as u64;
                            if end > file_size {
                                end = file_size;
                                file_buffer = vec![0; (end - start) as usize];
                            }
                            
                            
                            file.read(&mut file_buffer).unwrap();
                            
                            let bytes_range = format!("bytes {}-{}/{}", start, end - 1, file_size);
                            
                            let u_ret = 
                            self.upload_slice(&file_buffer, upload_url, &bytes_range).await;
                            
                            match u_ret {
                                Ok(r) => {
                                    match r.status() {
                                        StatusCode::CREATED |
                                        StatusCode::ACCEPTED => {
                                            p_bar.inc(1);
                                        }
                                        StatusCode::OK => {
                                            p_bar.inc(1);
                                            println!("{}", "✅ 上传成功".green());
                                        }
                                        _ => {
                                            eprintln!("{}", r.status());
                                            eprintln!("{}", r.text().await.unwrap());
                                            eprintln!("{}", "❌ 上传分片失败".red());
                                            continue
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    eprintln!("{} {}", "❌ 上传分片失败".red(), "正在重试".yellow());
                                    continue
                                }
                            }
                            
                            index += 1;
                            
                        }
                        p_bar.finish_and_clear();
                        println!("{} {}", "✅ 上传完成".green(), file_name);
                    }
                    _ => {
                        eprintln!("{}", session_data.text().await.unwrap());
                        eprintln!("{} {}", "❌ 创建上传会话失败".red(), file_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                eprintln!("{} {}", "❌ 创建上传会话失败".red(), file_name);
            }
        }
        
        
    }
    async fn upload_slice(&mut self, slice: &[u8], upload_url: &str, bytes_range: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_str(&format!("{}", slice.len())).unwrap());
        headers.insert(CONTENT_RANGE, HeaderValue::from_str(bytes_range).unwrap());
        // client.put(upload_url).headers(headers).body(slice.to_vec()).send().await
        self.client.request(
            Some("put"),
            upload_url,
            B::Vec(slice.to_vec()),
            Some(headers)
        ).await
    }
}