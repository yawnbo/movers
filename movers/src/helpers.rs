use crate::Movie;
use fzf_wrapped::Fzf;
use serde_json::Value;
use std::error::Error;
// helper
pub async fn check_json(json: &Value) -> Result<(), Box<dyn Error>> {
    if json.get("success").unwrap() == "false" {
        return Err("Failed to fetch valid json!".into());
    } else {
        println!("Valid json recieved!");
        return Ok(());
    }
}

//helper
pub async fn decrypt(cyphertext: String, key: String) -> String {
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
// helper
pub async fn find_movie_id(
    selection: String,
    movie_list: &Vec<Movie>,
) -> Result<String, Box<dyn Error>> {
    return Ok(movie_list
        .iter()
        .position(|a| a.title == selection)
        .unwrap()
        .to_string());
}
// helper
pub async fn fzf_results(movie_list: &Vec<Movie>) -> Result<String, Box<dyn Error>> {
    // create vector of only titles and pass it through fzf for user to select
    // TODO:
    // append a load more element to allow for the second page to be loaded +1
    let movie_titles: Vec<String> = movie_list.iter().map(|a| a.title.clone()).collect();
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(movie_titles).expect("Failed to add items");
    let selection = fzf.output().unwrap();
    return Ok(find_movie_id(selection, &movie_list).await?);
}
// helper
pub async fn get_movie_details(current_id: &str, movie_details: Value) -> Movie {
    return Movie {
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
    };
}
