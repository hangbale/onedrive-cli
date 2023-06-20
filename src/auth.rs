use std::collections::HashMap;
use crate::config::Config;
use reqwest;

type FormData<'a> = HashMap<&'a str, &'a str>;

fn gen_form_data<'a> (appid: &'a str, secret: &'a str, ms_graph_scope: &'a str) -> FormData<'a> {
    let mut form_data = HashMap::new();
    form_data.insert("client_id", appid);
    form_data.insert("client_secret", secret);
    form_data.insert("scope", ms_graph_scope);
    form_data.insert("grant_type", "client_credentials");
    form_data
}
pub async fn get_token (config: &Config) -> String {
    let form_data = gen_form_data(&config.onedrive.appid, &config.onedrive.secret, &config.onedrive.ms_graph_scope);
    let client = reqwest::Client::new();
    let res = client.post(&config.onedrive.token_endpoint)
        .form(&form_data)
        .send()
        .await.unwrap();
    let body = res.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    json["access_token"].as_str().unwrap().to_string()
}