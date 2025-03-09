// NOTE:
// this might be the worst code i've ever written but it's also my first time ever touching rust so
// whatever ig 03/05/2025

// TODO:
// -unwraps need to be changed to actual error handling especially in fzf function
// -do i really need the whole movie struct? useful if i end up doing ui but like really?
// -better variable and function handling because this is a fucking mess
// -see tother TODO's in the code please
use std::env;
use std::error::Error;
use std::process::Command;

mod cflixscraping;
mod helpers;
// TODO:
// mod loadconfig;
mod subtitles;

// struct for series and episodes should also be made but that's an issue for another day.
#[derive(Debug)]
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
    imdb_id: String,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // TODO:
    // probably use clap for better parsing and make a config file for things when they are needed
    let args: Vec<String> = env::args().collect();
    let options: &str = "Unknown argument, options are: \nversion, -v, --version: print version \nsearch, 'search | -S | --search <QUERY>\nhelp, -h, --help: print available commands.";
    if args.len() == 1 {
        println!("{}", options);
        return Err("No arguments provided".into());
    }

    //loadconfig::load_config().await?;

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
                    let movie_list: Vec<Movie> = cflixscraping::init_client(search_term).await?;
                    // TODO:
                    // probably make a ui rendering but lowkey hard :(
                    let selected_id = helpers::fzf_results(&movie_list).await?;
                    println!(
                        "Found movie: {}",
                        movie_list[selected_id.parse::<usize>().unwrap()].id
                    );
                    let mpegts_url = cflixscraping::get_mpegts(format!(
                        "https://catflix.su/movie/{}",
                        movie_list[selected_id.parse::<usize>().unwrap()].id
                    ))
                    .await?;
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
