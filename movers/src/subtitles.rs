use dirs::cache_dir;
use flate2::read::GzDecoder;
use futures::future::join_all;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;

const BASE_SUBTITLES_URL: &str = "https://justaproxy.xyz/subsApi.php?version=2&getsubs=simp";
const BASE_SUBTITLES_URL_LANG: &str = "&subkey=eng";
const BASE_SUBTITLE_CACHE: &str = "movers/subtitles/";

async fn ensure_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        tokio_fs::create_dir_all(path).await?;
    }
    Ok(())
}

// hotfix dont know if works
async fn download_subtitle(
    client: &Client,
    sub_cache: &Path,
    subtitle_info: &Value,
) -> Result<PathBuf, Box<dyn Error>> {
    let file_name = subtitle_info
        .get("SubFileName")
        .ok_or("Missing SubFileName")?
        .as_str()
        .ok_or("SubFileName is not a string")?;
    let download_link = subtitle_info
        .get("SubDownloadLink")
        .ok_or("Missing SubDownloadLink")?
        .as_str()
        .ok_or("SubDownloadLink is not a string")?;
    let file_path = sub_cache.join(file_name);
    let response = client.get(download_link).send().await?;
    let content = response.bytes().await?;

    // Check for gzip magic headers (0x1F 0x8B)
    let final_content = if content.len() >= 2 && content[0] == 0x1F && content[1] == 0x8B {
        let mut decoder = GzDecoder::new(&content[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        decompressed
    } else {
        content.to_vec()
    };

    let mut file = tokio_fs::File::create(&file_path).await?;
    file.write_all(&final_content).await?;
    Ok(file_path)
}
pub async fn get_subtitles(
    imdb_id: String,
    series: bool,
    episode_num: String,
    season_num: String,
) -> Result<String, Box<dyn Error>> {
    let sub_cache = cache_dir()
        .ok_or("Failed to determine cache directory")?
        .join(BASE_SUBTITLE_CACHE);

    println!("[INFO] Writing subtitles to: {:?}", sub_cache);

    ensure_directory(&sub_cache).await?;
    let subtitles_url: String;
    if !series {
        subtitles_url = format!(
            "{}&imdbid={}{}",
            BASE_SUBTITLES_URL, imdb_id, BASE_SUBTITLES_URL_LANG
        );
    } else {
        subtitles_url = format!(
            "{}episode={}&imdbid={}&season={}{}",
            BASE_SUBTITLES_URL, episode_num, imdb_id, season_num, BASE_SUBTITLES_URL_LANG
        );
    }
    println!("[INFO] Subtitle URL: {}", subtitles_url);

    let client = Client::new();

    let response = client.get(&subtitles_url).send().await?;
    let subtitles_json: Value = response.json().await?;
    let subtitles = subtitles_json
        .as_array()
        .ok_or("Subtitle data is not an array")?;

    if subtitles.is_empty() {
        return Err("[ERROR] No subtitles found".into());
    }

    println!(
        "[INFO] Downloading {} subtitles (this might take a moment)...",
        subtitles.len()
    );

    let download_futures = subtitles
        .iter()
        .map(|subtitle| download_subtitle(&client, &sub_cache, subtitle));

    let results = join_all(download_futures).await;

    let mut mpv_subtitle_arg = "--sub-files=".to_string();
    let mut first = true;

    for result in results {
        match result {
            Ok(file_path) => {
                if let Some(path_str) = file_path.to_str() {
                    if first {
                        mpv_subtitle_arg.push_str(path_str);
                        first = false;
                    } else {
                        mpv_subtitle_arg.push_str(":");
                        mpv_subtitle_arg.push_str(path_str);
                    }
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to download a subtitle: {}", e);
            }
        }
    }
    Ok(mpv_subtitle_arg)
}
