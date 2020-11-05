use std::io::{self, Read, Write};
use std::fs::File;
use serde::{Serialize, Deserialize};
use serde_json::json;
use chrono::{Duration, DateTime};

const SCHEMA_VERSION:  &'static str = "1.0.0";
const PODRACER_DIR:    &'static str = "~/.podracer";
const RACER_FILE_NAME: &'static str = "racer.file";


const SPACE_CHAR: u8 = 32;

// JSON keys
const KEY_SCHEMA_VERSION: &'static str = "schema_version";
const KEY_SOURCE_URL: &'static str = "source_url";
const KEY_RACER_URL: &'static str = "podracer_url";
const KEY_RELEASE_DATES: &'static str = "release_dates";
const KEY_EPISODE_NUMBER: &'static str = "ep_num";
const KEY_EPISODE_RACER_DATE: &'static str = "datestring";

#[derive(Serialize, Deserialize, Debug)]
struct RacerEpisode {
    ep_num: i64,
    date: String
}

#[derive(Serialize, Deserialize, Debug)]
struct RacerData {
    schema_version: String,
    source_url: String,
    podracer_url: String,
    release_dates: Vec<RacerEpisode>
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
    let file = File::create(dir+"/original.rss")?;
    channel.pretty_write_to(file, SPACE_CHAR, 2).unwrap();
    // Make racer file
    create_racer_file(&channel, &rate)?;
    // Run update() on this directory
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
    let first_pub_date = items.first().unwrap()
                            .pub_date().unwrap();
    let anchor_date = DateTime::parse_from_rfc2822(&first_pub_date).unwrap();
    let mut dates = Vec::new();
    let mut item_counter = 0;
    for item in items {
        // Get diff from anchor date
        let pub_date = item.pub_date().unwrap();
        let mut time_diff = DateTime::parse_from_rfc2822(pub_date).unwrap()
                        .signed_duration_since(anchor_date)
                        .num_seconds();
        // Scale that diff
        time_diff = ((time_diff as f32) / rate) as i64;
        // Add back to anchor date to get new publish date + convert to string
        let racer_date = anchor_date.checked_add_signed(Duration::seconds(time_diff)).unwrap()
                            .to_rfc2822();
        // Add to json
        dates.push( RacerEpisode {
            ep_num: item_counter,
            date: racer_date
         });
         item_counter += 1;
    }

    let racer_data = RacerData {
        schema_version: SCHEMA_VERSION.to_owned(),
        source_url: ch.link().to_owned(),
        podracer_url: "xxx".to_owned(),
        release_dates: dates
    };
    let json = serde_json::to_string_pretty(&racer_data)?;

    let filename = String::from(PODRACER_DIR) +"/"+ RACER_FILE_NAME;
    let mut fp = File::create(filename)?;
    fp.write_all(json.as_bytes())
}
