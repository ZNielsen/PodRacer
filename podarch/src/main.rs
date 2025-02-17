////////////////////////////////////////////////////////////////////////////////
//  File:   main.rs
//
//  © Zach Nielsen 2021
//  Feed Archiver
//

////////////////////////////////////////////////////////////////////////////////
//  Included Modules
////////////////////////////////////////////////////////////////////////////////
extern crate racer;

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use structopt::StructOpt;
use futures::{stream, StreamExt};
use tokio;

use std::path::PathBuf;
use std::io::Write;
use std::fs;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
#[tokio::main]
async fn main() {
    // Parse command line args
    let opt = Opt::from_args();
    let dir = PathBuf::from(&opt.dir);
    // Create target dir
    fs::create_dir(&dir).unwrap_or(());
    // Get the rss file
    let client = reqwest::Client::new();
    let rss = match racer::download_rss_channel(&client, &opt.url).await {
        Ok(feed) => feed,
        Err(e) => panic!("Error getting rss feed (from {}): {}", &opt.url, e),
    };

    // Write RSS file to disk
    let rss_file_name = std::path::Path::new(&opt.url).file_name().unwrap();
    let rss_file_path: PathBuf = [&opt.dir, rss_file_name.to_str().unwrap()].iter().collect();
    let rss_file = fs::File::create(rss_file_path).unwrap();
    rss.pretty_write_to(rss_file, b' ', 2).unwrap();

    let mut mime_type_map = std::collections::HashMap::new();
    mime_type_map.insert(String::from("audio/mpeg"), ".mp3");
    let mime_type_map = mime_type_map;

    // Download episode to file
    let parallel_gets = 5;
    let numbered_items: Vec<_> = rss.items().iter().rev().enumerate().collect();
    let results = stream::iter(numbered_items)
        .map(|(index, item)| {
            // println!("item: {:#?}", item);
            let client = &client;
            let opt = &opt;
            let mime_type_map = &mime_type_map;
            async move {
                // Create destination
                let mut filename = format!("{:03} - {}",
                    index + 1,
                    String::from(item.title().unwrap_or("FixMeNoTitle")));
                filename.push_str(mime_type_map.get(&item.enclosure().unwrap().mime_type).unwrap());
                filename = filename.replace("/", "-");

                let filepath: PathBuf = [&opt.dir, &filename]
                    .iter()
                    .collect();
                if filepath.as_path().exists() {
                    println!("File already exists, skipping: [{:?}]", filepath);
                    return
                }
                let mut fp = fs::File::create(&filepath).expect(&format!("Creating {:#?}", &filepath));

                // Download episode to file
                let link = &item.enclosure().unwrap().url;
                println!("Getting from link: {}", link);
                let mut stream = client.get(link).send().await.unwrap()
                    .bytes_stream();
                while let Some(stuff) = stream.next().await {
                    // Stream to file
                    fp.write_all(&stuff.unwrap()).unwrap();
                }

                if opt.get_description {
                    let mut desc_filename = format!("{:03} - {}_description.html",
                        index + 1,
                        String::from(item.title().unwrap_or("FixMeNoTitle")));
                    desc_filename = desc_filename.replace("/", "-");

                    let desc_filepath: PathBuf = [&opt.dir, &desc_filename]
                        .iter()
                        .collect();
                    let mut fp = fs::File::create(&desc_filepath).expect(&format!("Creating {:#?}", &desc_filepath));
                    fp.write_all(item.description().unwrap().as_bytes()).unwrap();
                }
            }
        })
        .buffer_unordered(parallel_gets);

    let items = results.collect::<Vec<_>>().await;

    println!("Done. Downloaded {} episodes to {}", items.len(), &opt.dir);
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "podarch",
    about = "A podcast feed archiver"
)]
struct Opt {
    /// The URL of the RSS feed to archive
    url: String,

    /// The directory to save the podcast to
    dir: String,

    /// Also download the description for each episode into a separate file
    #[structopt(long)]
    get_description: bool
}

