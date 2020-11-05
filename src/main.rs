use std::{error::Error, io::{self, Read}};
use rss::Channel;

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
    // Write out original rss feed
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
