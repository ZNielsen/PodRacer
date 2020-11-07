use crate::racer::*;

use std::io::{self, Read};
use std::fs::File;

pub fn get_params_cli() -> std::io::Result<()> {
    print!("What's the URL?: ");
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    println!("");
    println!("Got buffer: {}", buffer);
    let channel = match download_rss_channel(&buffer) {
        Ok(val) => val,
        Err(_) => panic!("Error in URL"),
    };
    let num_episodes = channel.items().len();
    let weeks_behind = get_time_behind(&channel);
    println!("There are {} episodes, and you are {} weeks behind.", num_episodes, weeks_behind);

    // Get the rate
    println!("How fast would you like to catch up?");
    print!("Enter a floating point number [defualt: 1.20]: ");
    let rate = match io::stdin().read_to_string(&mut buffer) {
        _ => 1.20,
    };

    println!("Do you still want to get new episodes in your feed as they arrive?");
    print!("[default: No]: ");
    let integrate_new = match io::stdin().read_to_string(&mut buffer) {
        _ => false,
    };

    Ok(())
}

pub fn create_feed(url: String, rate: f32, integrate_new: bool) -> FeedRacer {
    let channel = match download_rss_channel(&url) {
        Ok(val) => val,
        Err(_) => panic!("Error in URL"),
    };
    let num_episodes = channel.items().len();
    let weeks_behind = get_time_behind(&channel);
    // Make directory
    let dir = create_feed_racer_dir(&channel);
    // Write out original rss feed to file in dir
    let original_rss_file = File::create(String::from(&dir) + "/" + crate::racer::ORIGINAL_RSS_FILE).unwrap();
    channel.pretty_write_to(original_rss_file, crate::racer::SPACE_CHAR, 2).unwrap();
    // Make racer file
    let racer = FeedRacer::new(&mut channel.items().to_owned(), &rate, &channel.link(), &integrate_new);
    racer.write_to_file().unwrap();
    // Run update() on this directory
    racer_update(&dir).unwrap();
    // Give the user the url to subscribe to
    println!("Subscribe to this URL in your pod catcher: {}", "www.example.com");

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
    let mut dir = String::new();
    dir.push_str(crate::racer::PODRACER_DIR);
    dir.push_str("_");
    dir.push_str(ch.title());
    dir.push_str("_");
    let hash: Vec<u8> = md5::compute(ch.link()).to_vec();
    dir.push_str(std::str::from_utf8(hash.as_slice()).unwrap());
    // TODO - support multiple copies of the same feed
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn racer_update(path: &str) -> std::io::Result<()> {
    // Load in racer file
    let racer_file = File::open(String::from(path) +"/"+ crate::racer::RACER_FILE)?;
    let mut racer: FeedRacer = serde_json::from_reader(&racer_file)?;

    let original_rss = racer.get_updated_original_rss();
    // Check how many episodes we should publish at this point
    let num_to_pub = racer.get_num_to_publish();
    // Grab that many from the original rss file
    let mut items_to_publish = original_rss.items().to_owned();
    items_to_publish.truncate(num_to_pub);

    // Overwrite our racer.rss file, which includes the new content
    racer.update_published_items(&mut items_to_publish);
    racer.write_to_file();
    Ok(())
}
