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
use std::io::{BufReader, Write};
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
pub const SCHEMA_VERSION:    &'static str = "1.0.0";
pub const PODRACER_DIR:      &'static str = ".podracer";

pub const ORIGINAL_RSS_FILE: &'static str = "original.rss";
pub const RACER_RSS_FILE:    &'static str = "racer.rss";
pub const RACER_FILE:        &'static str = "racer.file";
pub const INDENT_AMOUNT:            usize = 2;
pub const SPACE_CHAR:                  u8 = 32;

pub struct RacerCreationParams {
    pub address: String,
    pub port: u64,
    pub url: String,
    pub rate: f32,
    pub integrate_new: bool,
    pub start_ep: usize,
}

pub enum RssFile {
    Download,
    FromStorage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RacerEpisode {
    ep_num: i64,
    date: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedRacer {
    schema_version: String,
    racer_path: PathBuf,
    source_url: String,
    podracer_url: String,
    anchor_date: DateTime<chrono::Utc>,
    first_pubdate: DateTime<chrono::FixedOffset>,
    rate: f32,
    integrate_new: bool,
    release_dates: Vec<RacerEpisode>
}
// Basic getter functions
impl FeedRacer {
    pub fn get_schema_version(&self) -> &str { &self.schema_version }
    pub fn get_racer_path(&self) -> &Path { &self.racer_path }
    pub fn get_source_url(&self) -> &str { &self.source_url }
    pub fn get_podracer_url(&self) -> &str { &self.podracer_url }
    pub fn get_anchor_date(&self) -> DateTime<chrono::Utc> { self.anchor_date }
    pub fn get_first_pubdate(&self) -> DateTime<chrono::FixedOffset> { self.first_pubdate }
    pub fn get_rate(&self) -> f32 { self.rate }
    pub fn get_integrate_new(&self) -> bool { self.integrate_new }
    pub fn get_release_dates(&self) -> &Vec<RacerEpisode> { &self.release_dates }
}



impl FeedRacer {
    ////////////////////////////////////////////////////////////////////////////////
     // NAME:   [name]
     //
     // NOTES:
     // ARGS:
     // RETURN:
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
        let podracer_url: PathBuf = [
                            (String::from(&params.address) +":"+ &params.address).as_str(),
                            "podcasts",
                            podcast_dir_name,
                            RACER_RSS_FILE].iter().collect();

        let racer_data = FeedRacer {
            schema_version: SCHEMA_VERSION.to_owned(),
            racer_path: PathBuf::from(dir),
            source_url: params.url.to_owned(),
            podracer_url: podracer_url.to_str().unwrap().to_owned(),
            rate: params.rate.to_owned(),
            anchor_date: anchor_date,
            first_pubdate: first_pubdate,
            integrate_new: params.integrate_new.to_owned(),
            release_dates: dates,
        };

        racer_data
    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;

        let filename: PathBuf = [self.get_racer_path().to_str().unwrap(), RACER_FILE].iter().collect();
        let mut fp = File::create(filename)?;
        fp.write_all(json.as_bytes())
    }

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

    // TODO -> handle the case where the url is invalid/disappears
    // TODO -> handle feeds that have a constant number of entries
    //         and push the oldest entry out
    pub fn get_original_rss(&mut self, mode: &RssFile) -> rss::Channel {
        let mut stored_rss_path = self.racer_path.clone();
        stored_rss_path.push(ORIGINAL_RSS_FILE);
        let stored_rss_file = File::open(&stored_rss_path).unwrap();
        let buf_reader = BufReader::new(stored_rss_file);
        let stored_rss = rss::Channel::read_from(buf_reader).unwrap();
        let original_rss = match mode {
            RssFile::Download => {
                let network_file = download_rss_channel(&self.source_url).unwrap();
                // Compare to stored file - update if we need to
                let num_to_update = (network_file.items().len() as i64 - stored_rss.items().len() as i64).abs();
                if num_to_update > 0 {
                    // Overwrite our stored original RSS file
                    let stored_rss_file = File::create(stored_rss_path).unwrap();
                    network_file.pretty_write_to(stored_rss_file, SPACE_CHAR, INDENT_AMOUNT).unwrap();
                    // Append new entries to our racer object
                    let mut new_items = network_file.items().to_owned();
                    new_items.truncate(num_to_update as usize);
                    self.add_new_items(&new_items, stored_rss.items().len());
                }
                network_file
            },
            RssFile::FromStorage => stored_rss,
        };

        original_rss
    }

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
}

pub fn update_racer_at_path(path: &str, mode: &RssFile) -> std::io::Result<()> {
    // Load in racer file
    let racer_file_path: PathBuf = [path, RACER_FILE].iter().collect();
    let racer_file = File::open(racer_file_path)?;
    let mut racer: FeedRacer = serde_json::from_reader(&racer_file)?;

    // Get original rss feed
    let mut rss = racer.get_original_rss(mode);

    // Tack on a `- PodRacer` to the title
    rss.set_title(String::from(rss.title()) + " - PodRacer");

    // Check how many episodes we should publish at this point
    let num_to_pub = racer.get_num_to_publish();
    let num_to_scrub = rss.items().len() - num_to_pub;
    // Drain the items we aren't publishing yet - TODO: Can we do this in place with slices?
    let mut items_to_publish = rss.items().to_owned();
    items_to_publish.drain(0..num_to_scrub);

    // Now that we have the items we want, overwrite the objects items.
    rss.set_items(items_to_publish);
    // Write out the racer.rss file
    let racer_rss_path: PathBuf = [racer.get_racer_path().to_str().unwrap(), RACER_RSS_FILE].iter().collect();
    let racer_rss_file = File::create(racer_rss_path)?;
    rss.pretty_write_to(racer_rss_file, SPACE_CHAR, INDENT_AMOUNT).unwrap();
    Ok(())
}

pub fn update_all() -> Result<(), String> {
    let mut dir = match dirs::home_dir() {
        Some(val) => val,
        None => return Err(format!("Error retrieving home dir")),
    };
    dir.push(PODRACER_DIR);
    let podcast_dirs = match Path::read_dir(dir.as_path()) {
        Ok(val) => val,
        Err(e) => return Err(format!("Cannot access dir for updating: {:?}.\nError: {}", dir, e)),
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

pub fn create_feed(params: &RacerCreationParams) -> Result<FeedRacer, String> {
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
        Ok(_) => println!("Subscribe to this URL in your pod catcher: {}", racer.get_podracer_url()),
        Err(e) => return Err(format!("Error writing file: {}", e)),
    };

    Ok(racer)
}


fn create_feed_racer_dir(ch: &rss::Channel, params: &RacerCreationParams) -> String {
    let day = chrono::Utc::today();
    // Create this feed's dir name
    let mut dir = String::from(dirs::home_dir().unwrap().to_str().unwrap());
    dir.push_str("/");
    dir.push_str(PODRACER_DIR);
    dir.push_str("/");
    dir.push_str(&ch.title().to_lowercase().replace(" ", "-"));
    dir.push_str("_");
    dir.push_str(&params.rate.to_string());
    dir.push_str("_");
    dir.push_str(&day.format("%Y-%m-%d").to_string());
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn download_rss_channel(url: &str) -> Result<rss::Channel, Box<dyn std::error::Error>> {
    let content = reqwest::blocking::get(url).unwrap().bytes().unwrap();
    let channel = rss::Channel::read_from(&content[..])?;
    Ok(channel)
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
        writeln!(f, "podracer_url: {}", self.podracer_url)?;
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
