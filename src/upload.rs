use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::State;
use rocket::tokio::io::AsyncReadExt;
use rocket_dyn_templates::{context, Template};
use crate::config::UploadConfig;
use crate::data_base_manager::login_in;
use crate::mp3_handler::process_mp3;

#[derive(FromForm)]
pub struct UploadForm<'r> {
    pub username: &'r str,
    pub password: &'r str,
    pub artist: &'r str,
    pub title: &'r str,
    pub album: &'r str,
    pub file: TempFile<'r>,
}

#[get("/")]
pub async fn upload_form(_config:&State<UploadConfig>) -> Template {
    Template::render("mp3SubmitForm", context! {})
}

#[inline(always)]
pub fn check_string_for_bad_actors<const CHECK_FOR_SPACE: bool>(data: &str) -> bool {
    // Check for empty string and validate characters based on the compile-time flag
    !data.is_empty() && data.chars().all(|c| {
        if CHECK_FOR_SPACE {
            c.is_alphanumeric() || c == '_' || c == '-' || c == ' '
        } else {
            c.is_alphanumeric() || c == '_' || c == '-'
        }
    })
}

#[inline(always)]
pub fn error_page(msg:& str) -> Template {
    Template::render("SubmitResponse", context! {
        failed: true,
        message: msg
    })
}

#[post("/upload", data = "<form>")]
pub async fn upload_data(config:&State<UploadConfig>,form: Form<UploadForm<'_>>) -> Template {
    let UploadForm {
        username,
        //password is encrypted client side so we need to dencrypt it
        password,
        artist,
        title,
        album,
        file,
    } = form.into_inner();
    let UploadConfig {
        web_assets_dir: _,
        music_dir,
    } = config.inner();
    //this is paranoia but we should never trust the client
    let is_username_ok: bool = check_string_for_bad_actors::<false>(&username);
    let is_password_ok: bool = check_string_for_bad_actors::<false>(&password);
    let is_artist_name_ok: bool = check_string_for_bad_actors::<false>(&artist);
    let is_title_name_ok: bool = check_string_for_bad_actors::<true>(&title);
    let is_album_name_ok: bool = check_string_for_bad_actors::<true>(&album);

    if !is_username_ok {
        return error_page("username has bad charaters in it so is malformed.");
    }

    if !is_password_ok {
        return error_page("password has bad charaters in it so is malformed. if you have added anything not A-Za-z-0-9 this is currently not supported");
    }

    let login = login_in(username,password);

    if !is_artist_name_ok {
        return error_page("Artist name is malformed.");
    }

    if !is_title_name_ok {
        return error_page("Title is malformed.");
    }

    if !is_album_name_ok {
        return error_page("Album name is malformed.");
    }

    let msg = login.await;
    if !msg.is_empty() {
        return error_page(msg.as_str());
    }

    let file_data = match file.open().await {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).await.is_err() {
                return error_page("Failed to read file data from client");
            }
            buffer
        },
        Err(_) => return error_page("Failed to get file from client"),
    };

    process_mp3(file_data,title,artist,album,username,music_dir)
}