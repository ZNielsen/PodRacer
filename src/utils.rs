use crate::racer::*;

pub fn download_rss_channel(url: &str) -> Result<rss::Channel, Box<dyn std::error::Error>> {
    let content = reqwest::blocking::get(url).unwrap().bytes().unwrap();
    let channel = rss::Channel::read_from(&content[..])?;
    Ok(channel)
}

pub fn get_time_behind(channel: &rss::Channel) -> i64 {
    let published = channel.items().last().unwrap().pub_date().unwrap();
    let diff = chrono::DateTime::parse_from_rfc2822(published).unwrap()
                .signed_duration_since(chrono::Utc::now());
    diff.num_weeks()
}

pub fn create_feed_racer_dir(ch: &rss::Channel) -> String {
    // Create this feed's dir name
    let mut dir = String::from(dirs::home_dir().unwrap().to_str().unwrap());
    dir.push_str("/");
    dir.push_str(crate::racer::PODRACER_DIR);
    dir.push_str("/");
    dir.push_str(ch.title().to_lowercase().replace(" ", "-").as_str());
    dir.push_str("_0");
    // TODO - support multiple copies of the same feed
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn get_hostname_and_port() -> Option<String> {
    // std::process::Command::new("").output()
    Some(String::from("http://") + HOSTNAME + ":" + PORT)
}
