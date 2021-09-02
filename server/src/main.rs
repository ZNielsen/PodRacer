////////////////////////////////////////////////////////////////////////////////
//  File:   main.rs
//
//  © Zach Nielsen 2020
//  Main server code
//
#![feature(async_closure)]
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
use rocket::fs::FileServer;

use rocket_dyn_templates::Template;

use routes::*;

use serde::Deserialize;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
// `web` is a symlink to the OUT_DIR location, see build.rs
// const STATIC_FILE_DIR: &'static str = "/etc/podracer/config/server/web/static";
// const STATIC_FILE_DIR: &'static str = "server/static";

#[derive(Clone, Deserialize)]
struct PodRacerRocketConfig {
    static_file_dir: String,
    update_factor: u32,
    podracer_dir: String,
    address: String,
    port: u32,
}

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
#[launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
    let rocket = rocket::build();
    let custom_config: PodRacerRocketConfig = rocket::Config::figment().extract()
        .expect("Can extract custom config from rocket");
    let config_for_closure = custom_config.clone();

    let rocket = rocket
        .register("/", catchers![not_found_handler])
        .mount("/", routes![create_feed_form_handler])
        .mount("/", routes![edit_feed_get_handler])
        .mount("/", routes![edit_feed_post_handler])
        .mount("/", routes![update_one_handler])
        .mount("/", routes![update_all_handler])
        .mount("/", routes![list_feeds_handler])
        .mount("/", routes![serve_rss_handler])
        .mount("/", routes![create_feed_handler])
        .mount("/", routes![create_feed_cli_handler])
        .mount("/", routes![create_feed_cli_ep_handler])
        .mount("/", FileServer::from(&custom_config.static_file_dir))
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Asset Config", |rocket| async move {
            // Parse out config values we need to tell users about
            let rocket_config = routes::RocketConfig {
                static_file_dir: config_for_closure.static_file_dir,
                podracer_dir: config_for_closure.podracer_dir,
                address: config_for_closure.address,
                port: config_for_closure.port,
            };

            // Add custom configs to the State manager - only one of each type is allowed
            rocket
                .manage(rocket_config)
                .manage(routes::UpdateFactor(config_for_closure.update_factor))
        }));

    // Manually update on start
    match racer::update_all(&custom_config.podracer_dir).await {
        Ok(update_metadata) => println!(
            "Manually updated on boot. Did {} feeds in {:?} ({} feeds with new episodes).",
            update_metadata.num_updated, update_metadata.time, update_metadata.num_with_new_eps
        ),
        Err(string) => println!("Error in update_all on boot: {}", string),
    };

    let duration: u32 = match rocket.state::<UpdateFactor>() {
        Some(val) => (val.0 * 60),
        None => (59 * 60),
    };

    println!("Spawning update thread. Will run every {} seconds.", duration);

    // Create update thread - update every <duration> (default to every hour if not specified in Rocket.toml)
    let looping_update_fn = async move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(duration as u64));
            print!("Updating all feeds... ");
            match racer::update_all(&custom_config.podracer_dir).await {
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
        };
    };
    let _update_thread = std::thread::Builder::new()
        .name("Updater".to_owned())
        .spawn(looping_update_fn);
    rocket
}

