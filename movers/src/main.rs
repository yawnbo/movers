use scraper::Html;
use serde_json::Value;
use std::env;
use std::error::Error;

struct Movie {
    title: String,
    release_date: String,
    tagline: String,
    overview: String,
    poster_path: String,
    average_rating: f32,
}
fn main() -> Result<(), Box<dyn Error>> {
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
                // hardcoded and should be changed later
                if let Some(search_term) = args.iter().nth(2) {
                    init_client(search_term);
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
fn init_client(search: &str) -> i32 {
    let mut movie_results: Vec<String> = Vec::new();
    let client = reqwest::blocking::Client::new();
    let base_url: &str = "https://catflix.su/";
    let json_query_url = format!(
        "{}api/autocomplete/?q={}&page=1&route=search&sid=0&context=all",
        base_url, search
    );
    let json_response: Value = client
        .get(json_query_url)
        .send()
        .expect("Failed to recieve JSON")
        .json()
        .unwrap();
    if let Some(data_array) = json_response
        .get("data")
        .and_then(|a| a.get("tmdb_id"))
        .and_then(|a| a.as_array())
    {
        // constatn api key???? No requests were made to get it so it's probably hardcoded
        // somewhere but might be subject to change I don't feel like finding out because it's
        // working ....
        let api_auth_header = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiIyOTIzODEzYzIwNWUzZDRjNGY4ZGVhNmFjZTQ2YTMwMiIsIm5iZiI6MTcyMzA1MTM1Mi44MjY2NTEsInN1YiI6IjY2YjNhYzM2YjMwNGY1Nzg1Y2UxODQwYyIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.Nx0oSk9Ts8LurGRrSZk5b-QE172zZ_dCLNT9WJJFLbc";
        let api_base_url = "https://api.themoviedb.org/3/movie/";
        for ids in data_array {
            // call movie db with bearer header here and add it another list of movie string
            // strings. Optionally, make a ui renderer that dynamically changes to give the
            // description.
            //
            // Also optionally, see the difference for looking at series and episodes instead of
            // movies. I'm not sure how the search querying is different.
            //
            // Anyway, call for each id should be like curl -X GET "https://api.themoviedb.org/3/movie/<MOVIE_ID>&language=en" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiIyOTIzODEzYzIwNWUzZDRjNGY4ZGVhNmFjZTQ2YTMwMiIsIm5iZiI6MTcyMzA1MTM1Mi44MjY2NTEsInN1YiI6IjY2YjNhYzM2YjMwNGY1Nzg1Y2UxODQwYyIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.Nx0oSk9Ts8LurGRrSZk5b-QE172zZ_dCLNT9WJJFLbc" -H "accept: application/json"
        }
    }
    return 0;
}
