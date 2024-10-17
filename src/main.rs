
#[macro_use]
extern crate rocket;
mod upload;
mod static_asset_manager;
mod data_base_manager;
mod mp3_handler;
mod config;
mod login;
mod error;

use std::env;
use std::path::PathBuf;
use std::process::exit;
use rocket::{Build, Rocket, State};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket_dyn_templates::Template;
use crate::config::{setup_config, Config};
use crate::error::_error;
use crate::login::{logout,login_data, login_form};
use crate::static_asset_manager::get_assets;
use crate::upload::{upload_data, upload_form, upload_success};

#[get("/")]
pub async fn root(_config:&State<Config>,cookies: &CookieJar<'_>) -> Redirect {
    if let Some(_) = cookies.get("user") {
        return Redirect::to("/submit/upload");
    }
    Redirect::to("/submit/login")
}

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
    let config = config.unwrap();

    // Build the Rocket application with increased limits
    let template_dir: PathBuf = PathBuf::from(config.root_dir.clone()).join("web_assets/templates");
    rocket::build()
        .configure(
            rocket::Config::figment()
                .merge(("port", 4545))
                .merge(("limits.form", 13 * 1024 * 1024))
                .merge(("limits.file", 13 * 1024 * 1024))
                .merge(("limits.data-form",13 * 1024 * 1024))
                .merge(("template_dir",template_dir))
        )
        .mount("/submit", routes![root,logout,login_form,login_data ,upload_form, upload_data,upload_success , _error, get_assets])
        .manage(config)
        .attach(Template::fairing())
}

