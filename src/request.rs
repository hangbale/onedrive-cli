use reqwest::header::{
    HeaderMap,
    HeaderValue,
    CONTENT_TYPE,
    CONTENT_LENGTH,
    CONTENT_RANGE,
    AUTHORIZATION,
};
use reqwest::StatusCode;
use std::collections::HashMap;
use crate::config::Config;
use reqwest::Body;

type FormData<'a> = HashMap<&'a str, &'a str>;

pub enum B {
    Vec(Vec<u8>),
    String(String)
}

impl Clone for B {
    fn clone(&self) -> Self {
        match self {
            B::Vec(v) => B::Vec(v.clone()),
            B::String(s) => B::String(s.clone())
        }
    }
}
impl From<B> for Body {
    fn from(b: B) -> Self {
        match b {
            B::Vec(v) => Body::from(v),
            B::String(s) => Body::from(s)
        }
    }
}

type ReqBody = B;

static MSAPI: &str = "https://graph.microsoft.com/v1.0/";

fn gen_form_data<'a> (appid: &'a str, secret: &'a str, ms_graph_scope: &'a str) -> FormData<'a> {
    let mut form_data = HashMap::new();
    form_data.insert("client_id", appid);
    form_data.insert("client_secret", secret);
    form_data.insert("scope", ms_graph_scope);
    form_data.insert("grant_type", "client_credentials");
    form_data
}

pub struct Request<'a> {
    pub client: reqwest::Client,
    pub token: String,
    config: &'a Config,
}

impl<'a> Request<'a> {
    pub async fn new(config: &'a Config) -> Request<'a> {
        let token = Self::get_token(config).await;
        let client = Self::gen_client(&token);
        Request {
            client,
            token,
            config,
        }
    }
    pub fn gen_client(token: &str) -> reqwest::Client {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE,
            HeaderValue::from_static("application/json")
        );
        headers.insert(AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap()
        );
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    }
    pub async fn get_token(config: &Config) -> String {
        let form_data = gen_form_data(&config.onedrive.appid, &config.onedrive.secret, &config.onedrive.ms_graph_scope);
        let client = reqwest::Client::new();
        let res = client.post(&config.onedrive.token_endpoint)
            .form(&form_data)
            .send()
            .await.unwrap();
        let body = res.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let t = json["access_token"].as_str().unwrap().to_string();
        t
    }
    pub async fn re_auth(&mut self) {
        let token = Self::get_token(self.config).await;
        self.client = Self::gen_client(&token);
        self.token = token;
    }
    pub async fn request(&mut self, method: Option<&str>, url: &str, body: ReqBody, headers: Option<HeaderMap>) -> Result<reqwest::Response, reqwest::Error> {
        if self.token == "" {
            self.re_auth().await;
        }
        
        let mut req = self.client.get(url);
        if let Some(mth) = method {
            match mth {
                "post" => req = self.client.post(url),
                "put" => req = self.client.put(url),
                "delete" => req = self.client.delete(url),
                "get" => req = self.client.get(url),
                "head" => req = self.client.head(url),
                "patch" => req = self.client.patch(url),
                _ => {}
            }
        }
        if let Some(h) = headers {
            req = req.headers(h);
        }
        let res = req.body(body).send().await;
        match res {
            Ok(ret) => {
                match ret.status() {
                    StatusCode::UNAUTHORIZED => {
                        self.re_auth().await;
                        Ok(ret)
                    }
                    _ => Ok(ret)
                }
            }
            _ => res
        }
  

    }
}