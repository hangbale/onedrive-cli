mod config;
mod uploader;
mod request;
#[tokio::main]
async fn main() {
    let cfg = config::get_config();
    let mut uploader = uploader::Uploader::new(&cfg).await;
    uploader.upload_files().await;
}
