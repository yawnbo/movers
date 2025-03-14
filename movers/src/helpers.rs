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
    let mut search_term = String::new();
    for i in 2..args.iter().len() {
        search_term.push_str(&args[i]);
        search_term.push_str(" ");
    }
    search_term = search_term.trim().to_string();
    let movie_list: Vec<Movie> = cflixscraping::init_client(&search_term).await?;

    if movie_list.is_empty() {
        return Err("I don't think we have that movie ... no results".into());
    }
    // handle amount of episodes and seasons here
    let selected_id = fzf_results(&movie_list).await?;
    let selected_id_parsed = selected_id
        .parse::<usize>()
        // this should literally never happen unless the returned json is terrible but less
        // unwrapping is good i guess
        .map_err(|_| format!("Invalid selection ID: {}", selected_id))?;

    // get this logic working on episodes
    let selected = movie_list.get(selected_id_parsed).unwrap();
    // if selected.series {
    //     let mut fzf = Fzf::default();
    //     fzf.run().expect("Failed to start fzf");
    //     for i in 1..selected.seasons.unwrap() + 1 {
    //         fzf.add_item(i.to_string()).expect("Failed to add items");
    //     }
    // }
    let selected_movie = selected;

    println!("Found movie id: {}", selected_movie.id);

    // setup async
    let subtitle_future = subtitles::get_subtitles(selected_movie.imdb_id.clone());
    let mpegts_future =
        cflixscraping::get_mpegts(format!("https://catflix.su/movie/{}", selected_movie.id));

    // wait for both to finish and pipe to mpv
    let (subtitle_arg, mpegts_url) = tokio::join!(subtitle_future, mpegts_future);
    let subtitle_arg = subtitle_arg?;
    let mpegts_url = mpegts_url?;

    println!("Starting mpv...");
    let status = Command::new("mpv")
        .arg(mpegts_url)
        .arg(subtitle_arg)
        .status()
        .map_err(|e| format!("Can't start mpv: {}", e))?;

    if status.success() {
        println!("MPV happy, hope you enjoyed the movie! :D");
    } else {
        eprintln!("MPV not happy: {:?}", status.code());
    }
    // iina-cli returns a status code while playback is still active and as a result subtitles are
    // cleared early, when using iina either comment out the cleaning of the cache and do it manually
    // afterwards or just leave it as is
    //
    // TODO:
    // iina support and cache timeout (maybe? i dont have a better idea) while using iina
    clean_subtitle_cache().await?;

    Ok(())
}

pub async fn clean_subtitle_cache() -> Result<(), Box<dyn Error>> {
    if let Some(cache_path) = cache_dir() {
        let subtitle_cache = cache_path.join(SUBTITLE_CACHE_DIR);

        if subtitle_cache.exists() {
            // Remove all files in the directory, there shouldn't be anything left
            let mut entries = fs::read_dir(&subtitle_cache).await?;
            println!("Cleaning subtitle cache directory: {:?}", subtitle_cache);
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path).await?;
                }
            }

            println!("Cache cleared!");
        } else {
            println!("did you manually clear the cache???");
        }
    } else {
        println!("uh where is the cache...");
    }

    Ok(())
}

// make sure valid json's are returned, this will just error anyway but it's fine
pub async fn check_json(json: &Value) -> Result<(), Box<dyn Error>> {
    if json.get("success").is_none_or(|a| !a.as_bool().unwrap()) {
        return Err("Failed to fetch valid json!".into());
    } else {
        println!("Valid json recieved!");
        return Ok(());
    }
}

// decryption with provided keys, i have no clue how this works
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
// convert from selected title to id, could probably be neglected with tuples but this works
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

// movie collecter this should be edited for episodes and series eventually, shouldn't be too bad
pub async fn get_movie_details(
    current_id: &str,
    movie_details: Value,
    vidtype: String,
    imdb_id: String,
) -> Movie {
    if vidtype == "tv" {
        println!("Returning series");
        return Movie {
            title: movie_details
                .get("original_name")
                .unwrap()
                .to_string()
                .trim_matches('"')
                .to_string(),
            release_date: movie_details
                .get("first_air_date")
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
            imdb_id,
            series: true,
            // TODO: get this working bitch
            seasons: None,
        };
    }
    println!("Returning movie");
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
        series: false,
        seasons: None,
    };
}
