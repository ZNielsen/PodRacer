use std::io::{self, Read, Write, BufReader};
use std::fs::File;
use serde::{Serialize, Deserialize};
use chrono::{Duration, DateTime};

const ORIGINAL_RSS_FILE: &'static str = "original.rss";
const RACER_FILE:        &'static str = "racer.file";
const SCHEMA_VERSION:    &'static str = "1.0.0";
const PODRACER_DIR:      &'static str = "~/.podracer";
const SPACE_CHAR: u8 = 32;

// JSON keys
const KEY_EPISODE_RACER_DATE: &'static str = "date_string";
const KEY_SCHEMA_VERSION:     &'static str = "schema_version";
const KEY_EPISODE_NUMBER:     &'static str = "ep_num";
const KEY_RELEASE_DATES:      &'static str = "release_dates";
const KEY_SOURCE_URL:         &'static str = "source_url";
const KEY_RACER_PATH:         &'static str = "racer_path";
const KEY_RACER_URL:          &'static str = "podracer_url";

#[derive(Serialize, Deserialize, Debug)]
struct RacerEpisode {
    ep_num: i64,
    date: String
}

#[derive(Serialize, Deserialize, Debug)]
struct RacerData {
    schema_version: String,
    racer_path: String,
    source_url: String,
    podracer_url: String,
    anchor_date: DateTime<chrono::Utc>,
    first_pubdate: DateTime<chrono::FixedOffset>,
    rate: f32,
    release_dates: Vec<RacerEpisode>
}

impl RacerData {
    fn add_new_items(&mut self, items: &Vec<rss::Item>, current_len: usize)
    {
        let mut item_counter = (current_len-1) as i64;
        for item in items {
            // Get diff from first published date
            let pub_date = item.pub_date().unwrap();
            let mut time_diff = DateTime::parse_from_rfc2822(pub_date).unwrap()
                            .signed_duration_since(self.first_pubdate)
                            .num_seconds();
            // Scale that diff
            time_diff = ((time_diff as f32) / self.rate) as i64;
            // Add back to anchor date to get new publish date + convert to string
            let racer_date = self.anchor_date
                                .checked_add_signed(Duration::seconds(time_diff)).unwrap()
                                .to_rfc2822();
            // Add to vector of dates
            self.release_dates.push( RacerEpisode {
                ep_num: item_counter,
                date: racer_date
            });
            item_counter += 1;
        }
    }

    // TODO -> handle the case where the url is invalid/disappears
    // TODO -> handle feeds that have a constant number of entries
    //         and push the oldest entry out
    fn update_if_needed(&mut self)
    {
        // Re-download file
        let original_rss = get_rss_channel(&self.source_url).unwrap();
        // Compare
        let mut stored_rss_file = File::open(String::from(&self.racer_path) +"/"+ ORIGINAL_RSS_FILE).unwrap();
        let mut buf_reader = BufReader::new(stored_rss_file);
        let stored_rss = rss::Channel::read_from(buf_reader).unwrap();
        let num_to_update = (original_rss.items().len() as i64 - stored_rss.items().len() as i64).abs();
        if num_to_update > 0 {
            // Overwrite our stored original RSS file
            let original_rss_file = File::create(String::from(&self.racer_path) +"/"+ ORIGINAL_RSS_FILE).unwrap();
            original_rss.pretty_write_to(original_rss_file, SPACE_CHAR, 2).unwrap();
            // Append new entries to our racer object
            let mut new_items = original_rss.items().to_owned();
            new_items.truncate(num_to_update as usize);
            self.add_new_items(&new_items, stored_rss.items().len());
        }
    }

    fn get_num_to_publish(&self) -> usize
    {
        let mut ret = 0;
        // Get today's date
        let now = chrono::Utc::now();
        // Count how many are before todays dates
        for release_date in &self.release_dates {
            let date = chrono::DateTime::parse_from_rfc2822(&release_date.date).unwrap();
            if date.signed_duration_since(now) < chrono::Duration::zero() {
                ret += 1;
            }
        }
        ret
    }
}

