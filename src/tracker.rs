use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::blocking::Response;
extern crate anitomy;
use anitomy::{Anitomy, ElementCategory};

#[derive(Debug)]
pub struct TrackerItem {
    pub title: String,
    pub url: String,
    pub parsed_title: Option<String>,
    pub parsed_episode: Option<String>,
}

impl TrackerItem {
    pub fn new(title: String, url: String) -> TrackerItem {
        let mut t = TrackerItem {
            title,
            url,
            parsed_title: None,
            parsed_episode: None,
        };
        t.parse();

        t
    }
    fn parse(&mut self) {
        let mut anitomy = Anitomy::new();
        match anitomy.parse(&self.title) {
            Ok(ref elements) => {
                // Maybe we should match here?
                self.parsed_title = Some(
                    elements
                        .get(ElementCategory::AnimeTitle)
                        .expect("Title")
                        .to_string(),
                );
                self.parsed_episode = match elements.get(ElementCategory::EpisodeNumber) {
                    Some(e) => Some(e.to_string()),
                    None => None,
                }
            }
            Err(ref _elements) => {
                self.parsed_title = None;
                self.parsed_episode = None;
            }
        }
    }
}

fn get_tracker_feed() -> Result<String, &'static str> {
    let client = reqwest::blocking::Client::new();

    let res: Response = match client
        .get("https://nyaa.si/?page=rss&q=1080p&c=1_2&f=0")
        .send()
    {
        Ok(response) => response,
        Err(_) => return Err("Failed to get response from tracker"),
    };
    let res_text = match res.text() {
        Ok(text) => text,
        Err(_) => return Err("Failed to get text from response"),
    };

    Ok(res_text)
}

// TODO: Refactor this
pub fn get_feed_items() -> Result<Vec<TrackerItem>, &'static str> {
    let feed: String = match get_tracker_feed() {
        Ok(feed) => feed,
        Err(_) => return Err("Failed to get feed from tracker"),
    };

    let mut items: Vec<TrackerItem> = Vec::new();
    let mut reader = Reader::from_str(&feed);
    reader.trim_text(true);

    let mut buffer = Vec::new();
    let mut title: String = String::new();
    let mut link: String = String::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Err(_) => return Err("Error reading buffer"),
            Ok(Event::Eof) => {
                break;
            }
            // Here we check for the begging of each item
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"item" => loop {
                match reader.read_event_into(&mut buffer) {
                    Ok(Event::Start(ref e)) if e.name().as_ref() == b"title" => {
                        title = reader.read_text(e.name()).unwrap().to_string();
                        // dbg!(title);
                    }
                    Ok(Event::Start(ref e)) if e.name().as_ref() == b"link" => {
                        link = reader.read_text(e.name()).unwrap().to_string();
                        // dbg!(link);
                    }
                    Ok(Event::End(ref e)) if e.name().as_ref() == b"item" => break,
                    _ => (),
                }
                // Creating the tracker item
            },
            _ => (),
        }
        if title.len() >= 1 && link.len() >= 1 {
            items.push(TrackerItem::new(title.clone(), link.clone()));
        }

        //Clearing the string so we can check if we got something later
        title.clear();
        link.clear();
        buffer.clear();
    }
    return Ok(items);
}
