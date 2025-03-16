use base64::prelude::*;
use futures::future::join_all;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::error::Error;

use crate::Episode;
use crate::Season;
use crate::WatchItem;
use crate::helpers;

const BASE_URL: &str = "https://catflix.su/";
// remove movie from the end as it can be tv/movie
const TMDB_API_URL: &str = "https://api.themoviedb.org/3";
// this is not my key im not leaking sensitive data :)
const TMDB_API_HEADER: &str = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiIyOTIzODEzYzIwNWUzZDRjNGY4ZGVhNmFjZTQ2YTMwMiIsIm5iZiI6MTcyMzA1MTM1Mi44MjY2NTEsInN1YiI6IjY2YjNhYzM2YjMwNGY1Nzg1Y2UxODQwYyIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.Nx0oSk9Ts8LurGRrSZk5b-QE172zZ_dCLNT9WJJFLbc";

pub async fn init_client(search: &str) -> Result<Vec<WatchItem>, Box<dyn Error>> {
    let client = Client::new();
    let json_query_url = format!(
        "{}api/autocomplete/?q={}&page=1&route=search&sid=0&context=all",
        BASE_URL, search
    );
    println!("Client initialized, fetching json from {}", json_query_url);

    let json_response: Value = client.get(json_query_url).send().await?.json().await?;

    let _ = helpers::check_json(&json_response).await;

    if let Some(data_array) = json_response.get("data").unwrap().as_array() {
        // TODO:
        // ugly ass print, make prettier prints please
        println!("Fetching movie details from tmdb api...");
        let movie_futures = data_array
            .iter()
            .map(|movie| {
                let current_id = movie.get("tmdb_id").unwrap().to_string();
                let client_clone = client.clone();
                let vidtype: String;
                let mut imdb_id: String = "".to_string();
                if let Some(_) = movie.get("movie_id") {
                    vidtype = "movie".to_string();
                } else {
                    vidtype = "tv".to_string();
                }

                let tmdb_api_call =
                    format!("{}/{}/{}?language=en", TMDB_API_URL, vidtype, current_id);

                async move {
                    // i dont actually remember what this was put in here for and I don't feel like
                    // tracing it so im leaving it here and adding a todo tag
                    // TODO:
                    // REWRITE
                    if vidtype == "tv" {
                        let call_url = format!("{}/tv/{}/external_ids", TMDB_API_URL, current_id);
                        let json: Value = client_clone
                            .get(call_url)
                            .header("Authorization", TMDB_API_HEADER)
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                        imdb_id = json
                            .get("imdb_id")
                            .map(|id| id.as_str().unwrap().to_string())
                            .unwrap();
                    }
                    let movie_details =
                        get_movie_search(&tmdb_api_call, &TMDB_API_HEADER, &client_clone).await;
                    helpers::get_movie_details(&current_id, movie_details, vidtype, imdb_id).await
                }
            })
            .collect::<Vec<_>>();

        let movie_results = join_all(movie_futures).await;
        println!("tmdb api response OK");

        return Ok(movie_results);
    }

    Err("not happy :(".into())
}

