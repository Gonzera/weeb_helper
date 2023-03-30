use reqwest::blocking::Client;
use reqwest::blocking::Response;
use serde_json::json;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

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

const NOTIFICATIONS_QUERY: &str = "
query {
	Page(page: 1, perPage: 100) {
		pageInfo {
			total
		}
		notifications(type: AIRING, resetNotificationCount: false) {
			... on AiringNotification {
				createdAt
				episode
				media {
					title {
						english
						romaji
					}
				}
			}
		}
	}
}";

fn should_download(created_at: i64) -> bool {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get current time")
        .as_secs() as i64;
    let one_day: i64 = 86_400;

    current_time - created_at <= one_day
}

pub fn get_new_releases(token: String) -> Option<Vec<Show>> {
    let client = Client::new();
    // Since this is the only graphql query that will be used in the entire project, I've decided
    // to not use any graphql librarys and instead just manually build the json from a str
    let json = json!({ "query": NOTIFICATIONS_QUERY });
    let formated_token = std::format!("Bearer {}", token);

    // Sending the request
    let res: Response = match client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", formated_token)
        .body(json.to_string())
        .send()
    {
        Ok(response) => response,
        Err(error) => {
            println!("get_new_releases failed with error: {}", error);
            return None;
        }
    };

    // Just checking the status code
    if res.status() != 200 {
        return None;
    }

    let response_json: Value = serde_json::from_str(&res.text().unwrap()).unwrap();
    let notifications = response_json["data"]["Page"]["notifications"]
        .as_array()
        .unwrap();
    let mut shows: Vec<Show> = Vec::new();
    for item in notifications {
        let show_title = Title {
            // Converting the title to a String, and removing the quotes
            english: item["media"]["title"]["english"]
                .to_string()
                .replace("\"", ""),
            romaji: item["media"]["title"]["romaji"]
                .to_string()
                .replace("\"", ""),
        };
        let created_at = item["createdAt"].clone();
        let should: bool;
        if created_at.is_i64() {
            should = should_download(created_at.as_i64().unwrap());
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
