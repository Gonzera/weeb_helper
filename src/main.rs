mod anilist;
mod torrent;
mod tracker;

extern crate anitomy;
extern crate daemonize;

use anilist::Show;
use clap::Parser;
use daemonize::Daemonize;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use tracker::TrackerItem;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Initialize configuration files
    #[arg(short, long, default_value_t = false)]
    init: bool,

    /// Run weeb_helper on the background
    #[arg(short, long, default_value_t = false)]
    daemon: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    qbittorrent_url: String,
    torrent_category: String,
    torrent_savepath: String,
    anilist_token: String,
}

fn main() {
    let config_path = get_config_path();
    let args = Args::parse();
    if !config_exists(&config_path) && !args.init {
        println!("No configuration files were found, if this is your first time running weeb_helper use -i");
        return;
    }
    if args.init {
        init_config(&config_path);
        return;
    }
    if args.daemon {
        let config: Config = read_config(&config_path);
        let stdout = File::create("/tmp/weeb_helper.out").unwrap();
        let stderr = File::create("/tmp/weeb_helper.err").unwrap();
        let daemonize = Daemonize::new()
            .working_directory(&config_path)
            .pid_file("/tmp/weeb_helper.pid")
            .stdout(stdout)
            .stderr(stderr);
        match daemonize.start() {
            Ok(_) => println!("Daemon started"),
            Err(e) => {
                eprintln!("Error starting daemon: {}", e);
                return;
            }
        }
        check_list(&config);
    } else {
        // idk runs it
        let config: Config = read_config(&config_path);
        check_list(&config);
    }
}

fn config_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

fn get_config_path() -> String {
    let home = std::env::var_os("HOME").expect("Failed to get HOME path");
    let path = std::path::PathBuf::from(home);
    format!("{}/.config/weeb_helper/", path.to_str().unwrap())
}

fn check_list(config: &Config) {
    // Checks the list for new releases in a endless loop
    let dur = std::time::Duration::from_secs(5 * 60);
    loop {
        let currently_watching = anilist::get_new_releases(config.anilist_token.to_owned());
        if currently_watching.is_some() {
            match tracker::get_feed_items() {
                Ok(nyaa_feed) => {
                    for c in currently_watching.unwrap() {
                        try_download(
                            &nyaa_feed,
                            &c,
                            config.qbittorrent_url.as_str(),
                            config.torrent_savepath.as_str(),
                        );
                    }
                }
                Err(_) => (),
            }
        }
        std::thread::sleep(dur);
    }
}

fn read_config(path: &str) -> Config {
    let config_path = format!("{}config.json", path);
    let mut file = std::fs::File::open(config_path).expect("failed to read config");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("failed to read config content");
    let config: Config = serde_json::from_str(&contents).unwrap();

    config
}

fn init_config(path: &str) {
    let mut qbittorrent_url = String::new();
    let mut torrent_category = String::new();
    let mut torrent_savepath = String::new();
    let mut anilist_token = String::new();

    println!("URL for the qbittorrent web ui (Example: http://localhost:8080/):");
    std::io::stdin()
        .read_line(&mut qbittorrent_url)
        .expect("Failed to read input");

    println!("Enter the torrent caregory (leave empty for nothing):");
    std::io::stdin()
        .read_line(&mut torrent_category)
        .expect("Failed to read input");

    println!("Enter the full path for downloads (keep in mind that a subdirectory will be created for each show):");
    std::io::stdin()
        .read_line(&mut torrent_savepath)
        .expect("Failed to read input");

    println!("Enter your anilist token:");
    std::io::stdin()
        .read_line(&mut anilist_token)
        .expect("Failed to read input");

    let config = Config {
        qbittorrent_url: qbittorrent_url.trim().to_owned(),
        torrent_category: torrent_category.trim().to_owned(),
        torrent_savepath: torrent_savepath.trim().to_owned(),
        anilist_token: anilist_token.trim().to_owned(),
    };

    let json = serde_json::to_string(&config).unwrap();

    let config_path = format!("{}config.json", path);
    torrent::ensure_path(&path);
    std::fs::write(&config_path, json).expect("Failed to write config");

    println!("Done! You can edit your settings later at .config/weeb_helper");
}

fn try_download(items: &Vec<TrackerItem>, show: &Show, url: &str, savepath: &str) {
    for item in items {
        if item.parsed_title.is_some() && item.parsed_episode.is_some() {
            let p_title: &str = &item.parsed_title.as_ref().unwrap();
            let p_ep: &str = &item.parsed_episode.as_ref().unwrap();
            if &show.title.english == p_title || &show.title.romaji == p_title {
                if show.should_download
                    && !already_downloaded(
                        &show.title.romaji,
                        item.parsed_episode.as_ref().unwrap(),
                    )
                {
                    torrent::add_torrent(&show.title.romaji, &item.url, savepath, url);
                    add_to_downloaded_list(&show.title.romaji, p_ep);
                }
            }
        }
    }
}

fn already_downloaded(title: &str, ep: &str) -> bool {
    let file = match File::open("downloaded_list") {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "Failed to open download list (normally this can be ignored): {}",
                e
            );
            return false;
        }
    };
    let reader = BufReader::new(file);

    for l in reader.lines() {
        let line = l.unwrap();
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() == 2 && fields[0] == title && fields[1] == ep {
            return true;
        }
    }

    return false;
}

fn add_to_downloaded_list(title: &str, ep: &str) {
    let file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open("downloaded_list")
        .unwrap();
    writeln!(&file, "{}|{}", title, ep).unwrap();
}
