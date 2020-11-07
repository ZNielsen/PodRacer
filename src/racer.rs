use chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION:    &'static str = "1.0.0";
pub const PODRACER_DIR:      &'static str = ".podracer";

pub const ORIGINAL_RSS_FILE: &'static str = "original.rss";
pub const RACER_RSS_FILE:    &'static str = "racer.rss";
pub const RACER_FILE:        &'static str = "racer.file";
pub const INDENT_AMOUNT:            usize = 2;
pub const SPACE_CHAR:                  u8 = 32;


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
    racer_path: String,
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
    pub fn get_racer_path(&self) -> &str { &self.racer_path }
    pub fn get_source_url(&self) -> &str { &self.source_url }
    pub fn get_podracer_url(&self) -> &str { &self.podracer_url }
    pub fn get_anchor_date(&self) -> DateTime<chrono::Utc> { self.anchor_date }
    pub fn get_first_pubdate(&self) -> DateTime<chrono::FixedOffset> { self.first_pubdate }
    pub fn get_rate(&self) -> f32 { self.rate }
    pub fn get_integrate_new(&self) -> bool { self.integrate_new }
    pub fn get_release_dates(&self) -> &Vec<RacerEpisode> { &self.release_dates }
}



impl FeedRacer {
    pub fn new(items: &mut Vec<rss::Item>,
        rate: &f32,
        source_url: &str,
        integrate_new: &bool,
        dir: &str) -> FeedRacer {
        // Reverse the items so the oldest entry is first
        items.reverse();
        // Get anchor date
        let first_pubdate = items.first().unwrap().pub_date().unwrap();
        let first_pubdate = DateTime::parse_from_rfc2822(first_pubdate).unwrap();
        let anchor_date = chrono::Utc::now();
        let mut dates = Vec::new();
        let mut item_counter = 0;
        for item in items {
            // Get diff from first published date
            let pub_date = item.pub_date().unwrap();
            let mut time_diff = DateTime::parse_from_rfc2822(pub_date).unwrap()
                                .signed_duration_since(first_pubdate)
                                .num_seconds();
            // Scale that diff
            time_diff = ((time_diff as f32) / rate) as i64;
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

        let racer_data = FeedRacer {
            schema_version: SCHEMA_VERSION.to_owned(),
            racer_path: dir.to_owned(),
            source_url: source_url.to_owned(),
            podracer_url: "http://example.com".to_owned(),
            rate: rate.to_owned(),
            anchor_date: anchor_date,
            first_pubdate: first_pubdate,
            integrate_new: integrate_new.to_owned(),
            release_dates: dates,
        };

        racer_data
    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;

        let filename = String::from(self.get_racer_path()) +"/"+ RACER_FILE;
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
        let stored_rss_file = File::open(String::from(&self.racer_path) +"/"+ ORIGINAL_RSS_FILE).unwrap();
        let buf_reader = BufReader::new(stored_rss_file);
        let stored_rss = rss::Channel::read_from(buf_reader).unwrap();
        let original_rss = match mode {
            RssFile::Download => {
                let network_file = crate::utils::download_rss_channel(&self.source_url).unwrap();
                // Compare to stored file - update if we need to
                let num_to_update = (network_file.items().len() as i64 - stored_rss.items().len() as i64).abs();
                if num_to_update > 0 {
                    // Overwrite our stored original RSS file
                    let stored_rss_file = File::create(String::from(&self.racer_path) +"/"+ ORIGINAL_RSS_FILE).unwrap();
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
    let racer_file = File::open(String::from(path) +"/"+ crate::racer::RACER_FILE)?;
    let mut racer: FeedRacer = serde_json::from_reader(&racer_file)?;

    // Get original rss feed
    let mut rss = racer.get_original_rss(mode);

    // Check how many episodes we should publish at this point
    let num_to_pub = racer.get_num_to_publish();
    // Pull out only the items we want published
    let mut items_to_publish = rss.items().to_owned();
    items_to_publish.reverse();
    items_to_publish.truncate(num_to_pub);
    // Set the items to only contain what we want pubished
    rss.set_items(items_to_publish);
    // Write out the racer.rss file
    let racer_rss_file = File::create(racer.get_racer_path().to_owned() +"/"+ RACER_RSS_FILE)?;
    rss.pretty_write_to(racer_rss_file, SPACE_CHAR, INDENT_AMOUNT).unwrap();
    Ok(())
}

pub fn update_all() {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(PODRACER_DIR);
    for podcast_dir in Path::read_dir(dir.as_path()).unwrap() {
        let path = podcast_dir.unwrap().path();
        match update_racer_at_path(path.to_str().unwrap(), &RssFile::Download) {
            Ok(()) => (),
            Err(e) => println!("Could not update path {}. Error was: {}",
                        path.to_str().unwrap(), e),
        };
    }
}


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
        writeln!(f, "racer_path: {}", self.racer_path )?;
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
