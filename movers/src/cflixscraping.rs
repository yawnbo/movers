use crate::Movie;
use base64::prelude::*;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::error::Error;

use crate::helpers;

// TODO: this is an async mess and can be optimized i think (juice part in specific)
pub async fn init_client(search: &str) -> Result<Vec<Movie>, Box<dyn Error>> {
    let mut movie_results: Vec<Movie> = Vec::new();
    let client = Client::new();
    let base_url: &str = "https://catflix.su/";
    let json_query_url = format!(
        "{}api/autocomplete/?q={}&page=1&route=search&sid=0&context=all",
        base_url, search
    );
    println!("Client initialized, fetching json from {}", json_query_url);
    let json_response: Value = client
        .get(json_query_url)
        .send()
        .await
        .expect("Failed to recieve JSON")
        .json()
        .await?;
    let _ = helpers::check_json(&json_response);
    if let Some(data_array) = json_response.get("data").unwrap().as_array() {
        // this should really be a function to allow for use with other scrapers like bflix but let
        // me live for now man

        // This api key is catflix's tmdb api key im not leaking sensitive data :)
        let api_auth_header = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiIyOTIzODEzYzIwNWUzZDRjNGY4ZGVhNmFjZTQ2YTMwMiIsIm5iZiI6MTcyMzA1MTM1Mi44MjY2NTEsInN1YiI6IjY2YjNhYzM2YjMwNGY1Nzg1Y2UxODQwYyIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.Nx0oSk9Ts8LurGRrSZk5b-QE172zZ_dCLNT9WJJFLbc";
        let api_base_url = "https://api.themoviedb.org/3/movie/";
        println!("Fetching movie details from tmdb api...");

        // TODO:
        // this should be loading aynchrously with tokio but I don't know to how to use tokio and
        // want this to just work so this will be done later.
        for movie in data_array {
            // Also optionally, see the difference for looking at series and episodes instead of
            // movies. I'm not sure how the search querying is different but might work on it once
            // basics are stable.
            // NOTE:
            // DO NOT WATCH THE MOVIE "her" USING THIS LMAO PLEASE FIND OTHER MOVIES THAT DO THIS
            // TOO
            let current_id = movie.get("tmdb_id").unwrap().to_string();
            let tmdb_api_call = format!("{}{}?language=en", api_base_url, current_id);
            let movie_details =
                get_movie_search(tmdb_api_call, api_auth_header.to_string(), client.clone()).await;
            let temp_movie: Movie = helpers::get_movie_details(&current_id, movie_details).await;
            movie_results.push(temp_movie);
        }
        println!("tmdb api response OK")
    }
    return Ok(movie_results);
}
pub async fn get_mpegts(catflix_movie_url: String) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let catflix_movie_html_response = client
        .get(catflix_movie_url)
        .send()
        .await
        .expect("Failed to fetch valid html from catflix movie url")
        .text()
        .await?;

    let catflix_movie_document = Html::parse_document(catflix_movie_html_response.as_str());
    let script_binding = Selector::parse("script").expect("Failed to parse selector");

    // Regex to be used for finding vars
    let re_main_origin = Regex::new(r#"const main_origin\s*=\s*"([^"]*)";"#).unwrap();
    let re_apkey = Regex::new(r#"const apkey\s*=\s*"([^"]*)";"#).unwrap();
    let re_xxid = Regex::new(r#"const xxid\s*=\s*"([^"]*)";"#).unwrap();

    // TODO: Turn this stuff into a function
    // get link to embedding api
    let last_script = catflix_movie_document
        .select(&script_binding)
        .last()
        .map(|script| script.inner_html());
    let pre_juice_target_encoded = last_script
        .as_deref()
        .and_then(|script| re_main_origin.captures(script))
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap()
        .to_string();
    let embed_api_url = BASE64_STANDARD
        .decode(pre_juice_target_encoded.as_bytes())
        .unwrap()
        .iter()
        .map(|a| *a as char)
        .collect::<String>();

    // get apkey and xxid
    let embed_api_html_response = client
        .get(&embed_api_url)
        .send()
        .await
        .expect("Failed to receive response from embed api")
        .text()
        .await?;

    let embed_api_document = Html::parse_document(embed_api_html_response.as_str());
    let embed_script = embed_api_document
        .select(&script_binding)
        .into_iter()
        .nth_back(1)
        .map(|script| script.inner_html());
    let apkey = embed_script
        .as_deref()
        .and_then(|script| re_apkey.captures(script))
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap()
        .to_string();
    let xxid = embed_script
        .as_deref()
        .and_then(|script| re_xxid.captures(script))
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap()
        .to_string();

    // get juice key and decrypt
    //
    // NOTE: Should be done async with tokio and refactored/made into a function when done
    let juice_key_url = "https://turbovid.eu/api/cucked/juice_key";
    let referer = embed_api_url;

    let juice_key_json: Value = client
        .get(juice_key_url)
        .header("Referer", &referer)
        .send()
        .await?
        .json()
        .await?;

    // TODO: Turn this into a function
    let _ = helpers::check_json(&juice_key_json);
    let mut juice_key = juice_key_json.get("juice").unwrap().to_string();
    juice_key = juice_key.trim_matches('"').to_string();
    // get data from api and decrypt
    let juice_data_url = format!(
        "https://turbovid.eu/api/cucked/the_juice_v2/?{}={}",
        apkey, xxid
    );

    let juice_data_json = client
        .get(juice_data_url)
        .header("Referer", &referer)
        .send()
        .await?
        .json()
        .await?;
    let _ = helpers::check_json(&juice_data_json);
    let mut data_crypted = juice_data_json.get("data").unwrap().to_string();
    data_crypted = data_crypted.trim_matches('"').to_string();
    println!("data: {}, Key: {}", data_crypted, juice_key);
    return Ok(helpers::decrypt(data_crypted, juice_key).await);
    // NOTE: remember to change the service file for gpu-screenrecorder
}

async fn get_movie_search(call: String, header: String, client: Client) -> Value {
    return client
        .get(call)
        .header("Authorization", header)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}
