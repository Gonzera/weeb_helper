use reqwest::blocking::Response;

// make sure this does not fail later
pub fn ensure_path(path: &str) {
    // check if a folder with the given title exists
    // if not, create it
    if std::fs::metadata(path).is_ok() {
        return;
    } else {
        std::fs::create_dir(path).expect("Failed to create directoy");
    }
}

pub fn add_torrent(title: &str, download_url: &str, savepath: &str, url: &str) {
    let client = reqwest::blocking::Client::new();
    let url: String = format!("{}{}", url, "api/v2/torrents/add");
    println!("{}", url);
    let path: String = format!("{}{}", savepath, title);
    ensure_path(&path);
    let form_data = [
        ("urls", download_url),
        ("autoTMM", "false"),
        ("savepath", &path),
        ("category", "Anime-Seasonal"),
    ];

    let _: Response = match client.post(url).form(&form_data).send() {
        Ok(res) => res,
        Err(e) => {
            println!("error adding torrent: {:?}", e);
            return;
        }
    };
    // dbg!(res);
    println!("Downloading {}", title);
}
