#![warn(rust_2018_idioms)]

////////////////////////////////////////////////////////////////////////////////
//  File:   racer.rs
//
//  © Zach Nielsen 2020
//  Items pertaining to rss manipulation
//

////////////////////////////////////////////////////////////////////////////////
//  Included Modules
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
pub const SCHEMA_VERSION: &'static str = "1.0.0";
// pub const PODRACER_DIR: &'static str = ".podracer";
pub const PODRACER_DIR: &'static str = ".config/pod_racer";

pub const ORIGINAL_RSS_FILE: &'static str = "original.rss";
pub const RACER_RSS_FILE: &'static str = "racer.rss";
pub const RACER_FILE: &'static str = "racer.file";
pub const INDENT_AMOUNT: usize = 2; // For pretty printing rss files
pub const SPACE_CHAR: u8 = 32; // ASCII ' '

// All parameters we need to create a PodRacer feed
pub struct RacerCreationParams {
    pub address: String,
    pub port: u64,
    pub url: String,
    pub rate: f32,
    pub start_ep: usize,
}

pub struct UpdateMetadata {
    pub num_updated: u64,
    pub time: std::time::Duration,
    pub num_with_new_eps: u64,
}

// Should we attempt to download the original RSS file, or just look at what we have?
// This is pretty much only used to prevent a refetch when creating a new feed.
pub enum RssFile {
    Download,
    FromStorage,
}

// Metadata about when each episode will be published
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RacerEpisode {
    ep_num: Option<i64>,
    date: String,
    title: Option<String>,
}

// All the fields of our racer file. Info we might want across sessions.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FeedRacer {
    schema_version: String,
    podcast_title: Option<String>,
    racer_path: PathBuf,
    source_url: String,
    subscribe_url: String,
    anchor_date: DateTime<chrono::Utc>,
    first_pubdate: DateTime<chrono::FixedOffset>,
    rate: f32,
    release_dates: Vec<RacerEpisode>,
}
// Basic getter functions
impl FeedRacer {
    // pub fn get_schema_version(&self) -> &str { &self.schema_version }
    // pub fn get_release_dates(&self) -> &Vec<RacerEpisode> { &self.release_dates }
    pub fn get_first_pubdate(&self) -> DateTime<chrono::FixedOffset> {
        self.first_pubdate
    }
    pub fn get_subscribe_url(&self) -> &str {
        &self.subscribe_url
    }
    pub fn get_anchor_date(&self) -> DateTime<chrono::Utc> {
        self.anchor_date
    }
    pub fn get_racer_path(&self) -> &Path {
        &self.racer_path
    }
    pub fn get_racer_name(&self) -> &std::ffi::OsStr {
        self.racer_path.file_name().unwrap()
    }
    pub fn get_podcast_title(&mut self) -> String {
        if let Some(title) = &self.podcast_title {
            title.to_owned()
        }
        else {
            let racer_path = self.racer_path.to_str().unwrap();
            let original_rss_path: PathBuf = [racer_path, ORIGINAL_RSS_FILE].iter().collect();
            let original_rss_file = File::open(&original_rss_path).unwrap();
            let buff = std::io::BufReader::new(&original_rss_file);
            let rss = rss::Channel::read_from(buff).unwrap();
            self.podcast_title = Some(rss.title().to_owned());
            rss.title().to_owned()
        }
    }
    pub fn get_source_url(&self) -> &str {
        &self.source_url
    }
    pub fn get_rate(&self) -> f32 {
        self.rate
    }
}

