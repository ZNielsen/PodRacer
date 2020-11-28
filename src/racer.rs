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
use chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fmt;
use std::io::{BufReader, Write};
use std::fs::File;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
pub const SCHEMA_VERSION:    &'static str = "1.0.0";
pub const PODRACER_DIR:      &'static str = ".podracer";

pub const ORIGINAL_RSS_FILE: &'static str = "original.rss";
pub const RACER_RSS_FILE:    &'static str = "racer.rss";
pub const RACER_FILE:        &'static str = "racer.file";
pub const INDENT_AMOUNT:            usize = 2;  // For pretty printing rss files
pub const SPACE_CHAR:                  u8 = 32; // ASCII ' '

// All parameters we need to create a PodRacer feed
pub struct RacerCreationParams {
    pub address: String,
    pub port: u64,
    pub url: String,
    pub rate: f32,
    pub integrate_new: bool,
    pub start_ep: usize,
}

// Should we attempt to download the original RSS file, or just look at what we have?
// This is pretty much only used to prevent a refetch when creating a new feed.
pub enum RssFile {
    Download,
    FromStorage,
}

// Metadata about when each episode will be published
#[derive(Serialize, Deserialize, Debug)]
pub struct RacerEpisode {
    ep_num: i64,
    date: String,
}

// All the fields of our racer file. Info we might want across sessions.
#[derive(Serialize, Deserialize, Debug)]
pub struct FeedRacer {
    schema_version: String,
    racer_path: PathBuf,
    source_url: String,
    subscribe_url: String,
    anchor_date: DateTime<chrono::Utc>,
    first_pubdate: DateTime<chrono::FixedOffset>,
    rate: f32,
    integrate_new: bool,
    release_dates: Vec<RacerEpisode>
}
// Basic getter functions
impl FeedRacer {
    // pub fn get_schema_version(&self) -> &str { &self.schema_version }
    pub fn get_racer_path(&self) -> &Path { &self.racer_path }
    pub fn get_racer_name(&self) -> &std::ffi::OsStr { self.racer_path.file_name().unwrap() }
    // pub fn get_source_url(&self) -> &str { &self.source_url }
    pub fn get_subscribe_url(&self) -> &str { &self.subscribe_url }
    // pub fn get_anchor_date(&self) -> DateTime<chrono::Utc> { self.anchor_date }
    pub fn get_first_pubdate(&self) -> DateTime<chrono::FixedOffset> { self.first_pubdate }
    pub fn get_rate(&self) -> f32 { self.rate }
    // pub fn get_integrate_new(&self) -> bool { self.integrate_new }
    // pub fn get_release_dates(&self) -> &Vec<RacerEpisode> { &self.release_dates }
}

