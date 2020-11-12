////////////////////////////////////////////////////////////////////////////////
//  File:   main.rs
//
//  © Zach Nielsen 2020
//  Main server code
//
#![feature(proc_macro_hygiene, decl_macro)]
////////////////////////////////////////////////////////////////////////////////
//  Included Modules
////////////////////////////////////////////////////////////////////////////////
#[macro_use]
extern crate rocket;
mod racer;

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use rocket::fairing::AdHoc;
use rocket::State;
use std::path::PathBuf;
use std::fs::File;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////

struct RocketConfig {
    pub address: String,
    pub port: u64,
}
struct UpdateFactor(u64);

#[get("/")]
fn show_form_handler() -> File {
    File::open("static/form.html").expect("form file to exist")
}

#[post("/update/<podcast>")]
fn update_one_handler(podcast: String) {
    // Update the specified podcast
}

#[post("/update")]
fn update_all_handler() {
    match racer::update_all() {
        Ok(_) => (),
        Err(string) => println!("Error in update_all_handler: {}", string),
    };
}
#[post("/create_feed?<url>&<rate>&<integrate_new>", rank = 2)]
fn create_feed_handler( config: State<RocketConfig>,
                        url: String,
                        rate: f32,
                        integrate_new: bool
                    ) -> String {
    create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate:rate,
        integrate_new: integrate_new,
        start_ep: 1
    })
}
#[post("/create_feed?<url>&<rate>&<integrate_new>&<start_ep>", rank = 1)]
fn create_feed_handler_ep(config: State<RocketConfig>, url: String, rate: f32, integrate_new: bool, start_ep: usize) -> String {
    create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate:rate,
        integrate_new: integrate_new,
        start_ep: start_ep
    })
}

fn create_feed(params: racer::RacerCreationParams) -> String {
    let feed_racer = match racer::create_feed(&params) {
        Ok(val) => val,
        Err(e) => return e,
    };
    println!("{}", feed_racer);

    // Grab some info to return
    let path: PathBuf = [feed_racer.get_racer_path().to_str().unwrap(), racer::ORIGINAL_RSS_FILE].iter().collect();
    let file = File::open(path).unwrap();
    let mut buf = std::io::BufReader::new(&file);
    let feed = rss::Channel::read_from(&mut buf).unwrap();
    let num_items = feed.items().len() - &params.start_ep;
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

#[get("/podcasts/<podcast>/racer.rss")]
fn serve_rss_handler(podcast: String) -> Option<File> {
    println!("Serving at {}", chrono::Utc::now().to_rfc3339());
    // Serve the rss file
    let home = dirs::home_dir()?;
    let path: PathBuf = [home.to_str()?, racer::PODRACER_DIR, &podcast, racer::RACER_RSS_FILE].iter().collect();
    match std::fs::File::open(path) {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   [name]
 //
 // NOTES:
 // ARGS:
 // RETURN:
 //
fn main() {
    let rocket = rocket::ignite()
        .mount("/", routes![show_form_handler])
        .mount("/", routes![update_one_handler])
        .mount("/", routes![update_all_handler])
        .mount("/", routes![delete_feed_handler])
        .mount("/", routes![list_feeds_handler])
        .mount("/", routes![serve_rss_handler])
        .mount("/", routes![create_feed_handler])
        .mount("/", routes![create_feed_handler_ep])
        .attach(AdHoc::on_attach("Asset Config", |rocket| {
            // Parse out custom config values
            let rocket_config = RocketConfig {
                address: rocket.config().get_str("host").unwrap().to_owned(),
                port: rocket.config().port as u64,
            };
            let update_factor = rocket.config().get_int("update_factor").unwrap() as u64;

            // Add custom configs to the State manager - only one of each type is allowed
            Ok(rocket
                .manage(rocket_config)
                .manage(UpdateFactor(update_factor))
            )
        }));

    // Manually update on start
    match racer::update_all() {
        Ok(_) => println!("Updated all racer feeds on server"),
        Err(string) => println!("Error in update_all on boot: {}", string),
    };

    let duration: u64 = match rocket.state::<u64>() {
        Some(val) => val.clone(),
        None => (59 * 60),
    };

    // Create update thread - update every hour
    let _update_thread = std::thread::Builder::new().name("Updater".to_owned()).spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(duration));
            print!("Updating all feeds... ");
            match racer::update_all() {
                Ok(_) => (),
                Err(string) => println!("Error in update_all in update thread: {}", string),
            };
            println!("Done")
        }
    });
    rocket.launch();
}
