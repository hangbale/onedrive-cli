mod config;
mod auth;
mod uploader;

#[tokio::main]
async fn main() {
    let cfg = config::get_config();
    uploader::upload_files(&cfg).await;
}
