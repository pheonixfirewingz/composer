
#[macro_use]
extern crate rocket;
mod upload;
mod static_asset_manager;
mod data_base_manager;
mod mp3_handler;
mod config;

use std::env;
use std::process::exit;
use rocket::{Build, Rocket};
use rocket_dyn_templates::Template;
use crate::config::{setup_config, Config, UploadConfig};
//main is only used to set up routing for the webserver don't add logic here
#[launch]
fn rocket() -> Rocket<Build> {
    let config: Option<Config> = setup_config();
    if config.is_none() {
        let args: Vec<String> = env::args().collect();
        eprint!(
            "We need a valid config. The config given is not valid or wrong path: {}",
            args.get(1).unwrap()
        );
        exit(1);
    }

    let config_data = config.unwrap();

    // Build the Rocket application with increased limits
    rocket::build()
        .configure(
            rocket::Config::figment()
                .merge(("port", 4545))
                .merge(("limits.form", 13 * 1024 * 1024))
                .merge(("limits.file", 13 * 1024 * 1024))
                .merge(("limits.data-form",13 * 1024 * 1024))
                .merge(("template_dir",config_data.templates_dir.clone()))
        )
        .mount(config_data.site_root.clone(), routes![upload::upload_form, upload::upload_data, static_asset_manager::get_assets])
        .manage(UploadConfig::new(config_data.web_assets_dir,config_data.music_dir))
        .attach(Template::fairing())
}