impl FeedRacer {
    ////////////////////////////////////////////////////////////////////////////////
    //  NAME:   FeedRacer::new
    //
    //  NOTES:
    //      Creates a new feedracer object. This involves parsing all the items from
    //      a feed + creating a transformed list of publish dates (shift + squish/stretch).
    //      The returned object is all ready to be written to disk as a json.
    //  ARGS:
    //      items - All the episodes to publish
    //      params - The input parameters for this feed
    //      dir - The directory for the new feed
    //  RETURN: A new, initialized FeedRacer object.
    //
    fn new(items: &mut Vec<rss::Item>, params: &RacerCreationParams, dir: &str) -> FeedRacer {
        // Reverse the items so the oldest entry is first
        items.reverse();
        // Get anchor date
        let start_idx = match items.len() >= params.start_ep && params.start_ep > 0 {
            true => params.start_ep - 1,
            false => {
                println!("Invalid index for parameter start_ep.");
                println!(
                    "Given start_ep {}, but feed only has {} items.",
                    params.start_ep,
                    items.len()
                );
                0
            }
        };
        let start_date = items[start_idx].pub_date().unwrap();
        let first_pubdate = DateTime::parse_from_rfc2822(start_date).unwrap();
        let anchor_date = chrono::Utc::now();
        let podcast_dir_name = Path::new(dir).file_name().unwrap().to_str().unwrap();
        let subscribe_url: PathBuf = [
            &params.address,
            "podcasts",
            podcast_dir_name,
            RACER_RSS_FILE,
        ]
        .iter()
        .collect();

        let mut racer_data = FeedRacer {
            schema_version: SCHEMA_VERSION.to_owned(),
            podcast_title: None,
            racer_path: PathBuf::from(dir),
            source_url: params.url.to_owned(),
            subscribe_url: subscribe_url.to_str().unwrap().to_owned(),
            rate: params.rate.to_owned(),
            anchor_date: anchor_date,
            first_pubdate: first_pubdate,
            release_dates: Vec::new(),
        };
        racer_data.render_release_dates(items);

        racer_data
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //  NAME:   FeedRacer::update
    //
    //  NOTES:  Update this feedracer object. Fetches the upstream file.
    //         Must not panic.
    //  ARGS:   preferred_mode - Whether we prefer to download or use the stored rss file
    //  RETURN: Result - I/O successful or not
    //
    pub fn update(&mut self, preferred_mode: &RssFile) -> std::io::Result<bool> {
        // Get original rss feed
        let (mut rss, new_episodes) = self.get_original_rss(preferred_mode)?;

        // Re-render in case of rate change
        // Probably won't need this in the future
        let mut items = rss.items().to_owned();
        // Sorts ascending order
        items.sort_by(|a, b| rss_item_cmp(a, b));
        self.render_release_dates(&items);

        // Tack on a `- PodRacer` to the title
        rss.set_title(String::from(rss.title()) + " - PodRacer");

        // Drain the items we aren't publishing yet
        let mut items_to_publish: Vec<rss::Item> =
            items.drain(..self.get_num_to_publish()).collect();

        // Append the next item's publish date to the podcast description
        let next_pub_date_str = if items.len() > 0 {
            let next_item = self.release_dates[self.get_num_to_publish()].clone();
            let s = match DateTime::parse_from_rfc2822(&next_item.date) {
                Ok(val) => val.with_timezone(&Local).format("%d %b %Y at %I:%M%P"),
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            };
            format!("Next episode publishes {}.", s)
        } else {
            format!("PodRacer feed has caught up.")
        };
        rss.set_description(format!("{} -- {}", rss.description(), &next_pub_date_str));

        // Append racer publish date to the end of the description
        for (item, info) in items_to_publish.iter_mut().zip(self.release_dates.iter()) {
            //
            // Get all the DateTime's we need
            //
            let racer_date = match DateTime::parse_from_rfc2822(&info.date) {
                Ok(val) => val,
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            };
            let item_pub_date = match item.pub_date() {
                Some(val) => val,
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "no pub_date on item",
                    ))
                }
            };
            let item_date = match DateTime::parse_from_rfc2822(item_pub_date) {
                Ok(val) => val,
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            };

            // If we have caught up, use the actual publish date because the racer date
            // will be in the past, which won't make much sense as a publish date
            let racer_pub_date = if racer_date < item_date {
                item_date.with_timezone(&Local).to_rfc2822()
            } else {
                racer_date.with_timezone(&Local).to_rfc2822()
            };
            let original_pub_date = match DateTime::parse_from_rfc2822(item_pub_date) {
                Ok(val) => val.with_timezone(&Local).format("%d %b %Y"),
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            };
            item.set_pub_date(racer_pub_date);

