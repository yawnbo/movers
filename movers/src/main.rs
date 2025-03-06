// NOTE:
// this might be the worst code i've ever written but it's also my first time ever touching rust so
// whatever ig 03/05/2025
use fzf_wrapped::Fzf;
use serde_json::Value;
use std::env;
use std::error::Error;
// struct for series and episodes should also be made but that's an issue for another day.
//#[derive(Debug)]
struct Movie {
    title: String,
    release_date: String,
    tagline: String,
    overview: String,
    poster_path: String,
    average_rating: f64,
    id: String,
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
                // hardcoded to the second term and should be changed later
                if let Some(search_term) = args.iter().nth(2) {
                    let movie_list: Vec<Movie> = init_client(search_term)?;
                    // TODO:
                    // probably make a ui rendering but lowkey hard :(
                    let selected_id = fzf_results(&movie_list).unwrap();
                    let mpegts_url = get_mpegts(format!(
                        "https://catflix.su/movie/{}",
                        movie_list[selected_id.parse::<usize>().unwrap()].id
                    ));
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
        let api_auth_header = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiIyOTIzODEzYzIwNWUzZDRjNGY4ZGVhNmFjZTQ2YTMwMiIsIm5iZiI6MTcyMzA1MTM1Mi44MjY2NTEsInN1YiI6IjY2YjNhYzM2YjMwNGY1Nzg1Y2UxODQwYyIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.Nx0oSk9Ts8LurGRrSZk5b-QE172zZ_dCLNT9WJJFLbc";
        let api_base_url = "https://api.themoviedb.org/3/movie/";
        println!("Fetching movie details from tmdb api...");
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
                average_rating: movie_details.get("vote_average").unwrap().as_f64().unwrap(),
                id: current_id,
            };
            movie_results.push(temp_movie);
        }
        println!("tmdb api response OK")
    }
    return Ok(movie_results);
}

fn fzf_results(movie_list: &Vec<Movie>) -> Result<String, Box<dyn Error>> {
    // create vector of only titles and pass it through fzf for user to select
    // TODO:
    // append a load more element to allow for the second page to be loaded
    let movie_titles: Vec<String> = movie_list.iter().map(|a| a.title.clone()).collect();
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(movie_titles).expect("Failed to add items");
    let selection = fzf.output().unwrap();
    return Ok(find_movie_id(selection, &movie_list)?);
}

fn find_movie_id(selection: String, movie_list: &Vec<Movie>) -> Result<String, Box<dyn Error>> {
    return Ok(movie_list
        .iter()
        .position(|a| a.title == selection)
        .unwrap()
        .to_string());
}

fn get_mpegts(url: String) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let mpegts_url = client
        .get(url)
        .send()
        .expect("Failed to fetch mpegts url")
        .url()
        .to_string();
    println!("Mpegts url: {}", mpegts_url);
    return Ok(());
}