pub async fn get_mpegts(catflix_movie_url: String) -> Result<String, Box<dyn Error>> {
    // something there takes super long... i'm not finding it.....
    let client = Client::new();

    let catflix_movie_html_response = client.get(&catflix_movie_url).send().await?.text().await?;

    let catflix_movie_document = Html::parse_document(&catflix_movie_html_response);
    let script_selector = Selector::parse("script").expect("Failed to parse selector");

    // upstream decided to encode extra data in their player for playing the next episode and they
    // mutilate the main origin to do so so it isn't const for episodes but is for movies
    let re_main_origin = Regex::new(r#"(?:const|let) main_origin\s*=\s*"([^"]*)";"#).unwrap();
    let re_apkey = Regex::new(r#"const apkey\s*=\s*"([^"]*)";"#).unwrap();
    let re_xxid = Regex::new(r#"const xxid\s*=\s*"([^"]*)";"#).unwrap();

    let last_script = catflix_movie_document
        .select(&script_selector)
        .last()
        .map(|script| script.inner_html())
        .ok_or("Failed to find script tag")?;

    let pre_juice_target_encoded = re_main_origin
        .captures(&last_script)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .ok_or("Failed to extract main_origin")?
        .to_string();
    if pre_juice_target_encoded == "" {
        return Err("main_origin isn't specified on scraped website! Please see above cflix call and choose another source from there, this app currently doesn't support sources apart from the default.\n".into());
    }
    let embed_api_url = BASE64_STANDARD
        .decode(pre_juice_target_encoded.as_bytes())?
        .iter()
        .map(|&a| a as char)
        .collect::<String>();

    let embed_api_html_response = client.get(&embed_api_url).send().await?.text().await?;

    let embed_api_document = Html::parse_document(&embed_api_html_response);
    let embed_script = embed_api_document
        .select(&script_selector)
        .nth_back(1)
        .map(|script| script.inner_html())
        .ok_or("Failed to find embed script")?;

    let apkey = re_apkey
        .captures(&embed_script)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .ok_or("Failed to extract apkey")?
        .to_string();

    let xxid = re_xxid
        .captures(&embed_script)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .ok_or("Failed to extract xxid")?
        .to_string();

    let juice_key_url = "https://turbovid.eu/api/cucked/juice_key";
    let juice_data_url = format!(
        "https://turbovid.eu/api/cucked/the_juice_v2/?{}={}",
        apkey, xxid
    );

    let juice_key_future = async {
        let juice_key_json: Value = client
            .get(juice_key_url)
            .header("Referer", &embed_api_url)
            .send()
            .await?
            .json()
            .await?;

        let _ = helpers::check_json(&juice_key_json).await;

        let juice_key = juice_key_json
            .get("juice")
            .ok_or("Missing juice field")?
            .as_str()
            .ok_or("Juice value is not a string")?
            .to_string();

        Ok::<String, Box<dyn Error>>(juice_key)
    };

    let juice_data_future = async {
        let juice_data_json: Value = client
            .get(&juice_data_url)
            .header("Referer", &embed_api_url)
            .send()
            .await?
            .json()
            .await?;

        let _ = helpers::check_json(&juice_data_json).await;

        let data_crypted = juice_data_json
            .get("data")
            .ok_or("Missing data field")?
            .as_str()
            .ok_or("Data value is not a string")?
            .to_string();

        Ok::<String, Box<dyn Error>>(data_crypted)
    };

    let (juice_key, data_crypted) = futures::try_join!(juice_key_future, juice_data_future)?;

    println!("data: {}, Key: {}", data_crypted, juice_key);
    Ok(helpers::decrypt(data_crypted, juice_key).await)
}

pub async fn populate_episodes(season: &Season, series_id: &String) -> Vec<Episode> {
    // this is done at series select time which is fine and works but if more speed is ever wanted
    // this can be done async while waiting for user to select season, downside is there will be
    // lot's of unneeded requests bcz only one season will be selected and this will fetch all of
    // them. price to pay for filling it quicker i guess
    let client = Client::new();
    let call_url = format!(
        "{}/tv/{}/season/{}?language=en",
        TMDB_API_URL, series_id, season.number
    );
    let json: Value = client
        .get(call_url)
        .header("Authorization", TMDB_API_HEADER)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    if let Some(episodes_array) = json.get("episodes").unwrap().as_array() {
        // Create a vector of futures
        let episode_futures: Vec<_> = episodes_array
            .iter()
            .map(|episode| {
                let id = episode.get("id").unwrap().to_string();
                let title = episode.get("name").unwrap().to_string();
                let overview = episode
                    .get("overview")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                let number = episode
                    .get("episode_number")
                    .unwrap()
                    .to_string()
                    .parse::<usize>()
                    .unwrap();
                let client_clone = client.clone();

                async move {
                    let imdb_id =
                        get_imdb_id(&season.number, &number, &client_clone, &series_id).await;
                    Episode {
                        overview,
                        title,
                        number,
                        id,
                        imdb_id,
                    }
                }
            })
            .collect();

        let episodes = futures::future::join_all(episode_futures).await;
        return episodes;
    } else {
        // ya i should handle this better tbh
        return Vec::new();
    }
}

// needed for subs because imdb id isn't provided by tmdb api on the default call
async fn get_imdb_id(
    season_number: &usize,
    episode_number: &usize,
    client: &Client,
    series_id: &String,
) -> String {
    let call_url = format!(
        "{}/tv/{}/season/{}/episode/{}/external_ids",
        TMDB_API_URL, series_id, season_number, episode_number
    );
    println!("call_url: {}", call_url);
    let json: Value = client
        .get(call_url)
        .header("Authorization", TMDB_API_HEADER)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    json.get("imdb_id")
        .map(|id| id.as_str().unwrap().to_string())
        .unwrap()
}
async fn get_movie_search(call: &str, header: &str, client: &Client) -> Value {
    client
        .get(call)
        .header("Authorization", header)
        .send()
        .await
        .unwrap() // error handling here
        .json()
        .await
        .unwrap() // error handling here
}
