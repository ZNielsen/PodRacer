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
extern crate racer;
extern crate tera;
mod routes;

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use rocket::fairing::AdHoc;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;
use routes::*;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
// `web` is a symlink to the OUT_DIR location, see build.rs
// const STATIC_FILE_DIR: &'static str = "/etc/podracer/config/server/web/static";
// const STATIC_FILE_DIR: &'static str = "server/static";

////////////////////////////////////////////////////////////////////////////////
//  NAME:   main
//
//  NOTES:
//      Main sets up rocket, spins off an updater thread, then launches
//      the rocket server.
//      Rocket setup includes mounting routes + getting Rocket config values.
//  ARGS:   None
//  RETURN: None
//
fn main() {
    let config = rocket::Config::build(rocket::config::Environment::Production)
        .root(std::path::Path::new("/etc/podracer/config"))
        .address("0.0.0.0")
        .port(42069)
        .keep_alive(5)
        .log_level(rocket::config::LoggingLevel::Normal)
        .extra("update_factor", 45)
        .extra("host", "http://podracer.zachn.me")
        .extra("static_file_dir", "/etc/podracer/config/server/web/static")
        .extra("podracer_dir", "/etc/podracer/podcasts")
        .finalize().expect("Config is valid");

    let rocket = rocket::custom(config.clone())
        .register(catchers![not_found_handler])
        .mount("/", routes![create_feed_form_handler])
        .mount("/", routes![update_one_handler])
        .mount("/", routes![update_all_handler])
        //.mount("/", routes![delete_feed_handler])
        .mount("/", routes![list_feeds_handler])
        .mount("/", routes![serve_rss_handler])
        .mount("/", routes![create_feed_handler])
        .mount("/", routes![create_feed_cli_handler])
        .mount("/", routes![create_feed_cli_ep_handler])
        // .mount("/", routes![manual::icon])
        .mount("/", StaticFiles::from(&config.get_str("podracer_dir").unwrap()))
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Asset Config", |rocket| {
            // Parse out config values we need to tell users about
            let rocket_config = routes::RocketConfig {
                static_file_dir: rocket.config().get_str("static_file_dir").unwrap().to_owned(),
                podracer_dir: rocket.config().get_str("podracer_dir").unwrap().to_owned(),
                address: rocket.config().get_str("host").unwrap().to_owned(),
                port: rocket.config().port as u64,
            };
            let update_factor = rocket.config().get_int("update_factor").unwrap() as u64;

            // Add custom configs to the State manager - only one of each type is allowed
            Ok(rocket
                .manage(rocket_config)
                .manage(routes::UpdateFactor(update_factor)))
        }));

    // Manually update on start
    match racer::update_all(&config.get_str("podracer_dir").unwrap()) {
        Ok(update_metadata) => println!(
            "Manually updated on boot. Did {} feeds in {:?} ({} feeds with new episodes).",
            update_metadata.num_updated, update_metadata.time, update_metadata.num_with_new_eps
        ),
        Err(string) => println!("Error in update_all on boot: {}", string),
    };

    let duration: u64 = match rocket.state::<UpdateFactor>() {
        Some(val) => (val.0 * 60),
        None => (59 * 60),
    };

    println!("Spawning update thread. Will run every {} seconds.", duration);

    // Create update thread - update every <duration> (default to every hour if not specified in Rocket.toml)
    let _update_thread = std::thread::Builder::new()
        .name("Updater".to_owned())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(duration));
            print!("Updating all feeds... ");
            match racer::update_all(&config.get_str("podcast_dir").unwrap()) {
                Ok(update_metadata) => {
                    println!(
                        "Done. Did {} feeds in {:?} ({} feeds with new episodes).",
                        update_metadata.num_updated,
                        update_metadata.time,
                        update_metadata.num_with_new_eps
                    );
                }
                Err(string) => {
                    println!("Error in update_all in update thread: {}", string);
                }
            };
        });
    rocket.launch();
}


