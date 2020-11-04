use std::{error::Error, io::{self, Read}};
use rss::Channel;

enum CatchUpMode {
    Interpolate,
    Rate
}

fn main() {
    println!("What's the URL?: ");
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let mut channel = get_rss_channel(&buffer)
                        .unwrap();
    let num_episodes = channel.items().len();
    let weeks_behind = get_time_behind(&channel);
    println!("There are {} episodes, and you are {} weeks behind.",
        num_episodes, weeks_behind);
    println!("How would you like to catch up?");
    println!("1) Catch up by certain date");
    println!("2) Consume episodes at an accelerated rate");
    println!("Choose 1/2: ");
    let mode = match io::stdin().read_to_string(&mut buffer).unwrap() {
        1 => CatchUpMode::Interpolate,
        2 => CatchUpMode::Rate,
        _ => panic!("Invalid choice"),
    };
}

fn get_rss_channel(url: &String) -> Result<Channel, Box<dyn Error>>
{
    let content = reqwest::blocking::get(url).unwrap()
                    .bytes().unwrap();
    let channel = Channel::read_from(&content[..])?;
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
