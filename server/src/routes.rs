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

use rocket_contrib::templates::Template;
use rocket::request::{Form};
use rocket::{Request, State};
use tera::Context;
use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::fs::File;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
const SUCCESS_FILE: &'static str = "submit_success";
const FAILURE_FILE: &'static str = "submit_failure";


//
// Structs for Rocket config
//
pub struct RocketConfig {
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
    subscribe_url: String
}

#[derive(FromForm)]
pub struct FormParams {
    pub url: String,
    pub rate: f32,
    pub integrate_new: Option<bool>,
    pub start_ep: usize
}

//
// Rocket Routes
//

// mod manual {
//     use rocket::response::NamedFile;

//     #[rocket::get("/rocket-icon.jpg")]
//     pub async fn icon() -> Option<NamedFile> {
//         NamedFile::open("static/rocket-icon.jpg").ok()
//     }
// }

////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed_form_handler
 //
 // NOTES:  Give the default form when requesting the root
 // ARGS:   None
 // RETURN: The new podcast form file
 //
#[get("/")]
pub fn create_feed_form_handler() -> Template {
    Template::render("create_feed_form", &Context::new().into_json())
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed_handler
 //
 // NOTES:  Creates a new PodRacer feed. From the web ui.
 // ARGS:
 //     config -
 //     url -
 //     rate -
 //     integrate_new -
 // RETURN: A result with string information either way. Tailored for a curl response
 //
#[get("/create_feed?<form_data..>")]
pub fn create_feed_handler( config: State<RocketConfig>,
                            form_data: Form<FormParams>) -> Template {
    let mut context = Context::new();
    let new = form_data.integrate_new.unwrap_or(false);
    match create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: form_data.url.clone(),
        rate: form_data.rate,
        integrate_new: new,
        start_ep: form_data.start_ep
    }) {
        Ok(fun_facts) => {
            let catch_up_date = format!("{}", fun_facts.catch_up_date.format("%d %b, %Y"));
            context.insert("weeks_to_catch_up", &fun_facts.weeks_to_catch_up);
            context.insert("days_to_catch_up",  &fun_facts.days_to_catch_up);
            context.insert("catch_up_date",     &catch_up_date);
            context.insert("subscribe_url",     &fun_facts.subscribe_url);
            context.insert("weeks_behind",      &fun_facts.weeks_behind);
            context.insert("num_items",         &fun_facts.num_items);
            Template::render(SUCCESS_FILE, &context.into_json())
        }
        Err(e) => {
            context.insert("error_string", &e);
            Template::render(FAILURE_FILE, &context.into_json())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed_cli_handler
 //
 // NOTES:  Creates a new PodRacer feed. This is probably from the curl script
 //     This can probably safely be deleted
 // ARGS:
 //     config -
 //     url -
 //     rate -
 //     integrate_new -
 // RETURN: A result with string information either way. Tailored for a curl response
 //
#[post("/create_feed_cli?<url>&<rate>&<integrate_new>", rank = 2)]
pub fn create_feed_cli_handler( config: State<RocketConfig>,
                                url: String,
                                rate: f32,
                                integrate_new: bool
) -> Result<String, String> {
    match create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate:rate,
        integrate_new: integrate_new,
        start_ep: 1
    }) {
        Ok(val) => Ok(make_fun_fact_string_cli(&val)),
        Err(e) => Err(e)
    }
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed_cli_ep_handler
 //
 // NOTES:
 //     Creatse a new PodRacer feed, but includes a start episode. This is
 //     probably from the form.
 // ARGS:
 //     config -
 //     url -
 //     rate -
 //     integrate_new -
 //     start_ep -
 // RETURN:
 //     A result containing either a success file or a failure file.
 //     If Ok(), the File will have the subscribe url to display to the user
 //
#[post("/create_feed_cli?<url>&<rate>&<integrate_new>&<start_ep>", rank = 1)]
 pub fn create_feed_cli_ep_handler( config: State<RocketConfig>,
                                    url: String,
                                    rate: f32,
                                    integrate_new: bool,
                                    start_ep: usize
) -> Result<String, String> {
    match create_feed( racer::RacerCreationParams {
        address: config.address.clone(),
        port: config.port,
        url: url,
        rate:rate,
        integrate_new: integrate_new,
        start_ep: start_ep
    }) {
       Ok(val) => Ok(make_fun_fact_string_cli(&val)),
       Err(e) => Err(e)
    }
}

#[catch(404)]
pub fn not_found_handler(req: &Request) -> Template {
    println!("404 served to: {:?}", req.client_ip());
    println!("\t{:?} requested {}", req.real_ip(), req.uri());
    Template::render("404", Context::new().into_json())
}

//
// Helper Functions
//

////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed
 //
 // NOTES:
 //     Crates a PodRacer feed for given parameters. Prints some stats for the
 //     user for display over curl. Might need to make another version to handle
 //     displaying info over web UI
 // ARGS:   params - All the parameters required to put together a feed. See
 //                  the struct for more info.
 // RETURN:
 //     A result. If Ok(), contains a bunch of stats for the user. If Err(),
 //     contains info for why it failed
 //
fn create_feed(mut params: racer::RacerCreationParams) -> Result<FeedFunFacts,String> {
    let feed_racer = match racer::create_feed(&mut params) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    println!("{}", feed_racer);

    // Grab some info to return
    let path: PathBuf = [feed_racer.get_racer_path().to_str().unwrap(), racer::ORIGINAL_RSS_FILE].iter().collect();
    super::scrub_xml(&path);
   println!("Getting file from {}", path.display());
   let file = File::open(&path).unwrap();
   let mut buf = std::io::BufReader::new(&file);
   let feed = rss::Channel::read_from(&mut buf).unwrap();
   let num_items = feed.items().len() - &params.start_ep;
   let weeks_behind = feed_racer.get_first_pubdate().signed_duration_since(chrono::Utc::now()).num_weeks().abs();
   let weeks_to_catch_up = ((weeks_behind as f32) / feed_racer.get_rate()) as u32;
   let days_to_catch_up = (((weeks_behind*7) as f32) / feed_racer.get_rate()) as u32;
   let catch_up_date = chrono::Utc::now() + chrono::Duration::weeks(weeks_to_catch_up as i64);

    Ok( FeedFunFacts {
        num_items: num_items,
        weeks_behind: weeks_behind,
        weeks_to_catch_up: weeks_to_catch_up,
        days_to_catch_up: days_to_catch_up,
        catch_up_date: catch_up_date,
        subscribe_url: feed_racer.get_subscribe_url().to_owned()
    })
}

fn make_fun_fact_string_cli(fff: &FeedFunFacts) -> String {
    // Package up the return string
    let mut ret = format!("You have {} episodes to catch up on.\n", fff.num_items);
    ret += format!("You are {} weeks behind, it will take you about {} weeks ({} days) to catch up (excluding new episodes).\n",
        fff.weeks_behind, fff.weeks_to_catch_up, fff.days_to_catch_up).as_str();
    ret += format!("You should catch up on {}.\n", fff.catch_up_date.format("%d %b, %Y")).as_str();
    ret += format!("\nSubscribe to this URL in your podcatching app of choice: {}", fff.subscribe_url).as_str();
    ret
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   update_one_handler
 //
 // NOTES:  Update one podcast, specified by folder name or subscribe url
 // ARGS:
 //     podcast - The podcast to update. Specified by the folder name or
 //               the PodRacer subscribe url.
 // RETURN:
 //
#[post("/update/<podcast>")]
pub fn update_one_handler(podcast: String) -> std::io::Result<()> {
    // Update the specified podcast
    // Check if podcast is folder name
    if let Some(mut racer) = racer::get_by_dir_name(&podcast) {
        return racer.update(&racer::RssFile::Download);
    }
    // Check if subscribe url
    if let Some(mut racer) = racer::get_by_url(&podcast) {
        return racer.update(&racer::RssFile::Download);
    }
    Err(Error::new(ErrorKind::NotFound, format!("podcast not found")))
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   update_all_handler
 //
 // NOTES:  Forces update of all podcast feeds on this server
 // ARGS:   None
 // RETURN: A result. If errored, a string containing some error info
 //
#[post("/update")]
pub fn update_all_handler() -> Result<(), String> {
    match racer::update_all() {
        Ok(_) => Ok(()),
        Err(string) => Err(format!("Error in update_all_handler: {}", string)),
    }
}
////////////////////////////////////////////////////////////////////////////////
 // NAME:   delete_feed_handler
 //
 // NOTES:
 //     Deletes the sepecified feed. No authentication required, so anyone can
 //     get in there and cause havoc. Probs should change that.
 // ARGS:   podcast - the podcast to delete. Can be a dir name or PodRacer URL
 // RETURN: Result - strings with info either way
 //
 // Eventually want to expand this to be a button on the web UI after listing all podcasts
//#[post("/delete_feed?<podcast>")]
//pub fn delete_feed_handler(podcast: String) -> Result<String, String> {
    //Err(format!("Need to put this behind some sort of auth so random people can't delete things"))
    // // Try dir name first
    // let mut dir = dirs::home_dir().unwrap();
    // dir.push(racer::PODRACER_DIR);
    // dir.push(&podcast);
    // if dir.is_dir() {
    //     // Delete it and return Ok
    //     match std::fs::remove_dir_all(dir.as_path()) {
    //         Ok(_) => return Ok(format!("Podcast deleted from server: {}", &podcast)),
    //         Err(e) => {
    //             println!("Error removing podcast {} from sever: {}", &podcast, e);
    //             return Err(format!("Error removing podcast from server."));
    //         },
    //     };
    // }
    // Not a dir, search for a FeedRacer that has this URL
    // Err(format!("TODO: search racers for this url: {}", podcast))
//}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   list_feeds_handler
 //
 // NOTES:
 //     List all the feeds on this server. Gives a list of podcasts by the
 //     directory names: <podcast_name>_<rate>_<feed creation date>
 // ARGS:   None
 // RETURN: Result string - either the feeds or info on what failed
 //
#[get("/list_feeds")]
pub fn list_feeds_handler() -> Result<String, String> {
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
////////////////////////////////////////////////////////////////////////////////
 // NAME:   serve_rss_handler
 //
 // NOTES:
 //     Serves the racer.rss file for the specified podcast. This is almost
 //     certainly called by a podcast player to check for new episodes.
 //     TODO: Should we not serve the file if nothing changed? Is there a safe
 //         way to do that?
 // ARGS:   podcast - The podcast to serve. Format is the folder name
 // RETURN: Our PodRacer RSS file
 //
#[get("/podcasts/<podcast>/racer.rss")]
pub fn serve_rss_handler(podcast: String) -> Option<File> {
    println!("Serving at {}", chrono::Utc::now().to_rfc3339());
    // Serve the rss file
    let home = dirs::home_dir()?;
    let path: PathBuf = [home.to_str()?, racer::PODRACER_DIR, &podcast, racer::RACER_RSS_FILE].iter().collect();
    match std::fs::File::open(path) {
        Ok(file) => Some(file),
        Err(_) => None,
    }
}