fn main() -> std::io::Result<()> {
    print!("What's the URL?: ");
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    println!("");
    println!("Got buffer: {}", buffer);
    let mut channel = match get_rss_channel(&buffer) {
        Ok(val) => val,
        Err(_) => panic!("Error in URL"),
    };
    let num_episodes = channel.items().len();
    let weeks_behind = get_time_behind(&channel);
    println!("There are {} episodes, and you are {} weeks behind.",
        num_episodes, weeks_behind);

    // Get the rate
    println!("How fast would you like to catch up?");
    print!("Enter a floating point number [defualt: 1.20]: ");
    let rate = match io::stdin().read_to_string(&mut buffer) {
        _ => 1.20,
    };

    println!("Do you still want to get new episodes in your feed as they arrive?");
    print!("[default: No]: ");
    let integrate_new = match io::stdin().read_to_string(&mut buffer) {
        _ => true,
    };

    // Make directory
    let dir = create_feed_racer_dir(&channel);
    // Write out original rss feed to file in dir
    let original_rss_file = File::create(String::from(&dir) +"/"+ ORIGINAL_RSS_FILE)?;
    channel.pretty_write_to(original_rss_file, SPACE_CHAR, 2).unwrap();
    // Make racer file
    create_racer_file(&channel, &rate)?;
    // Run update() on this directory
    racer_update(&dir);
    // Give the user the url to subscribe to
    Ok(())
}

fn get_rss_channel(url: &String) -> Result<rss::Channel, Box<dyn std::error::Error>>
{
    println!("Getting content");
    let content = reqwest::blocking::get(url).unwrap()
                    .bytes().unwrap();
    println!("Got content");
    let channel = rss::Channel::read_from(&content[..])?;
    println!("Got channel");
    Ok(channel)
}

fn get_time_behind(channel: &rss::Channel) -> i64
{
    let published = channel.items()
                    .last().unwrap()
                    .pub_date().unwrap();
    let diff = chrono::DateTime::parse_from_rfc2822(published).unwrap()
                .signed_duration_since(chrono::Utc::now());
    diff.num_weeks()
}

fn create_feed_racer_dir(ch: &rss::Channel) -> String
{
    // Create this feed's dir name
    let mut dir = String::new();
    dir.push_str(PODRACER_DIR);
    dir.push_str("_");
    dir.push_str(ch.title());
    dir.push_str("_");
    let hash: Vec<u8> = md5::compute(ch.link()).to_vec();
    dir.push_str(std::str::from_utf8(hash.as_slice()).unwrap());
    // TODO - support multiple copies of the same feed
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn create_racer_file(ch: &rss::Channel, rate: &f32) -> std::io::Result<()>
{
    // Reverse the items so the oldest entry is first
    let mut items = ch.items().to_owned();
    items.reverse();
    // Get anchor date
    let first_pubdate = items.first().unwrap()
                            .pub_date().unwrap();
    let first_pubdate = DateTime::parse_from_rfc2822(first_pubdate).unwrap();
    let anchor_date = chrono::Utc::now();
    let mut dates = Vec::new();
    let mut item_counter = 0;
    for item in items {
        // Get diff from first published date
        let pub_date = item.pub_date().unwrap();
        let mut time_diff = DateTime::parse_from_rfc2822(pub_date).unwrap()
                            .signed_duration_since(first_pubdate)
                            .num_seconds();
        // Scale that diff
        time_diff = ((time_diff as f32) / rate) as i64;
        // Add back to anchor date to get new publish date + convert to string
        let racer_date = anchor_date.checked_add_signed(Duration::seconds(time_diff)).unwrap()
                            .to_rfc2822();
        // Add to vector of dates
        dates.push( RacerEpisode {
            ep_num: item_counter,
            date: racer_date
        });
        item_counter += 1;
    }

    let racer_data = RacerData {
        schema_version: SCHEMA_VERSION.to_owned(),
        racer_path: PODRACER_DIR.to_owned(),
        source_url: ch.link().to_owned(),
        podracer_url: "xxx".to_owned(),
        rate: rate.to_owned(),
        anchor_date: anchor_date,
        first_pubdate: first_pubdate,
        release_dates: dates
    };
    let json = serde_json::to_string_pretty(&racer_data)?;

    let filename = String::from(PODRACER_DIR) +"/"+ RACER_FILE;
    let mut fp = File::create(filename)?;
    fp.write_all(json.as_bytes())
}

fn racer_update(path: &str) -> std::io::Result<()>
{
    // Load in racer file
    let mut racer_file = File::open(String::from(path) +"/"+ RACER_FILE)?;
    let mut racer: RacerData = serde_json::from_reader(&racer_file)?;

    // Check original rss feed for update, if required
    racer.update_if_needed();

    // Check how many episodes we should publish at this point
    let num_to_pub = racer.get_num_to_publish();
    // Grab that many from the original rss file
    // Overwrite our racer.rss file, which includes the new content
    Ok(())
}
