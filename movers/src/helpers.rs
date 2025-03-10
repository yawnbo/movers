use crate::Movie;
use dirs::cache_dir;
use fzf_wrapped::Fzf;
use serde_json::Value;
use std::error::Error;
use std::process::Command;
use tokio::fs;

use crate::cflixscraping;
use crate::subtitles;
const SUBTITLE_CACHE_DIR: &str = "movers/subtitles/";

pub async fn search_and_play(args: &[String]) -> Result<(), Box<dyn Error>> {
    let search_term = args.get(2).ok_or("No search term provided")?;

    let movie_list: Vec<Movie> = cflixscraping::init_client(search_term).await?;

    if movie_list.is_empty() {
        return Err("No movies found matching the search term".into());
    }

    let selected_id = fzf_results(&movie_list).await?;
    let selected_id_parsed = selected_id
        .parse::<usize>()
        .map_err(|_| format!("Invalid selection ID: {}", selected_id))?;

    let selected_movie = movie_list
        .get(selected_id_parsed)
        .ok_or_else(|| format!("Selection out of range: {}", selected_id_parsed))?;

    println!("Found movie: {}", selected_movie.id);

    let subtitle_future = subtitles::get_subtitles(selected_movie.imdb_id.clone());
    let mpegts_future =
        cflixscraping::get_mpegts(format!("https://catflix.su/movie/{}", selected_movie.id));

    let (subtitle_arg, mpegts_url) = tokio::join!(subtitle_future, mpegts_future);
    let subtitle_arg = subtitle_arg?;
    let mpegts_url = mpegts_url?;

    println!("Starting mpv...");
    let status = Command::new("mpv")
        .arg(mpegts_url)
        .arg(subtitle_arg)
        .status()
        .map_err(|e| format!("Failed to start mpv: {}", e))?;

    if status.success() {
        println!("Movie playback completed successfully");
    } else {
        eprintln!("MPV exited with error code: {:?}", status.code());
    }

    clean_subtitle_cache().await?;

    Ok(())
}

pub async fn clean_subtitle_cache() -> Result<(), Box<dyn Error>> {
    if let Some(cache_path) = cache_dir() {
        let subtitle_cache = cache_path.join(SUBTITLE_CACHE_DIR);

        if subtitle_cache.exists() {
            // Remove all files in the directory
            let mut entries = fs::read_dir(&subtitle_cache).await?;
            println!("Cleaning subtitle cache directory: {:?}", subtitle_cache);
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path).await?;
                }
            }

            println!("Subtitle cache cleaned successfully");
        } else {
            println!("No subtitle cache directory found to clean");
        }
    } else {
        println!("Could not determine cache directory location");
    }

    Ok(())
}
pub async fn check_json(json: &Value) -> Result<(), Box<dyn Error>> {
    if json.get("success").unwrap() == "false" {
        return Err("Failed to fetch valid json!".into());
    } else {
        println!("Valid json recieved!");
        return Ok(());
    }
}

pub async fn decrypt(cyphertext: String, key: String) -> String {
    let decimal_cypher: Vec<u8> = (0..cyphertext.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&cyphertext[i..i + 2], 16).ok())
        .collect();

    decimal_cypher
        .iter()
        .enumerate()
        .map(|(i, &byte)| (byte ^ key.as_bytes()[i % key.len()]) as char)
        .collect()
}
pub async fn find_movie_id(
    selection: String,
    movie_list: &Vec<Movie>,
) -> Result<String, Box<dyn Error>> {
    return Ok(movie_list
        .iter()
        .position(|a| a.title == selection)
        .unwrap()
        .to_string());
}
pub async fn fzf_results(movie_list: &Vec<Movie>) -> Result<String, Box<dyn Error>> {
    // TODO:
    // append a load more element to allow for the second page to be loaded +1
    let movie_titles: Vec<String> = movie_list.iter().map(|a| a.title.clone()).collect();
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(movie_titles).expect("Failed to add items");
    let selection = fzf.output().unwrap();
    return Ok(find_movie_id(selection, &movie_list).await?);
}
// helper
pub async fn get_movie_details(current_id: &str, movie_details: Value) -> Movie {
    return Movie {
        title: movie_details
            .get("original_title")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        release_date: movie_details
            .get("release_date")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        tagline: movie_details
            .get("tagline")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        overview: movie_details
            .get("overview")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        poster_path: movie_details
            .get("poster_path")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
        average_rating: movie_details
            .get("vote_average")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .parse()
            .unwrap(),
        id: current_id.to_string(),
        imdb_id: movie_details
            .get("imdb_id")
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string(),
    };
}
