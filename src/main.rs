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
#[macro_use]
extern crate tera;
mod racer;
mod routes;

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use rocket::fairing::AdHoc;
use routes::*;
use tera::Tera;
use std::path::PathBuf;
use std::io::{BufRead, Write};
use std::fs::File;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////
 // NAME:   scrub_xml
 //
 // NOTES:
 //     Some rss feeds don't properly escape things. Properly escape known issues.
 //     This is not really scalable, but if I'm the only one using it then it should be more or
 //     less fine.
 // ARGS:   file_name - The file to scrub and replace
 // RETURN: None
 //
fn scrub_xml(file_name: &PathBuf) {
    // Known bad strings
    let mut subs = std::collections::HashMap::new();
    subs.insert("& ".to_owned(), "&amp; ".to_owned());

    //
    // Go over everything and substitute known issues
    //
    let tmp_file_name = "/tmp/scrubbed.rss".to_owned();
    let file = File::open(file_name).expect("Could not open original file");
    let in_buf = std::io::BufReader::new(&file);
    let scrubbed_file = File::create(&tmp_file_name).expect("Failed to create tmp scrub file");
    let mut out_buf = std::io::BufWriter::new(scrubbed_file);
    in_buf.lines().map(|line_res| {
        line_res.and_then(|mut line| {
            for (key,val) in &subs {
                if line.contains(key) {
                    line = line.replace(key, val);
                }
            }
            out_buf.write_all(line.as_bytes())
        })
    }).collect::<Result<(), _>>().expect("IO failed");

    // Replace original with scrubbed file
    std::fs::rename(std::path::Path::new(&tmp_file_name),
                    std::path::Path::new(&file_name))
         .expect("Failed to overwrite file");
}

////////////////////////////////////////////////////////////////////////////////
 // NAME:   main
 //
 // NOTES:
 //     Main sets up rocket, spins off an updater thread, then launches
 //     the rocket server.
 //     Rocket setup includes mounting routes + getting Rocket config values.
 // ARGS:   None
 // RETURN: None
 //
fn main() {
    // Use globbing.
    // TODO - look into lazy_static! here.
    // let tera = match Tera::new("templates/**/*.html.tera") {
    //     Ok(t) => t,
    //     Err(e) => panic!("Tera parsing error(s): {}", e),
    // };

    let rocket = rocket::ignite()
        .mount("/", routes![create_feed_form_handler])
        .mount("/", routes![update_one_handler])
        .mount("/", routes![update_all_handler])
        //.mount("/", routes![delete_feed_handler])
        .mount("/", routes![list_feeds_handler])
        .mount("/", routes![serve_rss_handler])
        .mount("/", routes![create_feed_handler])
        .mount("/", routes![create_feed_handler_ep])
        // .mount("/", routes![manual::icon])
        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Asset Config", |rocket| {
            // Parse out custom config values
            let rocket_config = routes::RocketConfig {
                address: rocket.config().get_str("host").unwrap().to_owned(),
                port: rocket.config().port as u64,
            };
            let update_factor = rocket.config().get_int("update_factor").unwrap() as u64;

            // Add custom configs to the State manager - only one of each type is allowed
            Ok(rocket
                .manage(rocket_config)
                .manage(routes::UpdateFactor(update_factor))
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
