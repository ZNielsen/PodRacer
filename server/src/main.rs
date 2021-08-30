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
use rocket::fs::FileServer;
use rocket_dyn_templates::Template;
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
#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    let rocket = rocket::build();
    let figment = rocket::Config::figment();
    // TODO - don't extract everything peicewise, just have a podracer config struct
    let static_file_dir: String = figment.extract_inner::<String>("static_file_dir")
            .expect("static_file_dir in config");
    let podracer_dir = figment.extract_inner::<String>("podracer_dir")
            .expect("podracer_dir in config");
    let podracer_dir_for_closure = podracer_dir.clone();
    let rocket = rocket
        .register("/", catchers![not_found_handler])
        .mount("/", routes![create_feed_form_handler])
        .mount("/", routes![edit_feed_handler])
        .mount("/", routes![pause_feed_handler])
        .mount("/", routes![unpause_feed_handler])
        .mount("/", routes![update_one_handler])
        .mount("/", routes![update_all_handler])
        .mount("/", routes![list_feeds_handler])
        .mount("/", routes![serve_rss_handler])
        .mount("/", routes![create_feed_handler])
        .mount("/", routes![create_feed_cli_handler])
        .mount("/", routes![create_feed_cli_ep_handler])
        .mount("/", FileServer::from(&static_file_dir))
        .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Asset Config", |rocket| async move {
            // Parse out config values we need to tell users about
            let rocket_config = routes::RocketConfig {
                // static_file_dir: rocket.config().get_str("static_file_dir").expect("static_file_dir in config").to_owned(),
                // podracer_dir: rocket.config().get_str("podracer_dir").expect("podracer_dir in config").to_owned(),
                static_file_dir: static_file_dir,
                podracer_dir: podracer_dir_for_closure,
                address: figment.extract_inner::<String>("host").unwrap(),
                port: figment.extract_inner::<u64>("port").unwrap(),
            };
            let update_factor = figment.extract_inner::<u64>("update_factor").unwrap();

            // Add custom configs to the State manager - only one of each type is allowed
            rocket
                .manage(rocket_config)
                .manage(routes::UpdateFactor(update_factor))
        }));

    // Manually update on start
    match racer::update_all(&podracer_dir) {
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
            match racer::update_all(&podracer_dir) {
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
    rocket
}

