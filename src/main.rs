use std::{error::Error, io::{self, Read}};
use rss::Channel;

static PODRACER_DIR: &str = "~/.podracer";

fn main() {
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
    println!("How fast would you like to catch up?");
    print!("Enter a floating point number [defualt: 1.20]: ");
    io::stdin().read_to_string(&mut buffer).unwrap();

    // Make directory
    let dir = create_feed_racer_dir(&channel);
    // Write out original rss feed to file in dir
    channel.pretty_write_to(writer: W, indent_char: u8, indent_size: usize)
    // Make racer file
    // Write out racer file
    // Run update() on this directory
}

fn get_rss_channel(url: &String) -> Result<Channel, Box<dyn Error>>
{
    println!("Getting content");
    let content = reqwest::blocking::get(url).unwrap()
                    .bytes().unwrap();
    println!("Got content");
    let channel = Channel::read_from(&content[..])?;
    println!("Got channel");
    Ok(channel)
}

fn get_time_behind(channel: &Channel) -> i64
{
    let published = channel.items()
                    .last().unwrap()
                    .pub_date().unwrap();
    let diff = chrono::DateTime::parse_from_rfc2822(published).unwrap()
                .signed_duration_since(chrono::Utc::now());
    diff.num_weeks()
}

fn create_feed_racer_dir(ch: &Channel) -> String
{
    // Create this feed's dir name
    let mut dir = String::new();
    dir.push_str(PODRACER_DIR);
    dir.push_str("/test");
    std::fs::create_dir_all(&dir);
    dir
}
