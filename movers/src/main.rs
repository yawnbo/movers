// NOTE:
// this might be the worst code i've ever written but it's also my first time ever touching rust so
// whatever ig 03/05/2025

// TODO:
// -unwraps need to be changed to actual error handling especially in fzf function
// -do i really need the whole movie struct? useful if i end up doing ui but like really?
// -better variable and function handling because this is a fucking mess
// -see tother TODO's in the code please

// work list order
// 1. subtitles - DONE
// 2. episodes and series
// 3. clap arg parsing
// 4. mp4 packing with -d and subtitles

use std::env;
use std::error::Error;
mod helpers;
// TODO:
// mod loadconfig;
mod cflixscraping;
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
    series: bool,
    seasons: Option<Vec<Season>>,
}
#[derive(Debug)]
struct Season {
    episode_number: i32,
    episode_name: String,
    episode_id: String,
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
                println!("Version 0.1.2");
                return Ok(());
            }
            "help" | "-h" | "--help" => {
                println!("{}", options);
                return Ok(());
            }
            "search" | "-S" | "--search" => {
                match helpers::search_and_play(&args).await {
                    Ok(()) => {
                        return Ok(());
                    }
                    Err(e) => {
                        eprint!("Erorr: {}", e);
                        if let Err(clean_err) = helpers::clean_subtitle_cache().await {
                            eprint!("Error cleaning subtitle cache: {}", clean_err);
                        }
                    }
                }
                return Ok(());
            }
            _ => {
                continue;
            }
        }
    }
    println!("{}", options);
    Err("Unknown argument".into())
}
