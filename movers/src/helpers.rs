use crate::Season;
use crate::WatchItem;
use dirs::cache_dir;
use fzf_wrapped::Fzf;
use serde_json::Value;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

use crate::Episode;
use crate::HasTitle;
use crate::cflixscraping;
use crate::subtitles;
const SUBTITLE_CACHE_DIR: &str = "movers/subtitles/";

pub async fn get_link(args: &[String]) -> Result<(String, String), Box<dyn Error>> {
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
    let selected_id = fzf_results(&movie_list).await?;

    let selected = movie_list.get(selected_id).unwrap();
    let mut cflix_call = "https://catflix.su".to_string();
    let selected_episode: &Episode;
    let subtitle_future;
    if selected.series {
        // if selection is a series
        let selected_series: &Vec<Season> = selected.seasons.as_ref().unwrap();
        let season_index = fzf_results(&selected_series).await?;
        let selected_season = selected_series.iter().nth(season_index).unwrap();
        // its doable to speculatively fetch the episodes and probably worth it but for
        // now i'll do it jit, but this can be changed if performance is bad after season
        // selection.
        // maybe put debug/print statements here to show the user the program is even doing
        // anything?
        let episodes: Vec<Episode> = cflixscraping::populate_episodes(
            &selected_series.iter().nth(season_index).unwrap(),
            &selected.id,
        )
        .await;
        let selected_episode_index = fzf_results(&episodes).await?;
        selected_episode = episodes.iter().nth(selected_episode_index).unwrap();

        cflix_call = format!(
            "{}/episode/{}-season-{}-episode-{}/eid-{}",
            cflix_call,
            selected.title.to_lowercase(),
            selected_season.number,
            selected_episode.number,
            selected_episode.id
        );
        println!("[INFO] Found indirect episode link: {}", cflix_call);

        subtitle_future = subtitles::get_subtitles(
            selected.imdb_id.clone(),
            true,
            selected_episode.number.to_string(),
            selected_season.number.to_string(),
        );
    } else {
        cflix_call = format!("{}/movie/{}", cflix_call, selected.id);
        subtitle_future = subtitles::get_subtitles(
            selected.imdb_id.clone(),
            false,
            "".to_string(),
            "".to_string(),
        );
    }
    println!("[INFO] Selected item id: {}", selected.id);

    // setup async
    let mpegts_future = cflixscraping::get_mpegts(cflix_call);

    // wait for both to finish and pipe to mpv
    let (subtitle_arg, mpegts_url) = tokio::join!(subtitle_future, mpegts_future);
    let subtitle_arg = subtitle_arg?;
    let mpegts_url = mpegts_url?;
    return Ok((mpegts_url, subtitle_arg));
}

pub async fn play_movie(mpegts_url: String, subtitle_arg: String) -> Result<(), Box<dyn Error>> {
    println!("[INFO] Starting MPV");
    // should make a config option with iina that just adds --mpv-* to the flag as its really an
    // mpv wrapper. Also make sure to add the flag --keep-running so we don't clear subtitles
    // early.
    let status = Command::new("mpv")
        .arg(mpegts_url)
        .arg(subtitle_arg)
        // comment for more verbosity, currently the mpegts is really weird and ffmpeg
        // can't decide on a proper packet size which leads to lots of spam. Packets also
        // appear corrupt to ffmpeg but the video plays fine, although mpv says the time
        // remaining is 22:59:59 which is weird.
        // TODO:
        // nothing was changed on my end, and from surface testing I don't think anything
        // was changed on the cflix end, will need to look into it more, but the program
        // still works as intented.
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("Can't start mpv: {}", e))?;

    if status.success() {
        println!("[SUCCESS] MPV happy, hope you enjoyed the movie! :D");
    } else {
        eprintln!("[ERROR] MPV not happy: {:?}", status.code());
    }
    clean_subtitle_cache().await?;

    Ok(())
}
pub async fn clean_subtitle_cache() -> Result<(), Box<dyn Error>> {
    if let Some(cache_path) = cache_dir() {
        let subtitle_cache = cache_path.join(SUBTITLE_CACHE_DIR);

        if subtitle_cache.exists() {
            // Remove all files in the directory, there shouldn't be anything left
            let mut entries = fs::read_dir(&subtitle_cache).await?;
            println!(
                "[INFO] Cleaning subtitle cache directory: {:?}",
                subtitle_cache
            );
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path).await?;
                }
            }

            println!("[OK] Cache cleared!");
        } else {
            println!("[ERROR] did you manually clear the cache???");
        }
    } else {
        println!("[ERROR] uh where is the cache...");
    }

    Ok(())
}

// make sure valid json's are returned, this will just error anyway but it's fine
pub async fn check_json(json: &Value) -> Result<(), Box<dyn Error>> {
    if json.get("success").is_none_or(|a| !a.as_bool().unwrap()) {
        return Err("Failed to fetch valid json!".into());
    } else {
        return Ok(());
    }
}

// decryption with provided keys
pub async fn decrypt(cyphertext: String, key: String) -> String {
    let decimal_cypher: Vec<u8> = (0..cyphertext.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&cyphertext[i..i + 2], 16).ok())
        .collect();

    let mpegts = decimal_cypher
        .iter()
        .enumerate()
        .map(|(i, &byte)| (byte ^ key.as_bytes()[i % key.len()]) as char)
        .collect();
    println!("[INFO] Decrypted mpegts url: {}", mpegts);
    return mpegts;
}
pub async fn fzf_results<T: HasTitle>(movie_list: &[T]) -> Result<usize, Box<dyn Error>> {
    let mut special: bool = false;
    let readable_string = movie_list
        .iter()
        .enumerate()
        .map(|(i, item)| {
            item.get_title();
            if special | (item.get_title().to_lowercase() == "specials") {
                special = true;
                return format!("{}: {}", i, item.get_title());
            } else {
                format!("{}: {}, {}", i + 1, item.get_title(), item.get_overview())
            }
        })
        .collect::<Vec<_>>();
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(readable_string).expect("Failed to add items");
    let selection = fzf.output().unwrap();

    let selected_index = selection
        .split(':')
        .next()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if !special {
        return Ok(selected_index - 1);
    }
    return Ok(selected_index);
}
pub async fn get_movie_details(
    current_id: &str,
    movie_details: Value,
    vidtype: String,
    imdb_id: String,
) -> WatchItem {
    if vidtype == "tv" {
        println!("[INFO] Found series");
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
    println!("[INFO] Found movie");
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
pub async fn ensure_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

pub async fn download_basic(link: String) {
    // TODO:
    // Check ffmpeg version as the flag only works on 7.1+ as far as im aware
    Command::new("ffmpeg")
        .arg("-allowed_extensions")
        .arg("ALL")
        .arg("-extension_picky")
        .arg("0")
        .arg("-i")
        .arg(link)
        .arg("output.mp4")
        .status()
        .unwrap();
    return;
}
