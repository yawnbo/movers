use dirs::cache_dir;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

const BASE_SUBTITLES_URL: &str =
    "https://justaproxy.xyz/subsApi.php?version=2&getsubs=simp&imdbid=";
const BASE_SUBTITLES_URL_LANG: &str = "&subkey=eng";
const BASE_SUBTITLE_CACHE: &str = "movers/subtitles/";
pub async fn get_subtitles(imdb_id: String) -> Result<String, Box<dyn Error>> {
    let sub_cache = cache_dir().unwrap().join(BASE_SUBTITLE_CACHE);
    println!("{:?}", sub_cache);
    let _ = fs::create_dir_all(&sub_cache).unwrap(); // um i think this will error if the path exists
    // but we ball?
    let subtitles_url = format!(
        "{}{}{}",
        BASE_SUBTITLES_URL.to_string(),
        imdb_id,
        BASE_SUBTITLES_URL_LANG.to_string()
    );
    let client = Client::new();
    let subtitles_json: Value = client.get(subtitles_url).send().await?.json().await?;
    let subtitles = subtitles_json.as_array().unwrap();
    let mut mpv_subtitle_arg: String = "--sub-files=".to_string();
    println!("Don't panic this might hang for a while...");
    for object in subtitles.iter() {
        let file_path = sub_cache.join(
            object
                .get("SubFileName")
                .unwrap()
                .to_string()
                .trim_matches('"'),
        );
        let mut file = File::create(&file_path).unwrap();
        file.write_all(
            client
                .get(
                    object
                        .get("SubDownloadLink")
                        .unwrap()
                        .to_string()
                        .trim_matches('"'),
                )
                .send()
                .await?
                .bytes()
                .await?
                .as_ref(),
        )
        .unwrap();
        if subtitles[0] == *object {
            mpv_subtitle_arg = format!("{}{}", mpv_subtitle_arg, file_path.to_str().unwrap());
            continue;
        }
        mpv_subtitle_arg = format!("{}:{}", mpv_subtitle_arg, file_path.to_str().unwrap());
    }
    println!("{}", mpv_subtitle_arg);
    return Ok(mpv_subtitle_arg);
}
