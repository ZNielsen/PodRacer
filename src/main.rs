#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod racer;
mod utils;

use std::path::PathBuf;
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

#[post("/update/<podcast>")]
fn update_one_handler(podcast: String) {
    // Update the specified podcast
}

#[post("/update")]
fn update_all_handler() {
    racer::update_all();
}

#[post("/create_feed?<url>&<rate>&<integrate_new>")]
fn create_feed_handler(url: String, rate: f32, integrate_new: bool) -> String {
    let feed_racer = utils::create_feed(url, rate, integrate_new);
    println!("{}", feed_racer);
    // String::from("Success!!!")

    // Grab some info to return
    let path: PathBuf = [feed_racer.get_racer_path().to_str().unwrap() ,racer::ORIGINAL_RSS_FILE].iter().collect();
    let file = File::open(path).unwrap();
    let mut buf = std::io::BufReader::new(&file);
    let feed = rss::Channel::read_from(&mut buf).unwrap();
    let num_items = feed.items().len();
    let weeks_behind = feed_racer.get_first_pubdate().signed_duration_since(chrono::Utc::now()).num_weeks().abs();
    let weeks_to_catch_up = ((weeks_behind as f32) / feed_racer.get_rate()) as u32;
    let days_to_catch_up = (((weeks_behind*7) as f32) / feed_racer.get_rate()) as u32;
    let catch_up_date = chrono::Utc::now() + chrono::Duration::weeks(weeks_to_catch_up as i64);

    // Package up the return string
    let mut ret = format!("You have {} episodes to catch up on.\n", num_items);
    ret += format!("You are {} weeks behind, it will take you about {} weeks ({} days) to catch up (excluding new episodes).\n",
            weeks_behind, weeks_to_catch_up, days_to_catch_up).as_str();
    ret += format!("You should catch up on {}.\n", catch_up_date.format("%d %b, %Y")).as_str();
    ret += format!("\nSubscribe to this URL in your podcatching app of choice: {}", feed_racer.get_podracer_url()).as_str();
    ret
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

#[get("/podcasts/<podcast>")]
fn serve_rss_handler(podcast: String) -> Option<File> {
    // Serve the rss file
    let home = dirs::home_dir()?;
    let path: PathBuf = [home.to_str()?, racer::PODRACER_DIR, &podcast, racer::RACER_RSS_FILE].iter().collect();
    match std::fs::File::open(path) {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}

fn launch_rocket() {
    let rocket = rocket::ignite();
    rocket.mount("/", routes![index])
          .mount("/", routes![update_one_handler])
          .mount("/", routes![update_all_handler])
          .mount("/", routes![delete_feed_handler])
          .mount("/", routes![list_feeds_handler])
          .mount("/", routes![serve_rss_handler])
          .mount("/", routes![create_feed_handler]).launch();
}

fn main() {
    // Manually update on start
    racer::update_all();
    // Create update thread - update every hour
    let _update_thread = std::thread::Builder::new().name("Updater".to_owned()).spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            racer::update_all();
        }
    });
    launch_rocket();
}
