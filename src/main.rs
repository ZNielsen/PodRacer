#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod racer;
mod utils;
use std::fs::File;

// JSON keys
const KEY_EPISODE_RACER_DATE: &'static str = "date_string";
const KEY_SCHEMA_VERSION:     &'static str = "schema_version";
const KEY_EPISODE_NUMBER:     &'static str = "ep_num";
const KEY_RELEASE_DATES:      &'static str = "release_dates";
const KEY_SOURCE_URL:         &'static str = "source_url";
const KEY_RACER_PATH:         &'static str = "racer_path";
const KEY_RACER_URL:          &'static str = "podracer_url";

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/hello?wave&<name>")]
fn test_url_handler(name: &rocket::http::RawStr) -> String {
    format!("Yes test: {}", name)
}

#[post("/create_feed?<url>&<rate>&<integrate_new>")]
fn create_feed_handler(url: String, rate: f32, integrate_new: bool) -> String {
    let feed_racer = utils::create_feed(url, rate, integrate_new);
    println!("{}", feed_racer);
    String::from("Success!!!")

    // // Grab some info to return
    // let file = File::open(String::from(feed_racer.get_racer_path()) +"/"+ racer::ORIGINAL_RSS_FILE).unwrap();
    // let mut buf = std::io::BufReader::new(&file);
    // let feed = rss::Channel::read_from(&mut buf).unwrap();
    // let num_items = feed.items().len();
    // let weeks_behind = feed_racer.get_first_pubdate().signed_duration_since(chrono::Utc::now()).num_weeks().abs();
    // let weeks_to_catch_up = ((weeks_behind as f32) / feed_racer.get_rate()) as u32;
    // let catch_up_date = chrono::Utc::now() + chrono::Duration::weeks(weeks_to_catch_up as i64);

    // // Package up the return string
    // let mut ret = format!("You have {} episodes to catch up on.\n", num_items);
    // ret += format!("You are {} weeks behind, it will take you about {} weeks to catch up (excluding new episodes).\n",
    //         weeks_behind, weeks_to_catch_up).as_str();
    // ret += format!("You should catch up on {}.\n", catch_up_date.format("%d %m, %Y")).as_str();
    // ret += format!("\nSubscribe to this URL in your podcatching app of choice: {}", feed_racer.get_podracer_url()).as_str();
    // ret
}

#[post("/delete_feed?<url>")]
fn delete_feed_handler(url: String) {
    // Search for a FeedRacer that has this URL
}

#[get("/list_feeds")]
fn list_feeds_handler() -> String {
    // Search for a FeedRacer that has this URL
    "".to_owned()
}

fn launch_rocket() {
    let mut rocket = rocket::ignite();
    rocket.mount("/", routes![index])
          .mount("/", routes![test_url_handler])
          .mount("/", routes![delete_feed_handler])
          .mount("/", routes![list_feeds_handler])
          .mount("/", routes![create_feed_handler]).launch();
}

fn main() {

    launch_rocket();
}
