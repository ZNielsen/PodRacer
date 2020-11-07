use chrono::{DateTime, Duration};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::fs::File;

pub const SCHEMA_VERSION:    &'static str = "1.0.0";
pub const PODRACER_DIR:      &'static str = "~/.podracer";

pub const ORIGINAL_RSS_FILE: &'static str = "original.rss";
pub const RACER_FILE:        &'static str = "racer.file";
pub const SPACE_CHAR:                  u8 = 32;



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
    pub fn new(items: &mut Vec<rss::Item>, rate: &f32, source_url: &str, integrate_new: &bool) -> FeedRacer {
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
            racer_path: PODRACER_DIR.to_owned(),
            source_url: source_url.to_owned(),
            podracer_url: "xxx".to_owned(),
            rate: rate.to_owned(),
            anchor_date: anchor_date,
            first_pubdate: first_pubdate,
            integrate_new: integrate_new.to_owned(),
            release_dates: dates,
        };

        racer_data
    }

    pub fn update_published_items(&self, items: &mut std::vec::Vec<rss::Item>) {

    }

    pub fn write_to_file(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self)?;

        let filename = String::from(PODRACER_DIR) +"/"+ RACER_FILE;
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
    pub fn get_updated_original_rss(&mut self) -> rss::Channel {
        // Re-download file
        let original_rss = crate::utils::download_rss_channel(&self.source_url).unwrap();
        // Compare
        let stored_rss_file = File::open(String::from(&self.racer_path) +"/"+ ORIGINAL_RSS_FILE).unwrap();
        let buf_reader = BufReader::new(stored_rss_file);
        let stored_rss = rss::Channel::read_from(buf_reader).unwrap();
        let num_to_update = (original_rss.items().len() as i64 - stored_rss.items().len() as i64).abs();
        if num_to_update > 0 {
            // Overwrite our stored original RSS file
            let original_rss_file = File::create(String::from(&self.racer_path) +"/"+ ORIGINAL_RSS_FILE).unwrap();
            original_rss.pretty_write_to(original_rss_file, SPACE_CHAR, 2).unwrap();
            // Append new entries to our racer object
            let mut new_items = original_rss.items().to_owned();
            new_items.truncate(num_to_update as usize);
            self.add_new_items(&new_items, stored_rss.items().len());
        }
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