use crate::racer::*;

use std::fs::File;

pub fn create_feed(url: String, rate: f32, integrate_new: bool) -> FeedRacer {
    let channel = match download_rss_channel(&url) {
        Ok(val) => val,
        Err(_) => panic!("Error in URL"),
    };
    // let num_episodes = channel.items().len();
    // let weeks_behind = get_time_behind(&channel);
    // Make directory
    let dir = create_feed_racer_dir(&channel);
    // Write out original rss feed to file in dir
    let original_rss_file = File::create(String::from(&dir) + "/" + crate::racer::ORIGINAL_RSS_FILE).unwrap();
    channel.pretty_write_to(original_rss_file, crate::racer::SPACE_CHAR, 2).unwrap();
    // Make racer file
    let racer = FeedRacer::new(
                    &mut channel.items().to_owned(),
                    &rate,
                    &url,
                    &integrate_new,
                    &dir);
    match racer.write_to_file() {
        Ok(_) => (),
        Err(e) => panic!("failed with error: {}", e)
    }
    // Run update() on this directory. We just created it, so no need to refresh the rss file
    update_racer_at_path(&dir, &RssFile::FromStorage).unwrap();
    // Give the user the url to subscribe to
    println!("Subscribe to this URL in your pod catcher: {}", racer.get_podracer_url());

    racer
}

pub fn download_rss_channel(url: &String) -> Result<rss::Channel, Box<dyn std::error::Error>> {
    println!("Getting content");
    let content = reqwest::blocking::get(url).unwrap().bytes().unwrap();
    println!("Got content");
    let channel = rss::Channel::read_from(&content[..])?;
    println!("Got channel");
    Ok(channel)
}

pub fn get_time_behind(channel: &rss::Channel) -> i64 {
    let published = channel.items().last().unwrap().pub_date().unwrap();
    let diff = chrono::DateTime::parse_from_rfc2822(published).unwrap()
                .signed_duration_since(chrono::Utc::now());
    diff.num_weeks()
}

fn create_feed_racer_dir(ch: &rss::Channel) -> String {
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
