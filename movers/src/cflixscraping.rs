// NOTE:
//
// Episodes look horrible to try and fetch so some notes on how to work on it,
// load additional data for tvid playert
// if (main_origin != "" && next_episode_url != "") {
//   let to = atob(main_origin);
//   let json = JSON.stringify({
//     next_episode_url,
//     LOCALE
//   });
//   to = safeBtoa(to + "/" + safeBtoa(json));
//   main_origin = to;
// }
//
// const sourceUrls = [main_origin, safeBtoa("https://vidlink.pro/tv/" + series_tmdb_id + "/" + def_season + "/" + def_episode),
// chernobyl 1-1 example, https://vidlink.pro/tv/87108/1/1
// no clue what to do with this tho
use crate::Movie;
use base64::prelude::*;
use futures::future::join_all;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::error::Error;

use crate::helpers;

const BASE_URL: &str = "https://catflix.su/";
// remove movie from the end as it can be tv/movie
const TMDB_API_URL: &str = "https://api.themoviedb.org/3";
// this is not my key im not leaking sensitive data :)
const TMDB_API_HEADER: &str = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiIyOTIzODEzYzIwNWUzZDRjNGY4ZGVhNmFjZTQ2YTMwMiIsIm5iZiI6MTcyMzA1MTM1Mi44MjY2NTEsInN1YiI6IjY2YjNhYzM2YjMwNGY1Nzg1Y2UxODQwYyIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.Nx0oSk9Ts8LurGRrSZk5b-QE172zZ_dCLNT9WJJFLbc";

pub async fn init_client(search: &str) -> Result<Vec<Movie>, Box<dyn Error>> {
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
        // make prettier prints
        println!("Fetching movie details from tmdb api...");

        // TODO:
        //
        // currently crashes when a series pops up because it's not handled and tmdb returns
        // invalid json, instead check the other element in each |movie| to see if it's a
        // 'movie_id' or 'series_id', if it's series set a bool to true in the movie struct and add
        // how many epidoes/seasons there are. remember to allow for another 2 fzf calls to choose
        // the season and episode.
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
                        println!("imdb_id: {}", imdb_id);
                    }
                    println!("{}", tmdb_api_call);
                    let movie_details =
                        get_movie_search(&tmdb_api_call, &TMDB_API_HEADER, &client_clone).await;
                    println!("{:?}", movie_details);
                    helpers::get_movie_details(&current_id, movie_details, vidtype, imdb_id).await
                }
            })
            .collect::<Vec<_>>();

        let movie_results = futures::future::join_all(movie_futures).await;
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

    let re_main_origin = Regex::new(r#"const main_origin\s*=\s*"([^"]*)";"#).unwrap();
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
