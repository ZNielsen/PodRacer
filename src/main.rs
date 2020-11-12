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
fn update_all_handler() -> Result<(), String> {
    match racer::update_all() {
        Ok(_) => Ok(()),
        Err(string) => Err(format!("Error in update_all_handler: {}", string)),
    }
}
#[post("/create_feed?<url>&<rate>&<integrate_new>", rank = 2)]
fn create_feed_handler( config: State<RocketConfig>,
                        url: String,
                        rate: f32,
                        integrate_new: bool
                    ) -> Result<String, String> {
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
fn create_feed_handler_ep(config: State<RocketConfig>,
                          url: String,
                          rate: f32,
                          integrate_new: bool,
                          start_ep: usize
                        ) -> Result<File, File> {
    match create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate:rate,
        integrate_new: integrate_new,
        start_ep: start_ep
    }) {
        Ok(_) => Ok(File::open("static/submit_success.html").unwrap()),
        Err(_) => Err(File::open("static/submit_failure.html").unwrap()),
    }
}
#[get("/create_feed?<url>&<rate>&<integrate_new>&<start_ep>", rank = 1)]
fn create_feed_from_form_handler(config: State<RocketConfig>,
                                 url: String,
                                 rate: f32,
                                 integrate_new: bool,
                                 start_ep: usize
                                ) -> Result<String, String> {
    create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate:rate,
        integrate_new: integrate_new,
        start_ep: start_ep
    })
}

fn create_feed(params: racer::RacerCreationParams) -> Result<String,String> {
    let feed_racer = match racer::create_feed(&params) {
        Ok(val) => val,
        Err(e) => return Err(e),
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
    Ok(ret)
}

// Accept URL or dir name
#[post("/delete_feed?<podcast>")]
fn delete_feed_handler(podcast: String) -> Result<String, String> {
    // Try dir name first
    let mut dir = dirs::home_dir().unwrap();
    dir.push(racer::PODRACER_DIR);
    dir.push(&podcast);
    if dir.is_dir() {
        // Delete it and return Ok
        match std::fs::remove_dir_all(dir.as_path()) {
            Ok(_) => return Ok(format!("Podcast deleted from server: {}", &podcast)),
            Err(e) => {
                println!("Error removing podcast {} from sever: {}", &podcast, e);
                return Err(format!("Error removing podcast from server."));
            },
        };
    }

    // Not a dir, search for a FeedRacer that has this URL
    Err(format!("TODO: search racers for this url: {}", podcast))
}

// List all feeds on this server
#[get("/list_feeds")]
fn list_feeds_handler() -> Result<String, String> {
    let mut ret = String::new();
    // Get all folders in the podracer dir
    let podcast_dirs = match racer::get_all_podcast_dirs() {
        Ok(val) => val,
        Err(str) => {
            println!("Error in list_feeds_handler: {}", str);
            return Err(format!("Error getting feeds, check logs"));
        },
    };
    for podcast_dir in podcast_dirs {
        let path = match podcast_dir {
            Ok(val) => val.path(),
            Err(e) => return Err(format!("Error iterating over path from read_dir: {}", e)),
        };
        let this_dir_name = path.file_name().unwrap();
        ret.push_str(this_dir_name.to_str().unwrap());
        ret.push_str("\n");
    }
    Ok(ret)
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
        .mount("/", routes![create_feed_from_form_handler])
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
