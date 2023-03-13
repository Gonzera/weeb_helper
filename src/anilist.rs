use reqwest::blocking::Client;
use reqwest::blocking::Response;
use serde_json::json;
use serde_json::Value;

#[derive(Debug)]
pub struct Title {
    pub english: String,
    pub romaji: String,
}

#[derive(Debug)]
pub struct Show {
    pub title: Title,
    pub should_download: bool,
}

const WATCHING_LIST_QUERY: &str = "
query ($user_name: String) {
    MediaListCollection (userName: $user_name, status_in: [CURRENT], type: ANIME) {
        lists {
            entries {
                media {
                    id
                    title {
                        english
                        romaji
                    }
                    status
                    season
                    nextAiringEpisode{
                        airingAt
                        timeUntilAiring
                    }
                }
            }
        }
    }
    }";

fn should_download(next: i64) -> bool {
    // six days in seconds
    let six_days = 6 * 24 * 60 * 60;
    // 22 hours in seconds
    let hours = 15 * 60 * 60;

    let time_spam = six_days + hours;
    // println!("next: {}, time_spam: {}", next, time_spam);

    if next >= time_spam {
        return true;
    }
    return false;
}

pub fn get_users_shows(user_name: String) -> Option<Vec<Show>> {
    let client = Client::new();
    // Since this is the only graphql query that will be used in the entire project, I've decided
    // to not use any graphql librarys and instead just manually build the json from a str
    let json = json!({"query": WATCHING_LIST_QUERY, "variables": {"user_name": user_name}});

    // Sending the request
    let res: Response = match client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
    {
        Ok(response) => response,
        Err(error) => {
            println!("get_user_list failed with error: {}", error);
            return None;
        }
    };

    // Just checking the status code
    if res.status() != 200 {
        return None;
    }

    let response_json: Value = serde_json::from_str(&res.text().unwrap()).unwrap();
    let entries = response_json["data"]["MediaListCollection"]["lists"][0]["entries"]
        .as_array()
        .unwrap();
    let mut shows: Vec<Show> = Vec::new();
    for item in entries {
        let show_title = Title {
            // Converting the title to a String, and removing the quotes
            english: item["media"]["title"]["english"]
                .to_string()
                .replace("\"", ""),
            romaji: item["media"]["title"]["romaji"]
                .to_string()
                .replace("\"", ""),
        };
        let next_ep_in = item["media"]["nextAiringEpisode"]["timeUntilAiring"].clone();
        let should: bool;
        if next_ep_in.is_i64() {
            should = should_download(next_ep_in.as_i64().unwrap());
        } else {
            should = false;
        }

        // Creating the show
        let show = Show {
            title: show_title,
            should_download: should,
        };
        // Adding the show to the vec
        shows.push(show);
    }
    Some(shows)
}
