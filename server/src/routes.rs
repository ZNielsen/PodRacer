////////////////////////////////////////////////////////////////////////////////
//  File:   routes.rs
//
//  © Zach Nielsen 2020
//  All the routes for rocket
//
////////////////////////////////////////////////////////////////////////////////
//  Included Modules
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use super::racer;

use rocket::request::Form;
use rocket::{Request, State};
use rocket_contrib::{templates::Template, uuid::Uuid};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use tera::Context;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////

const FEED_NOT_FOUND_FILE: &'static str = "feed_not_found";
// const GENERIC_TEXT_FILE:   &'static str = "generic_text";
const EDIT_FEED_FILE:      &'static str = "edit_feed";
const SUCCESS_FILE:        &'static str = "submit_success";
const FAILURE_FILE:        &'static str = "submit_failure";

//
// Structs for Rocket config
//
pub struct RocketConfig {
    pub static_file_dir: String,
    pub podracer_dir: String,
    pub address: String,
    pub port: u64,
}
pub struct UpdateFactor(pub u64);

struct FeedFunFacts {
    num_items: usize,
    weeks_behind: i64,
    weeks_to_catch_up: u32,
    days_to_catch_up: u32,
    catch_up_date: chrono::DateTime<chrono::Utc>,
    subscribe_url: String,
    uuid: String,
}

#[derive(FromForm)]
pub struct FormParams {
    pub url: String,
    pub rate: f32,
    pub start_ep: usize,
}

//
// Rocket Routes
//
////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed_form_handler
//
//  NOTES:  Give the default form when requesting the root
//  ARGS:   None
//  RETURN: The new podcast form file
//
#[get("/")]
pub fn create_feed_form_handler(config: State<RocketConfig>) -> File {
    let file = format!("{}/{}", &config.static_file_dir, "create_feed_form.html");
    match File::open(&file) {
        Ok(f) => f,
        Err(e) => {
            println!("Error: {}", e);
            println!("Attempted to access {}", file);
            File::open(format!("{}/{}", &config.static_file_dir, "404.html")).unwrap()
        }
    }
}

