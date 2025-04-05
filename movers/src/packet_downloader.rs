const BASE_CACHE: &str = "movers";
const BASE_PACKET_CACHE: &str = "movers/packets/";

use dirs::cache_dir;
use futures::future::join_all;
use reqwest::Client;
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn download_video_raw(hls_path: String) -> Result<(), Box<dyn Error>> {
    // first pull the .m3u8
    let client = Client::new();
    return Ok(());
}
