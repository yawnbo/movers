use crate::Season;
use crate::WatchItem;
use dirs::cache_dir;
use fzf_wrapped::Fzf;
use serde_json::Value;
use std::error::Error;
use std::process::Command;
use tokio::fs;

use crate::Episode;
use crate::HasTitle;
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
    let movie_list: Vec<WatchItem> = cflixscraping::init_client(&search_term).await?;

    if movie_list.is_empty() {
        return Err("I don't think we have that movie ... no results".into());
    }
    // handle amount of episodes and seasons here
    let selected_id = fzf_results(&movie_list, false).await?;
    let selected_id_parsed = selected_id
        .parse::<usize>()
        // this should literally never happen unless the returned json is terrible but less
        // unwrapping is good i guess
        .map_err(|_| format!("Invalid selection ID: {}", selected_id))?;

    let selected = movie_list.get(selected_id_parsed).unwrap();
    let mut cflix_call = "https://catflix.su".to_string();
    if selected.series {
        // if selection is a series
        let selected_series: &Vec<Season> = selected.seasons.as_ref().unwrap();
        let season_title = fzf_results(&selected_series, true).await?;
        // its doable to speculatively fetch the episodes and probably worth it but for
        // now i'll do it jit, but this can be changed if performance is bad after season
        // selection.
        let selected_season = selected_series
            .iter()
            .find(|a| a.get_title() == season_title)
            .unwrap();
        // maybe put debug/print statements here to show the user the program is even doing
        // anything?
        let episodes: Vec<Episode> =
            cflixscraping::populate_episodes(&selected_season, &selected.id).await;
        let selected_episode_title = fzf_results(&episodes, true).await?;
        let selected_episode = episodes
            .iter()
            .find(|a| a.get_title() == selected_episode_title)
            .unwrap();

        cflix_call = format!(
            "{}/episode/{}-season-{}-episode-{}/eid-{}",
            cflix_call,
            selected.title.to_lowercase(),
            selected_season.number,
            selected_episode.number,
            selected_episode.id
        );
        println!("Episode cflix call: {}", cflix_call);
    } else {
        cflix_call = format!("{}/movie/{}", cflix_call, selected.id);
    }
    println!("Found id: {}", selected.id);

    // setup async
    let subtitle_future = subtitles::get_subtitles(selected.imdb_id.clone());
    let mpegts_future = cflixscraping::get_mpegts(cflix_call);

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

// decryption with provided keys
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
pub async fn find_movie_id<T: HasTitle>(
    selection: String,
    movie_list: &[T],
) -> Result<String, Box<dyn Error>> {
    return Ok(movie_list
        .iter()
        .position(|a| a.get_title() == selection)
        .unwrap()
        .to_string());
}
pub async fn fzf_results<T: HasTitle>(
    movie_list: &[T],
    series: bool,
) -> Result<String, Box<dyn Error>> {
    // um this readable was supposed to have numbers on it but that made fetching the id again
    // REALLY annoying because the selection is whatever you put into it so ill sort this out later
    // TODO:
    let readable_string;
    if series {
        readable_string = movie_list
            .iter()
            .map(|item| item.get_title())
            .collect::<Vec<_>>();
    } else {
        readable_string = movie_list.iter().map(|a| a.get_title()).collect();
    }
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(readable_string).expect("Failed to add items");
    let selection = fzf.output().unwrap();
    if !series {
        return Ok(find_movie_id(selection, &movie_list).await?);
    } else {
        // i want to manually convert this if it's a series
        return Ok(selection);
    }
}
pub async fn get_movie_details(
    current_id: &str,
    movie_details: Value,
    vidtype: String,
    imdb_id: String,
) -> WatchItem {
    if vidtype == "tv" {
        println!("Returning series");
        return WatchItem {
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
            seasons: get_season_vec(movie_details).await.ok(),
        };
    }
    println!("Returning movie");
    return WatchItem {
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
async fn get_season_vec(movie_details: Value) -> Result<Vec<Season>, Box<dyn Error>> {
    let mut season_vec: Vec<Season> = Vec::new();

    // rust likes to throw a warning that the code is unreadble with just a for loop so this it is
    if let Some(seasons_array) = movie_details.get("seasons") {
        if let Some(seasons) = seasons_array.as_array() {
            for season in seasons {
                season_vec.push(Season {
                    overview: season
                        .get("overview")
                        .map(|a| a.as_str().unwrap_or("").to_string()),
                    number: season
                        .get("season_number")
                        .and_then(|e| e.as_i64())
                        .unwrap_or(0) as usize,
                    title: season
                        .get("name")
                        .map(|a| a.as_str().unwrap_or(""))
                        .unwrap_or("")
                        .to_string(),
                    id: season.get("id").map(|a| a.to_string()).unwrap_or_default(),
                    episode_count: season
                        .get("episode_count")
                        .and_then(|e| e.as_i64())
                        .unwrap_or(0) as usize,
                    // episodes can be fetched here with async but itll be expensive
                    // and probably wont save much with the current flow of subtitles
                    // and key fetching
                    //
                    // now that i look at this terrible code again i dont think i even ended up
                    // linking the episodes and season vectors so i'll need to do that eventually
                    // TODO:
                    episodes: None,
                });
            }
        }
    }
    return Ok(season_vec);
}