            //
            // Set description + content
            //
            let description = item.description().unwrap_or("").to_owned();
            let mut new_description = description.replace("\r\n", "\n");
            new_description.push_str(&format!("\n\nOriginally published on {}", original_pub_date));
            item.set_description(new_description);
            // Only do content if it is present
            match item.content() {
                Some(content) => {
                    let mut new_content = content.replace("\r\n", "\n");
                    new_content.push_str(&format!("<br><br>Originally published on {}", original_pub_date));
                    item.set_content(new_content);
                },
                None => (),
            };
        }
        // Now that we have the items we want, overwrite the objects items.
        rss.set_items(items_to_publish);

        correct_known_rss_issues(&mut rss);

        // Write out the racer.file
        match self.write_to_file() {
            Ok(_) => (),
            Err(e) => println!("Error writing the racer.file, but still continuing: {}.", e),
        }

        // Write out the racer.rss file
        let racer_path = match self.racer_path.to_str() {
            Some(val) => val,
            None => {
                println!(
                    "Error getting self.racer_path as a str. Very not good. Using the tmp dir."
                );
                "/tmp"
            }
        };
        let racer_rss_path: PathBuf = [racer_path, RACER_RSS_FILE].iter().collect();
        let racer_rss_file = File::create(&racer_rss_path)?;
        match rss.pretty_write_to(racer_rss_file, SPACE_CHAR, INDENT_AMOUNT) {
            Ok(_) => {
                // Need to scrub on write since pretty_write doesn't write valid xml
                // Should be fixed with GH-33
                //scrub_xml_file(&racer_rss_path);
                Ok(new_episodes)
            },
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //  NAME:   FeedRacer::render_release_dates
    //
    //  NOTES:  Renders the release dates for the passed in items. Items must be in the correct order.
    //  ARGS:   items - The items to render. Must be in the correct order.
    //  RETURN: None
    //
    fn render_release_dates(&mut self, items: &Vec<rss::Item>) {
        self.release_dates = Vec::new();
        let mut item_counter = 1;
        for item in items {
            // Get diff from first published date
            let pub_date = item.pub_date().unwrap();
            let mut time_diff = DateTime::parse_from_rfc2822(pub_date)
                .unwrap()
                .signed_duration_since(self.first_pubdate)
                .num_seconds();
            // Scale that diff
            time_diff = ((time_diff as f32) / self.rate) as i64;
            // Add back to anchor date to get new publish date + convert to string
            let racer_date = self
                .anchor_date
                .checked_add_signed(Duration::seconds(time_diff))
                .unwrap()
                .to_rfc2822();
            // Add to vector of dates
            self.release_dates.push(RacerEpisode {
                ep_num: Some(item_counter),
                title: Some(item.title().unwrap_or("[no title]").to_owned()),
                date: racer_date,
            });
            item_counter += 1;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //  NAME:   FeedRacer::write_to_file
    //
    //  NOTES:  Writes the racer to a file in JSON format
    //  ARGS:   None
    //  RETURN: Result - I/O successful or not
    //
    fn write_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;

        let filename: PathBuf = [self.racer_path.to_str().unwrap(), RACER_FILE]
            .iter()
            .collect();
        let mut fp = File::create(filename)?;
        fp.write_all(json.as_bytes())
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    //  NAME:   FeedRacer::get_original_rss
    //
    //  NOTES:
    //      Gets the original rss one way or another (downloaded or from storage).
    //      We try to avoid downloading if possible. If we have the file on disk and the feed
    //      doesn't integrate new episodes, there's no need to download so we can just serve back
    //      what we have on disk. If either of those things is not true, we need to fetch to update
    //      the rss feed, but we only overwrite the stored file if it has more feed items.
    // TODO -> handle feeds that have a constant number of entries and push the oldest entry out.
    // TODO -> Wrap this return in an option for the case where there is no stored file and the
    //         network request fails.
    //  ARGS:
    //      preferred_mode - the requested mode. We don't always honnor it, but it lets us know if the asker
    //      wants to go to the network or not.
    //  RETURN: A tuple - the original rss channel + if there were new episodes to publish
    //

    fn get_original_rss(
        &mut self,
        preferred_mode: &RssFile,
    ) -> std::io::Result<(rss::Channel, bool)> {
        let mut stored_rss_path = self.racer_path.clone();
        stored_rss_path.push(ORIGINAL_RSS_FILE);
        let stored_rss_file = match File::open(&stored_rss_path) {
            Ok(val) => Some(val),
            Err(e) => {
                println!("Error opening original rss file ({}): {}", stored_rss_path.display(), e);
                None
            }
        };
        let stored_rss = if let Some(f) = stored_rss_file {
            let buf_reader = BufReader::new(f);
            match rss::Channel::read_from(buf_reader) {
                Ok(val) => Some(val),
                Err(e) => {
                    println!("Error reading rss channel from disk: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let (stored_rss, functional_mode) = match stored_rss {
            Some(val) => {
                // We do have a file on disk, so use whatever the requested mode was
                (Some(val), preferred_mode)
            }
            None => {
                // If we don't have a stored file, we have to download.
                (None, &RssFile::Download)
            }
        };

        match functional_mode {
            RssFile::Download => {
                match download_rss_channel(&self.source_url) {
                    Ok(network_file) => {
                        // Compare to stored file - update if we need to
                        let num_to_update = match &stored_rss {
                            Some(rss) => {
                                (network_file.items().len() as i64 - rss.items().len() as i64).abs()
                            }
                            None => network_file.items().len() as i64,
                        };
                        if num_to_update > 0 {
                            // Overwrite our stored original RSS file
                            // TODO - If a feed pushes out the oldest entries, overwriting won't cut it.
                            //        We'll need to save old items.
                            match File::create(stored_rss_path) {
                                Ok(stored_rss_file) => {
                                    match network_file.pretty_write_to(stored_rss_file, SPACE_CHAR, INDENT_AMOUNT) {
                                        Ok(_) => (),
                                        Err(e) => println!("Error writing network_file to disk: {}. Continuing without writing.", e),
                                    };
                                },
                                Err(e) => println!("Error during File::create(stored_rss_path): {}. Continuing without writing.", e),
                            };
                        }
                        return Ok((network_file, num_to_update > 0));
                    }
                    Err(e) => {
                        println!("Could not get network file: {}", e);
                        println!("Resuming with stored rss file");
                        // Panics if there was no stored rss and the network failed
                        if let Some(stored_rss) = stored_rss {
                            return Ok((stored_rss, false));
                        }
                    }
                };
            }
            RssFile::FromStorage => {
                if let Some(stored_rss) = stored_rss {
                    return Ok((stored_rss, false));
                }
            }
        };

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Error getting original rss",
        ))
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //  NAME:   FeedRacer::get_num_to_publish
    //
    //  NOTES:
    //      Counts how many items are ready to publish according to this racer's rules.
    //      This function must not panic, as it's used in the update thread
    //  ARGS:   None
    //  RETURN: The number of items that should be published.
    //
    fn get_num_to_publish(&self) -> usize {
        let mut ret = 0;
        // Get today's date
        let now = chrono::Utc::now();
        // Count how many are before todays dates
        for release_date in &self.release_dates {
            let date = chrono::DateTime::parse_from_rfc2822(&release_date.date).unwrap();
            if date.signed_duration_since(now) < chrono::Duration::zero() {
                ret += 1;
            }
        }
        ret
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   rss_item_cmp
//
//  NOTES:
//      A sort function for rss items. Sorts by date. Lots of string stuff for each item, might
//      need to come up with a better solution to this if it ends up being a bottleneck.
//  ARGS:   a/b - The rss::Items to sort
//  RETURN: An ordering
//
fn rss_item_cmp(a: &rss::Item, b: &rss::Item) -> std::cmp::Ordering {
    let a_sec = DateTime::parse_from_rfc2822(a.pub_date().unwrap())
        .unwrap()
        .timestamp();
    let b_sec = DateTime::parse_from_rfc2822(b.pub_date().unwrap())
        .unwrap()
        .timestamp();
    if a_sec < b_sec {
        std::cmp::Ordering::Less
    } else if b_sec < a_sec {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   get_racer_at_path
//
//  NOTES:
//      Grabs a racer object from a racer file located in the specified directory.
//      This function must not panic, as it's used in the update thread.
//  ARGS:   The path to the directory of interest
//  RETURN: The FeedRacer or an error.
//
fn get_racer_at_path(path: &str) -> std::io::Result<FeedRacer> {
    let racer_file_path: PathBuf = [path, RACER_FILE].iter().collect();
    let racer_file = File::open(racer_file_path)?;
    let racer: FeedRacer = serde_json::from_reader(&racer_file)?;
    Ok(racer)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   update_racer_at_path
//
//  NOTES:
//      Updates the items that need to be published for the racer in the given directory
//      This function must not panic, as it's used in the update thread.
//  ARGS:
//      path - the directory of the racer of interest
//      preferred_mode - Whether we prefer to download a fresh copy or not.
//  RETURN: A result. Typically only fails on I/O or network stuff.
//
fn update_racer_at_path(path: &str, preferred_mode: &RssFile) -> std::io::Result<bool> {
    // Load in racer file
    let mut racer = get_racer_at_path(path)?;

    racer.update(preferred_mode)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   get_all_podcast_dirs
//
//  NOTES:
//      Gets all the dirs in the PODRACER_DIR (~/.podracer). Each of these dirs has info for one
//      feed.
//      This function must not panic, as it's used in the update thread.
//  ARGS:   None
//  RETURN: All the items in the podracer dir
//
pub fn get_all_podcast_dirs() -> Result<std::fs::ReadDir, String> {
    let mut dir = match dirs::home_dir() {
        Some(val) => val,
        None => return Err(format!("Error retrieving home dir")),
    };
    dir.push(PODRACER_DIR);
    match Path::read_dir(dir.as_path()) {
        Ok(val) => Ok(val),
        Err(e) => return Err(format!("Cannot access dir: {:?}.\nError: {}", dir, e)),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   update_all
//
//  NOTES:
//      Updates all the racers on this server
//      This function must not panic, as it's used in the update thread.
//  ARGS:   None
//  RETURN: A result containing some metadata about the update or an error string
//
pub fn update_all() -> Result<UpdateMetadata, String> {
    let start = std::time::SystemTime::now();
    let mut counter = 0;
    let mut num_with_new_eps = 0;
    let podcast_dirs = match get_all_podcast_dirs() {
        Ok(val) => val,
        Err(str) => return Err(format!("Error in update_all: {}", str)),
    };
    for podcast_dir in podcast_dirs {
        let path = match podcast_dir {
            Ok(val) => val.path(),
            Err(e) => return Err(format!("Error iterating over path from read_dir: {}", e)),
        };
        let path_str = match path.to_str() {
            Some(val) => val,
            None => return Err(format!("Tried to open empty path")),
        };
        match update_racer_at_path(path_str, &RssFile::Download) {
            Ok(new_eps) => {
                if new_eps {
                    num_with_new_eps += 1;
                }
            }
            Err(e) => {
                return Err(format!(
                    "Could not update path {}. Error was: {}",
                    path_str, e
                ))
            }
        };
        counter += 1;
    }
    let end = std::time::SystemTime::now();
    let duration = match end.duration_since(start) {
        Ok(val) => val,
        Err(e) => {
            println!("Error getting update_all duration: {}", e);
            std::time::Duration::new(0, 0)
        }
    };
    Ok(UpdateMetadata {
        num_updated: counter,
        time: duration,
        num_with_new_eps: num_with_new_eps,
    })
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed
//
//  NOTES:  Creates a new feedracer object + sets up the diretcory and files
//  ARGS:   params - All the params needed to make a racer
//  RETURN: A FeedRacer or error String
//
pub fn create_feed(params: &mut RacerCreationParams) -> Result<FeedRacer, String> {
    if None == params.url.find("http") {
        params.url = String::from("https://") + &params.url;
    }
    let rss = match download_rss_channel(&params.url) {
        Ok(val) => val,
        Err(e) => return Err(format!("Error downloading rss feed: {}", e)),
    };

    // Make directory
    let dir = create_feed_racer_dir(&rss, &params);
    // Write out original rss feed to file in dir
    let original_rss_file = match File::create(String::from(&dir) + "/" + ORIGINAL_RSS_FILE) {
        Ok(val) => val,
        Err(e) => return Err(format!("Unable to create file: {}", e)),
    };
    match rss.pretty_write_to(original_rss_file, SPACE_CHAR, 2) {
        Ok(_) => (),
        Err(e) => return Err(format!("unable to write original rss file: {}", e)),
    };
    // Make racer file
    let racer = FeedRacer::new(&mut rss.items().to_owned(), &params, &dir);
    match racer.write_to_file() {
        Ok(_) => (),
        Err(e) => return Err(format!("failed with error: {}", e)),
    };
    // Run update() on this directory. We just created it, so no need to refresh the rss file
    match update_racer_at_path(&dir, &RssFile::FromStorage) {
        Ok(_) => println!(
            "Subscribe to this URL in your pod catcher: {}",
            racer.get_subscribe_url()
        ),
        Err(e) => return Err(format!("Error writing file: {}", e)),
    };

    Ok(racer)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   create_feed_racer_dir
//
//  NOTES:  Creates the direcotry for a racer with these parameters
//  ARGS:
//      ch - An rss channel (used for the title of the show)
//      params - Input parameters
//  RETURN: The path of the created directory
//
fn create_feed_racer_dir(ch: &rss::Channel, params: &RacerCreationParams) -> String {
    let day = chrono::Utc::today();
    // Create this feed's dir name
    let scrubbed_pod_name = &ch
        .title()
        .to_lowercase()
        .replace(" ", "-")
        .replace("/", "-")
        .replace(":", "");
    let mut dir = String::from(dirs::home_dir().unwrap().to_str().unwrap());
    dir.push_str("/");
    dir.push_str(PODRACER_DIR);
    dir.push_str("/");
    dir.push_str(scrubbed_pod_name);
    dir.push_str("_");
    dir.push_str(&params.rate.to_string());
    dir.push_str("_ep");
    dir.push_str(&params.start_ep.to_string());
    dir.push_str("_");
    dir.push_str(&day.format("%Y-%m-%d").to_string());
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   download_rss_channel
//
//  NOTES:
//      Handles the network stuff for getting an rss file from the network.
//  ARGS:   url - the url of the file to get
//  RETURN: A channel or error information
//
fn download_rss_channel(url: &str) -> Result<rss::Channel, Box<dyn std::error::Error>> {
    let content = match reqwest::blocking::get(url) {
        Ok(val) => match val.bytes() {
            Ok(val) => val,
            Err(e) => return Err(Box::new(e)),
        },
        Err(e) => return Err(Box::new(e)),
    };

    //
    // Scrub the file
    //
    let buf_content = BufReader::new(&content[..]);
    let tmp_file_name = "/tmp/scrubbed.rss".to_owned();
    let scrubbed_file = File::create(&tmp_file_name).expect("Failed to create tmp scrub file");
    scrub_xml_content_to_file(buf_content, &scrubbed_file);
    let scrubbed_file = File::open(&tmp_file_name).expect("Failed to open tmp scrub file");
    let scrubbed_buff = std::io::BufReader::new(&scrubbed_file);

    match rss::Channel::read_from(scrubbed_buff) {
        Ok(channel) => Ok(channel),
        Err(e) => {
            println!("Failure when downloading rss channel");
            Err(Box::new(e))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   get_by_dir_name
//
//  NOTES:  Check if the specified directory hosts a FeedRacer + return it.
//  ARGS:   target_dir: the name of the directory to check
//  RETURN: A FeedRacer or None
//
pub fn get_by_dir_name(target_dir: &str) -> Option<FeedRacer> {
    let mut dir = match dirs::home_dir() {
        Some(val) => val,
        None => {
            println!("Error retrieving home dir");
            return None;
        }
    };
    dir.push(PODRACER_DIR);
    dir.push(target_dir);
    if dir.is_dir() {
        let racer = get_racer_at_path(dir.as_path().to_str().unwrap()).unwrap();
        return Some(racer);
    }
    println!("{:?} is not a racer dir", dir);
    None
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   get_by_url
//
//  NOTES:  Check all racers on this server to see if we have one with this URL
//  ARGS:   url - the racer url to check for
//  RETURN: A FeedRacer or None
//
pub fn get_by_url(url: &str) -> Option<FeedRacer> {
    let dirs = get_all_podcast_dirs().unwrap();
    for dir_res in dirs {
        let dir = dir_res.unwrap();
        let racer = get_racer_at_path(dir.path().to_str().unwrap()).unwrap();
        if racer.get_subscribe_url() == url {
            return Some(racer);
        };
    }
    None
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   get_all_racers
//
//  NOTES:  Gets all the racers on this server.
//  ARGS:   None
//  RETURN: A vector of the racers on this server, or an error.
//
pub fn get_all_racers() -> Result<Vec<FeedRacer>, String> {
    let mut racers = Vec::new();

    // Get all folders in the podracer dir
    let podcast_dirs = match get_all_podcast_dirs() {
        Ok(val) => val,
        Err(str) => {
            println!("Error in list_feeds_handler: {}", str);
            return Err(format!("Error getting feeds, check logs"));
        }
    };
    for podcast_dir_res in podcast_dirs {
        let podcast_dir = match podcast_dir_res {
            Ok(val) => val,
            Err(e) => {
                return Err(format!(
                    "Error iterating over path from get_all_podcast_dirs: {}",
                    e
                ))
            }
        };
        let path = podcast_dir.path();
        let path = match path.to_str() {
            Some(val) => val,
            None => return Err(format!("Error converting podcast_dir path to string")),
        };
        let racer = match get_racer_at_path(path) {
            Ok(val) => val,
            Err(e) => return Err(format!("Error getting racer_at_path: {}", e)),
        };
        racers.push(racer);
    }

    Ok(racers)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   scrub_xml_content_to_file
//
//  NOTES:
//      Some rss feeds don't properly escape things. Properly escape known issues.
//      This is not really scalable, but if I'm the only one using it then it should be more or
//      less fine.
//  ARGS:   file_name - The file to scrub and replace
//  RETURN: None
//
pub fn scrub_xml_content_to_file<B: BufRead>(in_buf: B, file: &File) {
    // Known bad strings
    let subs: std::collections::HashMap<String, String> =
        [("& ".to_owned(), "&amp; ".to_owned())]
        .iter().cloned().collect();
    //subs.insert("&source".to_owned(), "&amp;source".to_owned());
    //subs.insert("&stitched".to_owned(), "&amp;stitched".to_owned());

    let mut out_buf = std::io::BufWriter::new(file);
    in_buf
        .lines()
        .map(|line_res| {
            line_res.and_then(|mut line| {
                for (key, val) in &subs {
                    if line.contains(key) {
                        line = line.replace(key, val);
                    }
                }
                line.push_str("\n");
                out_buf.write_all(line.as_bytes())
            })
        })
        .collect::<Result<(), _>>()
        .expect("IO failed");
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  NAME:   scrub_xml_file
//
//  NOTES:
//      Some rss feeds don't properly escape things. Properly escape known issues.
//      This is not really scalable, but if I'm the only one using it then it should be more or
//      less fine.
//  ARGS:   file_name - The file to scrub and replace
//  RETURN: None
//
pub fn scrub_xml_file(file_name: &PathBuf) {
    let tmp_file_name = "/tmp/scrubbed.rss".to_owned();
    let scrubbed_file = File::create(&tmp_file_name).expect("Failed to create tmp scrub file");
    let file = File::open(file_name).expect("Could not open original file");
    let in_buf = std::io::BufReader::new(&file);
    scrub_xml_content_to_file(in_buf, &scrubbed_file);

    // Replace original with scrubbed file
    std::fs::rename(
        std::path::Path::new(&tmp_file_name),
        std::path::Path::new(&file_name),
    )
    .expect("Failed to overwrite file");
}


pub fn correct_known_rss_issues(rss: &mut rss::Channel) {
    // Check  for itunes:owner element itunes:email
    match &mut rss.itunes_ext {
        Some(itunes) => {
            match &mut itunes.owner {
                Some(owner) => {
                    if owner.name.is_some() {
                        if owner.email.is_none() {
                            owner.set_email("example@example.com".to_owned());
                        }
                    }
                },
                None => (),
            }
        },
        None => (),
    }

    // Remove <media:rights status="userCreated" />
}

//
// Display implementation
//
impl fmt::Display for RacerEpisode {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(
            f,
            "ep_num: {}, date: {}, title: {}",
            self.ep_num.as_ref().unwrap_or(&0),
            self.date,
            self.title.as_ref().unwrap_or(&"[none]".to_string())
        )
    }
}
impl fmt::Display for FeedRacer {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        writeln!(f, "schema_version: {}", self.schema_version)?;
        writeln!(f, "racer_path: {}", self.racer_path.display())?;
        writeln!(f, "source_url: {}", self.source_url)?;
        writeln!(f, "subscribe_url: {}", self.subscribe_url)?;
        writeln!(f, "anchor_date: {}", self.anchor_date)?;
        writeln!(f, "first_pubdate: {}", self.first_pubdate)?;
        writeln!(f, "rate: {}", self.rate)?;
        writeln!(f, "release_dates {{")?;
        for entry in self.release_dates.as_slice() {
            writeln!(f, "\t{},", entry)?;
        }
        writeln!(f, "}}")
    }
}
