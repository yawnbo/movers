// NOTE:
// this might be the worst code i've ever written but it's also my first time ever touching rust so
// whatever ig 03/05/2025

// TODO:
// -unwraps need to be changed to actual error handling especially in fzf function
// -better variable and function handling because this is a fucking mess
// -see tother TODO's in the code please

// work list order
// 1. subtitles - DONE
// 2. episodes and series - DONE (kinda this code is terrible plz rewrite before arg parsing)
// 3. clap arg parsing
// 4. mp4 packing with -d and subtitles

use std::env;
use std::error::Error;
mod helpers;
// TODO:
// mod loadconfig;
mod cflixscraping;
mod subtitles;

// trait for generic fzf calling
trait HasTitle {
    fn get_title(&self) -> String;
    // same note as get_overview for watch item
    fn get_overview(&self) -> String;
    fn get_id(&self) -> String;
}
//#[derive(Debug)]
struct WatchItem {
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
impl HasTitle for WatchItem {
    // needed for generic fzf calling but honestly not incredibly useful
    fn get_title(&self) -> String {
        self.title.clone()
    }
    // overview is useful for a verbose flag that gives an overview with each fzf selection for
    // later, not currently used
    fn get_overview(&self) -> String {
        self.overview.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

// this should've been done with as a child...................... im in too deep to change too
// TODO:
// rewrite everything..
//#[derive(Debug)]
struct Season {
    // option is needed because special seasons appear in tmdb that have no string with them so we
    // can have a none type.
    overview: Option<String>,
    number: usize,
    title: String,
    id: String,
    episode_count: usize,
    episodes: Option<Vec<Episode>>,
}
impl HasTitle for Season {
    fn get_title(&self) -> String {
        self.title.clone()
    }
    fn get_overview(&self) -> String {
        self.overview.clone().unwrap_or("".to_string())
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
//#[derive(Debug)]
struct Episode {
    overview: String,
    title: String,
    number: usize,
    id: String,
    // i thought this was needed for subtitles so it's here for now but is useless, i'll keep it
    // here for future use like the others
    imdb_id: String,
}
impl HasTitle for Episode {
    fn get_title(&self) -> String {
        self.title.clone()
    }
    fn get_overview(&self) -> String {
        self.overview.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
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
    //TODO:
    //
    // This needs to be loaded and options like fzf list format need to be included, ie. "Title *
    // tagline * release_date * overview" etc. whatever the user wants displayed to recognize what
    // they want to watch because things with the same name are confusing like severance.
    //
    //loadconfig::load_config().await?;

    for string in args.iter() {
        match string.as_str() {
            "version" | "v" | "-v" | "--version" => {
                println!("Version 0.1.3");
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
    Err("[ERROR] Unknown argument".into())
}