impl FeedRacer {
    ////////////////////////////////////////////////////////////////////////////////
     // NAME:   FeedRacer::new
     //
     // NOTES:
     //     Creates a new feedracer object. This involves parsing all the items from
     //     a feed + creating a transformed list of publish dates (shift + squish/stretch).
     //     The returned object is all ready to be written to disk as a json.
     // ARGS:
     //     items - All the episodes to publish
     //     params - The input parameters for this feed
     //     dir - The directory for the new feed
     // RETURN: A new, initialized FeedRacer object.
     //
    pub fn new(items: &mut Vec<rss::Item>,
        params: &RacerCreationParams,
        dir: &str) -> FeedRacer {
        // Reverse the items so the oldest entry is first
        items.reverse();
        // Get anchor date
        let start_idx = match items.len() >= params.start_ep && params.start_ep > 0 {
            true => params.start_ep - 1,
            false => {
                println!("Invalid index for parameter start_ep.");
                println!("Given start_ep {}, but feed only has {} items.", params.start_ep, items.len());
                0
            },
        };
        let start_date = items[start_idx].pub_date().unwrap();
        let first_pubdate = DateTime::parse_from_rfc2822(start_date).unwrap();
        let anchor_date = chrono::Utc::now();
        let mut dates = Vec::new();
        let mut item_counter = 1;
        for item in items {
            // Get diff from first published date
            let pub_date = item.pub_date().unwrap();
            let mut time_diff = DateTime::parse_from_rfc2822(pub_date).unwrap()
                                .signed_duration_since(first_pubdate)
                                .num_seconds();
            // Scale that diff
            time_diff = ((time_diff as f32) / params.rate) as i64;
            // Add back to anchor date to get new publish date + convert to string
            let racer_date = anchor_date.checked_add_signed(Duration::seconds(time_diff)).unwrap()
                                .to_rfc2822();
            // Add to vector of dates
            dates.push( RacerEpisode {
                ep_num: item_counter,
                date: racer_date,
            });
            item_counter += 1;
        }

        let podcast_dir_name = Path::new(dir).file_name().unwrap().to_str().unwrap();
        let subscribe_url: PathBuf = [
                            &params.address,
                            "podcasts",
                            podcast_dir_name,
                            RACER_RSS_FILE].iter().collect();

        let racer_data = FeedRacer {
            schema_version: SCHEMA_VERSION.to_owned(),
            racer_path: PathBuf::from(dir),
            source_url: params.url.to_owned(),
            subscribe_url: subscribe_url.to_str().unwrap().to_owned(),
            rate: params.rate.to_owned(),
            anchor_date: anchor_date,
            first_pubdate: first_pubdate,
            integrate_new: params.integrate_new.to_owned(),
            release_dates: dates,
        };

        racer_data
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
     // NAME:   FeedRacer::write_to_file
     //
     // NOTES:  Writes the racer to a file in JSON format
     // ARGS:   None
     // RETURN: Result - I/O successful or not
     //
    pub fn write_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;

        let filename: PathBuf = [self.racer_path.to_str().unwrap(), RACER_FILE].iter().collect();
        let mut fp = File::create(filename)?;
        fp.write_all(json.as_bytes())
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
     // NAME:   FeedRacer::add_new_items
     //
     // NOTES:  Appends items to this racer
     // ARGS:
     //     items - new items to add to this racer object
     //     curren_len: the counter value to start at. Probably don't need this? Can be refactored.
     // RETURN: None, modifies self
     //
    pub fn add_new_items(&mut self, items: &Vec<rss::Item>, current_len: usize) {
        let mut item_counter = (current_len - 1) as i64;
        for item in items {
            // Get diff from first published date
            let pub_date = item.pub_date().unwrap();
            let mut time_diff = DateTime::parse_from_rfc2822(pub_date).unwrap()
                                    .signed_duration_since(self.first_pubdate)
                                    .num_seconds();
            // Scale that diff
            time_diff = ((time_diff as f32) / self.rate) as i64;
            // Add back to anchor date to get new publish date + convert to string
            let racer_date = self.anchor_date
                                .checked_add_signed(Duration::seconds(time_diff)).unwrap()
                                .to_rfc2822();
            // Add to vector of dates
            self.release_dates.push(RacerEpisode {
                ep_num: item_counter,
                date: racer_date,
            });
            item_counter += 1;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
     // NAME:   FeedRacer::get_original_rss
     //
     // NOTES:
     //     Gets the original rss one way or another (downloaded or from storage).
     //     We try to avoid downloading if possible. If we have the file on disk and the feed
     //     doesn't integrate new episodes, there's no need to download so we can just serve back
     //     what we have on disk. If either of those things is not true, we need to fetch to update
     //     the rss feed, but we only overwrite the stored file if it has more feed items.
     // TODO -> handle feeds that have a constant number of entries and push the oldest entry out.
     // TODO -> Wrap this return in an option for the case where there is no stored file and the
     //         network request fails.
     // ARGS:
     //     preferred_mode - the requested mode. We don't always honnor it, but it lets us know if the asker
     //     wants to go to the network or not.
     // RETURN: The original rss channel
     //
    pub fn get_original_rss(&mut self, preferred_mode: &RssFile) -> rss::Channel {
        let mut stored_rss_path = self.racer_path.clone();
        stored_rss_path.push(ORIGINAL_RSS_FILE);
        let stored_rss_file = File::open(&stored_rss_path).unwrap();
        let buf_reader = BufReader::new(stored_rss_file);
        let stored_rss = rss::Channel::read_from(buf_reader);

        let (stored_rss, functional_mode) = match stored_rss {
            Ok(val) => {
                // We do have a file on disk, so see if we need to download or not
                match self.integrate_new {
                    true => (Some(val), preferred_mode),   // Preserve mode
                    false => {
                        // Not integrating, don't fetch from network
                        println!("{:?} is not integrating new episodes. Using stored rss file", self.get_racer_name());
                        (Some(val), &RssFile::FromStorage)
                    },
                }
            },
            Err(e) => {
                // If we don't have a stored file, we have to download.
                println!("Couldn't get rss file on disk: {}", e);
                (None, &RssFile::Download)
            },
        };

        match functional_mode {
            RssFile::Download => {
                match download_rss_channel(&self.source_url) {
                    Ok(network_file) => {
                        // Compare to stored file - update if we need to
                        let num_to_update = match &stored_rss {
                            Some(rss) => (network_file.items().len() as i64 - rss.items().len() as i64).abs(),
                            None => network_file.items().len() as i64,
                        };
                        if num_to_update > 0 {
                            // Overwrite our stored original RSS file
                            let stored_rss_file = File::create(stored_rss_path).unwrap();
                            network_file.pretty_write_to(stored_rss_file, SPACE_CHAR, INDENT_AMOUNT).unwrap();
                            // Append new entries to our racer object
                            let mut new_items = network_file.items().to_owned();
                            new_items.truncate(num_to_update as usize);
                            let next_ep_num = match &stored_rss {
                                Some(rss) => rss.items().len(),
                                None => 1,
                            };
                            self.add_new_items(&new_items, next_ep_num);
                        }
                        return network_file
                    },
                    Err(e) => {
                        println!("Could not get network file: {}", e);
                        println!("Resuming with stored rss file");
                        // Panics if there was no stored rss and the network failed
                        return stored_rss.unwrap()
                    },
                };
            },
            RssFile::FromStorage => return stored_rss.unwrap(), // Should not panic if mode checks above are correct
        };
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
     // NAME:   FeedRacer::get_num_to_publish
     //
     // NOTES:
     //     Counts how many items are ready to publish according to this racer's rules.
     //     This function must not panic, as it's used in the update thread
     // ARGS:   None
     // RETURN: The number of items that should be published.
     //
    pub fn get_num_to_publish(&self) -> usize {
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

    pub fn update(&mut self, preferred_mode: &RssFile) -> std::io::Result<()> {
        // Get original rss feed
        let mut rss = self.get_original_rss(preferred_mode);

        // Tack on a `- PodRacer` to the title
        rss.set_title(String::from(rss.title()) + " - PodRacer");

        // Check how many episodes we should publish at this point
        let num_to_pub = self.get_num_to_publish();
        let num_to_scrub = rss.items().len() - num_to_pub;
        // Drain the items we aren't publishing yet - TODO: Can we do this in place with slices?
        let mut items_to_publish = rss.items().to_owned();
        items_to_publish.drain(0..num_to_scrub);
        items_to_publish.sort_by(|a, b| rss_item_cmp(a,b));

        // Append racer publish date to the end of the description
        for (item, info) in items_to_publish.iter_mut().zip(self.release_dates.iter()) {
            let date = format!("{}", DateTime::parse_from_rfc2822(&info.date).unwrap()
                        .format("%d %b %Y"));
            item.set_description(
                item.description().unwrap_or("").to_owned() +
                "<br><br>" +
                "PodRacer published on " +
                &date +
                " (UTC)"
            );
        }
        // Now that we have the items we want, overwrite the objects items.
        rss.set_items(items_to_publish);

        // Write out the racer.rss file
        let racer_rss_path: PathBuf = [self.racer_path.to_str().unwrap(), RACER_RSS_FILE].iter().collect();
        let racer_rss_file = File::create(racer_rss_path)?;
        match rss.pretty_write_to(racer_rss_file, SPACE_CHAR, INDENT_AMOUNT) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

fn rss_item_cmp(a: &rss::Item, b: &rss::Item) -> std::cmp::Ordering {
    let a_sec = DateTime::parse_from_rfc2822(a.pub_date().unwrap()).unwrap()
                .timestamp();
    let b_sec = DateTime::parse_from_rfc2822(b.pub_date().unwrap()).unwrap()
                .timestamp();
    if a_sec < b_sec {
        std::cmp::Ordering::Less
    }
    else if b_sec < a_sec {
        std::cmp::Ordering::Greater
    }
    else {
        std::cmp::Ordering::Equal
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
 // NAME:   get_racer_at_path
 //
 // NOTES:
 //     Grabs a racer object from a racer file located in the specified directory.
 //     This function must not panic, as it's used in the update thread.
 // ARGS:   The path to the directory of interest
 // RETURN: The FeedRacer or an error.
 //
pub fn get_racer_at_path(path: &str) -> std::io::Result<FeedRacer> {
    let racer_file_path: PathBuf = [path, RACER_FILE].iter().collect();
    let racer_file = File::open(racer_file_path)?;
    let racer: FeedRacer = serde_json::from_reader(&racer_file)?;
    Ok(racer)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
 // NAME:   update_racer_at_path
 //
 // NOTES:
 //     Updates the items that need to be published for the racer in the given directory
 //     This function must not panic, as it's used in the update thread.
 // ARGS:
 //     path - the directory of the racer of interest
 //     preferred_mode - Whether we prefer to download a fresh copy or not.
 // RETURN: A result. Typically only fails on I/O or network stuff.
 //
pub fn update_racer_at_path(path: &str, preferred_mode: &RssFile) -> std::io::Result<()> {
    // Load in racer file
    let mut racer = get_racer_at_path(path)?;

    racer.update(preferred_mode)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
 // NAME:   get_all_podcast_dirs
 //
 // NOTES:
 //     Gets all the dirs in the PODRACER_DIR (~/.podracer). Each of these dirs has info for one
 //     feed.
 //     This function must not panic, as it's used in the update thread.
 // ARGS:   None
 // RETURN: All the items in the podracer dir
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
 // NAME:   update_all
 //
 // NOTES:
 //     Updates all the racers on this server
 //     This function must not panic, as it's used in the update thread.
 // ARGS:   None
 // RETURN: A result containing an error string
 //
pub fn update_all() -> Result<(), String> {
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
            Ok(()) => (),
            Err(e) => return Err(format!("Could not update path {}. Error was: {}", path_str, e)),
        };
    };
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed
 //
 // NOTES:  Creates a new feedracer object + sets up the diretcory and files
 // ARGS:   params - All the params needed to make a racer
 // RETURN: A FeedRacer or error String
 //
pub fn create_feed(params: &mut RacerCreationParams) -> Result<FeedRacer, String> {
    if None == params.url.find("http") {
        params.url = String::from("https://") + &params.url;
    }
    let channel = match download_rss_channel(&params.url) {
        Ok(val) => val,
        Err(e) => return Err(format!("Error downloading rss feed: {}", e)),
    };

    // Make directory
    let dir = create_feed_racer_dir(&channel, &params);
    // Write out original rss feed to file in dir
    let original_rss_file = match File::create(String::from(&dir) + "/" + crate::racer::ORIGINAL_RSS_FILE) {
        Ok(val) => val,
        Err(e) => return Err(format!("Unable to create file: {}", e)),
    };
    match channel.pretty_write_to(original_rss_file, crate::racer::SPACE_CHAR, 2) {
        Ok(_) => (),
        Err(e) => return Err(format!("unable to write original rss file: {}", e)),
    };
    // Make racer file
    let racer = FeedRacer::new(
                    &mut channel.items().to_owned(),
                    &params,
                    &dir);
    match racer.write_to_file() {
        Ok(_) => (),
        Err(e) => return Err(format!("failed with error: {}", e))
    };
    // Run update() on this directory. We just created it, so no need to refresh the rss file
    match update_racer_at_path(&dir, &RssFile::FromStorage) {
        Ok(_) => println!("Subscribe to this URL in your pod catcher: {}", racer.get_subscribe_url()),
        Err(e) => return Err(format!("Error writing file: {}", e)),
    };

    Ok(racer)
}


////////////////////////////////////////////////////////////////////////////////////////////////////
 // NAME:   create_feed_racer_dir
 //
 // NOTES:  Creates the direcotry for a racer with these parameters
 // ARGS:
 //     ch - An rss channel (used for the title of the show)
 //     params - Input parameters
 // RETURN: The path of the created directory
 //
fn create_feed_racer_dir(ch: &rss::Channel, params: &RacerCreationParams) -> String {
    let day = chrono::Utc::today();
    // Create this feed's dir name
    let scrubbed_pod_name = &ch.title().to_lowercase()
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
 // NAME:   download_rss_channel
 //
 // NOTES:
 //     Handles the network stuff for getting an rss file from the network.
 //     TODO - Might need to scrub the input here
 // ARGS:   url - the url of the file to get
 // RETURN: A channel or error information
 //
fn download_rss_channel(url: &str) -> Result<rss::Channel, Box<dyn std::error::Error>> {
    let content = match reqwest::blocking::get(url) {
        Ok(val) => val.bytes().unwrap(),
        Err(e) => return Err(Box::new(e)),
    };
    // scrub_bytes(&content);   // Is this needed?
    let channel = rss::Channel::read_from(&content[..])?;
    Ok(channel)
}


pub fn get_by_dir_name(target_dir: &str) -> Option<FeedRacer> {
    // Read all dirs in the podracer dir and compare to this
    let dirs = get_all_podcast_dirs().unwrap();
    for dir_res in dirs {
        let dir = dir_res.unwrap();
        if dir.file_name() == target_dir {
            let racer = get_racer_at_path(dir.path().to_str().unwrap()).unwrap();
            return Some(racer);
        }
    }
    None
}

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


//
// Display implementation
//
impl fmt::Display for RacerEpisode {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "ep_num: {}, date: {}", self.ep_num, self.date)
    }
}
impl fmt::Display for FeedRacer {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        writeln!(f, "schema_version: {}", self.schema_version)?;
        writeln!(f, "racer_path: {}", self.racer_path.display() )?;
        writeln!(f, "source_url: {}", self.source_url)?;
        writeln!(f, "subscribe_url: {}", self.subscribe_url)?;
        writeln!(f, "anchor_date: {}", self.anchor_date)?;
        writeln!(f, "first_pubdate: {}", self.first_pubdate)?;
        writeln!(f, "rate: {}", self.rate)?;
        writeln!(f, "integrate_new: {}", self.integrate_new)?;
        writeln!(f, "release_dates {{")?;
        for entry in self.release_dates.as_slice() {
            writeln!(f, "\t{},", entry)?;
        }
        writeln!(f, "}}")
    }
}