#[catch(404)]
pub fn not_found_handler(req: &Request) -> File {
    println!("404 served to: {:?}", req.client_ip());
    println!("\t{:?} requested {}", req.real_ip(), req.uri());
    let static_file_dir: String = req.guard::<State<RocketConfig>>()
        .map(|config| String::from(&config.static_file_dir))
        .expect("request's rocket config has a static file dir");
    let filename = format!("{}/{}", static_file_dir, "404.html");
    println!("\tServing 404 file at {}", filename);
    File::open(&filename).unwrap()
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed_handler
//
//  NOTES:  Creates a new PodRacer feed. From the web ui.
//  ARGS:
//      config -
//      url -
//      rate -
//  RETURN: A result with string information either way. Tailored for a curl response
//
#[get("/create_feed?<form_data..>")]
pub fn create_feed_handler(config: State<RocketConfig>, form_data: Form<FormParams>) -> Template {
    let mut context = Context::new();
    match create_feed(racer::RacerCreationParams {
        static_file_dir: config.static_file_dir.clone(),
        podracer_dir: config.podracer_dir.clone(),
        start_ep: form_data.start_ep,
        address: config.address.clone(),
        rate: form_data.rate,
        port: config.port,
        url: form_data.url.clone(),
    }) {
        Ok(fun_facts) => {
            let catch_up_date = format!("{}", fun_facts.catch_up_date.format("%d %b, %Y"));
            context.insert("weeks_to_catch_up", &fun_facts.weeks_to_catch_up);
            context.insert("days_to_catch_up", &fun_facts.days_to_catch_up);
            context.insert("catch_up_date", &catch_up_date);
            context.insert("subscribe_url", &fun_facts.subscribe_url);
            context.insert("weeks_behind", &fun_facts.weeks_behind);
            context.insert("num_items", &fun_facts.num_items);
            context.insert("uuid", &fun_facts.uuid);
            Template::render(SUCCESS_FILE, &context.into_json())
        }
        Err(e) => {
            context.insert("error_string", &e);
            Template::render(FAILURE_FILE, &context.into_json())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   edit_feed_handler
//
//  NOTES:  Edits a PodRacer feed by uuid. From the web ui.
//  ARGS:
//      uuid - The UUID of the feed to edit
//  RETURN: A result with string information either way. Tailored for a curl response
//
#[get("/edit_feed?<uuid>")]
pub fn edit_feed_handler(config: State<RocketConfig>, uuid: Uuid) -> Template {
    let mut context = Context::new();
    match get_feed_by_uuid(&config, &uuid) {
        Ok(racer) => {
            fill_edit_feed_data_from_racer(&mut context, &racer);
            Template::render(EDIT_FEED_FILE, &context.into_json())
        }
        Err(e) => {
            println!("Error getting feed: {}", e);
            context.insert("uuid", &uuid.to_string());
            Template::render(FEED_NOT_FOUND_FILE, &context.into_json())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   pause_feed_handler
//
//  NOTES:
//  ARGS:
//  RETURN:
//
#[post("/pause_feed?<uuid>")]
pub fn pause_feed_handler(config: State<RocketConfig>, uuid: Uuid) -> Template {
    let mut context = Context::new();
    match get_feed_by_uuid(&config, &uuid) {
        Ok(mut racer) => {
            racer.pause_feed();

            fill_edit_feed_data_from_racer(&mut context, &racer);
            context.insert("top_text", "Feed has been paused. No new episodes will be published \
                until you unpause this feed.");
            Template::render(EDIT_FEED_FILE, &context.into_json())
        }
        Err(e) => {
            println!("Error getting feed: {}", e);
            context.insert("uuid", &uuid.to_string());
            Template::render(FEED_NOT_FOUND_FILE, &context.into_json())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   unpause_feed_handler
//
//  NOTES:
//  ARGS:
//  RETURN:
//
#[post("/unpause_feed?<uuid>")]
pub fn unpause_feed_handler(config: State<RocketConfig>, uuid: Uuid) -> Template {
    let mut context = Context::new();
    match get_feed_by_uuid(&config, &uuid) {
        Ok(mut racer) => {
            racer.unpause_feed();

            fill_edit_feed_data_from_racer(&mut context, &racer);
            context.insert("top_text", "Feed has been unpaused. The next episode has \
                been published (but give it a couple minutes to show up in your podcatcher)");
            Template::render(EDIT_FEED_FILE, &context.into_json())
        }
        Err(e) => {
            println!("Error getting feed: {}", e);
            context.insert("uuid", &uuid.to_string());
            Template::render(FEED_NOT_FOUND_FILE, &context.into_json())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed_cli_handler
//
//  NOTES:  Creates a new PodRacer feed. This is probably from the curl script
//      This can probably safely be deleted
//  ARGS:
//      config -
//      url -
//      rate -
//  RETURN: A result with string information either way. Tailored for a curl response
//
#[post("/create_feed_cli?<url>&<rate>", rank = 2)]
pub fn create_feed_cli_handler(
    config: State<RocketConfig>,
    url: String,
    rate: f32,
) -> Result<String, String> {
    match create_feed(racer::RacerCreationParams {
        static_file_dir: config.static_file_dir.clone(),
        podracer_dir: config.podracer_dir.clone(),
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate: rate,
        start_ep: 1,
    }) {
        Ok(val) => Ok(make_fun_fact_string_cli(&val)),
        Err(e) => Err(e),
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed_cli_ep_handler
//
//  NOTES:
//      Creates a new PodRacer feed, but includes a start episode. This is
//      probably from the form.
//  ARGS:
//      config -
//      url -
//      rate -
//      start_ep -
//  RETURN:
//      A result containing either a success file or a failure file.
//      If Ok(), the File will have the subscribe url to display to the user
//
#[post("/create_feed_cli?<url>&<rate>&<start_ep>", rank = 1)]
pub fn create_feed_cli_ep_handler(
    config: State<RocketConfig>,
    url: String,
    rate: f32,
    start_ep: usize,
) -> Result<String, String> {
    match create_feed(racer::RacerCreationParams {
        static_file_dir: config.static_file_dir.clone(),
        podracer_dir: config.podracer_dir.clone(),
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate: rate,
        start_ep: start_ep,
    }) {
        Ok(val) => Ok(make_fun_fact_string_cli(&val)),
        Err(e) => Err(e),
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   update_one_handler
//
//  NOTES:  Update one podcast, specified by folder name or subscribe url
//  ARGS:
//      podcast - The podcast to update. Specified by the folder name or
//               the PodRacer subscribe url.
//  RETURN:
//
#[post("/update/<podcast>")]
pub fn update_one_handler(config: State<RocketConfig>, podcast: String) -> std::io::Result<()> {
    // Update the specified podcast
    // Check if podcast is folder name
    if let Some(mut racer) = racer::get_by_dir_name(&config.podracer_dir, &podcast) {
        return match racer.update(&racer::RssFile::Download) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }
    // Check if subscribe url
    if let Some(mut racer) = racer::get_by_url(&config.podracer_dir, &podcast) {
        return match racer.update(&racer::RssFile::Download) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }
    Err(Error::new(
        ErrorKind::NotFound,
        format!("podcast not found"),
    ))
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   update_all_handler
//
//  NOTES:  Forces update of all podcast feeds on this server
//  ARGS:   None
//  RETURN: A result. If error, a string containing some error info
//
#[post("/update")]
pub fn update_all_handler(config: State<RocketConfig>) -> Result<(), String> {
    match racer::update_all(&config.podracer_dir) {
        Ok(_) => Ok(()),
        Err(string) => Err(format!("Error in update_all_handler: {}", string)),
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   list_feeds_handler
//
//  NOTES:
//      List all the feeds on this server. Gives a list of podcasts by the
//      directory names: <podcast_name>_<rate>_<feed creation date>
//  ARGS:   None
//  RETURN: Result string - either the feeds or info on what failed
//
#[get("/list_feeds")]
pub fn list_feeds_handler(config: State<RocketConfig>) -> Result<String, String> {
    let mut ret = String::new();
    let racers = match racer::get_all_racers(&config.podracer_dir) {
        Ok(val) => val,
        Err(e) => return Err(format!("Error getting racers: {}", e)),
    };

    // Parse into a string to be fed back to curl
    for mut racer in racers {
        ret += &format!("Podcast: {}", racer.get_or_create_podcast_title());
        ret += &format!(
            "\tpodcast folder: {:?}\n",
            racer.get_racer_path().file_name().unwrap()
        );
        ret += &format!("\tsubscribe_url: {}\n", racer.get_subscribe_url());
        ret += &format!("\tsource_url: {}\n", racer.get_source_url());
        ret += &format!("\tfirst_pubdate: {}\n", racer.get_first_pubdate());
        ret += &format!("\tanchor_date: {}\n", racer.get_anchor_date());
        ret += &format!("\trate: {}\n", racer.get_rate());
        ret.push_str("\n");
    }

    Ok(ret)
}
////////////////////////////////////////////////////////////////////////////////
//  NAME:   serve_rss_handler
//
//  NOTES:
//      Serves the racer.rss file for the specified podcast. This is almost
//      certainly called by a podcast player to check for new episodes.
//      TODO: Should we not serve the file if nothing changed? Is there a safe
//         way to do that?
//  ARGS:   podcast - The podcast to serve. Format is the folder name
//  RETURN: Our PodRacer RSS file
//
#[get("/podcasts/<podcast>/racer.rss")]
pub fn serve_rss_handler(config: State<RocketConfig>, podcast: String) -> Result<File, std::io::Error> {
    println!("Serving at {}", chrono::Utc::now().to_rfc3339());
    // Serve the rss file
    let path: PathBuf = [
        &config.podracer_dir,
        &podcast,
        racer::RACER_RSS_FILE,
    ]
    .iter()
    .collect();
    println!("Getting podcast from path: {:?}", path);
    std::fs::File::open(&path)
}

//
// Helper Functions
//

fn fill_edit_feed_data_from_racer(cx: &mut Context, racer: &racer::FeedRacer) {
    let next = racer.get_next_episode_pub_date();
    let now = chrono::Utc::now();
    let next_pub_date_string = if next <= now {
        String::from("Caught up, whenever they publish another one")
    }
    else if let Some(_) = racer.get_pause_date() {
        String::from("Feed paused, unpause to publish next episode")
    }
    else {
        next.to_rfc2822()
    };
    cx.insert("next_pub_date_string", &next_pub_date_string);
    cx.insert("podcast_title", &racer.get_podcast_title());
    cx.insert("subscribe_url", &racer.get_subscribe_url());
    cx.insert("first_pubdate", &racer.get_first_pubdate().to_rfc2822());
    cx.insert("num_published", &racer.get_num_to_publish());
    cx.insert("num_episodes",  &racer.get_num_episodes());
    cx.insert("anchor_date",   &racer.get_anchor_date().to_rfc2822());
    cx.insert("source_url",    &racer.get_source_url());
    cx.insert("rate",          &format!("{:.2}", racer.get_rate()));
    cx.insert("uuid",          &racer.get_uuid_string());
    if let Some(old_rate) = racer.get_old_rate() {
        cx.insert("old_rate", &format!("{:.2}", old_rate));
    }
    if let Some(pause_date) = racer.get_pause_date() {
        cx.insert("pause_date", &pause_date.to_rfc2822());
    }
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed
//
//  NOTES:
//      Crates a PodRacer feed for given parameters. Prints some stats for the
//      user for display over curl. Might need to make another version to handle
//      displaying info over web UI
//  ARGS:   params - All the parameters required to put together a feed. See
//                  the struct for more info.
//  RETURN:
//      A result. If Ok(), contains a bunch of stats for the user. If Err(),
//      contains info for why it failed
//
fn create_feed(mut params: racer::RacerCreationParams) -> Result<FeedFunFacts, String> {
    let feed_racer = match racer::create_feed(&mut params) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    println!("{}", feed_racer);
    println!("Success creating feed!");

    // Grab some info to return
    let path: PathBuf = [
        feed_racer.get_racer_path().to_str().unwrap(),
        racer::ORIGINAL_RSS_FILE,
    ].iter().collect();

    // This should not be needed, but it is.
    // Look into why this can't be removed -> It's needed because rss writes a file it can't read
    // GH-33 GH-39
    racer::scrub_xml_file(&path);

    println!("Getting stats from file at {}", path.display());
    let file = File::open(&path).unwrap();
    let mut buf = std::io::BufReader::new(&file);
    let feed = rss::Channel::read_from(&mut buf).unwrap();
    let num_items = feed.items().len() - &params.start_ep;
    let weeks_behind = feed_racer
        .get_first_pubdate()
        .signed_duration_since(chrono::Utc::now())
        .num_weeks()
        .abs();
    let days_behind = feed_racer
        .get_first_pubdate()
        .signed_duration_since(chrono::Utc::now())
        .num_days()
        .abs();
    let weeks_to_catch_up = ((weeks_behind as f32) / feed_racer.get_rate()) as u32;
    let days_to_catch_up = ((days_behind as f32) / feed_racer.get_rate()) as u32;
    let catch_up_date = chrono::Utc::now() + chrono::Duration::weeks(weeks_to_catch_up as i64);

    Ok(FeedFunFacts {
        num_items: num_items,
        weeks_behind: weeks_behind,
        weeks_to_catch_up: weeks_to_catch_up,
        days_to_catch_up: days_to_catch_up,
        catch_up_date: catch_up_date,
        subscribe_url: feed_racer.get_subscribe_url().to_owned(),
        uuid: feed_racer.get_uuid_string(),
    })
}

////////////////////////////////////////////////////////////////////////////////
//  NAME:   get_feed_by_uuid
//
//  NOTES:
//  ARGS:
//  RETURN:
//
fn get_feed_by_uuid(config: &State<RocketConfig>, uuid: &Uuid) -> Result<racer::FeedRacer, String> {
    let racers = match racer::get_all_racers(&config.podracer_dir) {
        Ok(val) => val,
        Err(e) => return Err(format!("Error getting racers: {}", e)),
    };

    // Parse into a string to be fed back to curl
    for racer in racers {
        if let Some(ref racer_uuid) = racer.get_uuid() {
            if racer_uuid == &&uuid.to_string() {
                return Ok(racer)
            }
        }
    }
    Err(format!("Error: no racer with uuid: {}", uuid))
}

fn make_fun_fact_string_cli(fff: &FeedFunFacts) -> String {
    // Package up the return string
    let mut ret = format!("You have {} episodes to catch up on.\n", fff.num_items);
    ret += format!("You are {} weeks behind, it will take you about {} weeks ({} days) to catch up (excluding new episodes).\n",
        fff.weeks_behind, fff.weeks_to_catch_up, fff.days_to_catch_up).as_str();
    ret += format!(
        "You should catch up on {}.\n",
        fff.catch_up_date.format("%d %b, %Y")
    )
    .as_str();
    ret += format!(
        "\nSubscribe to this URL in your podcatching app of choice: {}",
        fff.subscribe_url
    )
    .as_str();
    ret
}
