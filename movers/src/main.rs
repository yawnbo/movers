// NOTE:
// this might be the worst code i've ever written but it's also my first time ever touching rust so
// whatever ig 03/05/2025

// TODO:
// -unwraps need to be changed to actual error handling especially in fzf function
// -do i really need the whole movie struct? useful if i end up doing ui but like really?
// -better variable and function handling because this is a fucking mess
// -see tother TODO's in the code please
use base64::prelude::*;
use fzf_wrapped::Fzf;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use std::env;
use std::error::Error;
use std::process::Command;

// struct for series and episodes should also be made but that's an issue for another day.
//#[derive(Debug)]
struct Movie {
    // NOTE:
    // 90% of this data is not **CURRENTLY** used but i plan to implement it someday (maybe) so i'm
    // leaving this here
    title: String,
    release_date: String,
    tagline: String,
    overview: String,
    poster_path: String,
    average_rating: f64,
    id: String,
}
fn main() -> Result<(), Box<dyn Error>> {
    // TODO:
    // probably use clap for better parsing and make a config file for things when they are needed
    let args: Vec<String> = env::args().collect();
    let options: &str = "Unknown argument, options are: \nversion, -v, --version: print version \nsearch, 'search | -S | --search <QUERY>\nhelp, -h, --help: print available commands.";
    if args.len() == 1 {
        println!("{}", options);
        return Err("No arguments provided".into());
    }

    for string in args.iter() {
        match string.as_str() {
            "version" | "v" | "-v" | "--version" => {
                println!("Version 0.0.1 Alpha testing");
                return Ok(());
            }
            "help" | "-h" | "--help" => {
                println!("{}", options);
                return Ok(());
            }
            "search" | "-S" | "--search" => {
                // hardcoded to the second term and should be changed later
                if let Some(search_term) = args.iter().nth(2) {
                    let movie_list: Vec<Movie> = init_client(search_term)?;
                    // TODO:
                    // probably make a ui rendering but lowkey hard :(
                    let selected_id = fzf_results(&movie_list).unwrap();
                    println!(
                        "Found movie: {}",
                        movie_list[selected_id.parse::<usize>().unwrap()].id
                    );
                    let mpegts_url = get_mpegts(format!(
                        "https://catflix.su/movie/{}",
                        movie_list[selected_id.parse::<usize>().unwrap()].id
                    ))
                    .unwrap();
                    // TODO: Make config parsing so mpv can also be chosen (include a bflix
                    // searcher too later)
                    let status = Command::new("iina")
                        .arg(mpegts_url)
                        .status()
                        .expect("Failed to start mpv");
                    if status.success() {
                        println!("So was the movie good :)");
                    } else {
                        println!("Mpv not happy :(");
                    }
                    return Ok(());
                } else {
                    return Err("Missing search term, Ex. movers search <SEARCH>".into());
                }
            }
            _ => {
                continue;
            }
        }
    }
    println!("{}", options);
    Err("Unknown argument".into())
}
fn init_client(search: &str) -> Result<Vec<Movie>, Box<dyn Error>> {
    let mut movie_results: Vec<Movie> = Vec::new();
    let client = reqwest::blocking::Client::new();
    let base_url: &str = "https://catflix.su/";
    let json_query_url = format!(
        "{}api/autocomplete/?q={}&page=1&route=search&sid=0&context=all",
        base_url, search
    );
    println!("Client initialized, fetching json from {}", json_query_url);
    let json_response: Value = client
        .get(json_query_url)
        .send()
        .expect("Failed to recieve JSON")
        .json()
        .unwrap();
    if json_response.get("success").unwrap() == "false" {
        println!("Catflix api error fetching valid JSON movie list.");
        return Err("failed to fetch catflix JSON".into());
    } else {
        println!("Catflix api returned valid JSON movie list.");
    }
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
            let movie_details: Value = client
                .get(tmdb_api_call)
                .header("Authorization", api_auth_header)
                .send()
                .and_then(|response| response.json())
                .unwrap();
            let temp_movie: Movie = Movie {
                title: get_movie_details("original_title", &movie_details),
                release_date: get_movie_details("release_date", &movie_details),
                tagline: get_movie_details("tagline", &movie_details),
                overview: get_movie_details("overview", &movie_details),
                poster_path: get_movie_details("poster_path", &movie_details),
                average_rating: movie_details.get("vote_average").unwrap().as_f64().unwrap(),
                id: current_id,
            };
            movie_results.push(temp_movie);
        }
        println!("tmdb api response OK")
    }
    return Ok(movie_results);
}

// TODO: holy function hell please organize this into files

// helper
fn get_movie_details(target: &str, movie_details: &Value) -> String {
    return movie_details
        .get(target)
        .unwrap()
        .to_string()
        .trim_matches('"')
        .to_string();
}

// helper
fn fzf_results(movie_list: &Vec<Movie>) -> Result<String, Box<dyn Error>> {
    // create vector of only titles and pass it through fzf for user to select
    // TODO:
    // append a load more element to allow for the second page to be loaded +1
    let movie_titles: Vec<String> = movie_list.iter().map(|a| a.title.clone()).collect();
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(movie_titles).expect("Failed to add items");
    let selection = fzf.output().unwrap();
    return Ok(find_movie_id(selection, &movie_list)?);
}

// helper
fn find_movie_id(selection: String, movie_list: &Vec<Movie>) -> Result<String, Box<dyn Error>> {
    return Ok(movie_list
        .iter()
        .position(|a| a.title == selection)
        .unwrap()
        .to_string());
}

fn get_mpegts(catflix_movie_url: String) -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let catflix_movie_html_response = client
        .get(catflix_movie_url)
        .send()
        .expect("Failed to fetch valid html from catflix movie url")
        .text()
        .unwrap();

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
        .expect("Failed to receive response from embed api")
        .text()
        .unwrap();

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
        .and_then(|r| r.json())
        .unwrap();

    // TODO: Turn this into a function
    let _ = check_json(&juice_key_json);
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
        .and_then(|r| r.json())
        .unwrap();
    let _ = check_json(&juice_data_json);
    let mut data_crypted = juice_data_json.get("data").unwrap().to_string();
    data_crypted = data_crypted.trim_matches('"').to_string();
    println!("data: {}, Key: {}", data_crypted, juice_key);
    return Ok(decrypt(data_crypted, juice_key));
    // NOTE: remember to change the service file for gpu-screenrecorder
}

// helper
fn check_json(json: &Value) -> Result<(), Box<dyn Error>> {
    if json.get("success").unwrap() == "false" {
        return Err("Failed to fetch valid json!".into());
    } else {
        println!("Valid json recieved!");
        return Ok(());
    }
}
fn decrypt(cyphertext: String, key: String) -> String {
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
