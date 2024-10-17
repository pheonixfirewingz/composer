use crate::error;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::State;
use rocket::tokio::io::AsyncReadExt;
use rocket_dyn_templates::{context, Template};
use crate::config::Config;
use crate::mp3_handler::process_mp3;

#[derive(FromForm)]
pub struct UploadForm<'r> {
    pub artist: &'r str,
    pub title: &'r str,
    pub album: &'r str,
    pub file: TempFile<'r>,
}


#[get("/upload")]
pub async fn upload_form(_config:&State<Config>) -> Template {
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

#[get("/success")]
pub fn upload_success() -> Template {
    Template::render("SubmitResponse", context! {
        failed: false,
        message: ""
    })
}
#[post("/upload/send", data = "<form>")]
pub async fn upload_data(config: &State<Config>, form: Form<UploadForm<'_>>, cookies: &CookieJar<'_>) -> Redirect {
    if let Some(cookie) = cookies.get("user") {
        let username = cookie.value();
        let UploadForm {
            artist,
            title,
            album,
            file,
        } = form.into_inner();
        let Config {
            root_dir: _,
            music_dir,
        } = config.inner();
        //this is paranoia but we should never trust the client
        let is_artist_name_ok: bool = check_string_for_bad_actors::<true>(&artist);
        let is_title_name_ok: bool = check_string_for_bad_actors::<true>(&title);
        let is_album_name_ok: bool = check_string_for_bad_actors::<true>(&album);
        
        if !is_artist_name_ok {
            return error::page("Artist name is malformed.");
        }

        if !is_title_name_ok {
            return error::page("Title is malformed.");
        }

        if !is_album_name_ok {
            return error::page("Album name is malformed.");
        }
        
        let file_data = match file.open().await {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).await.is_err() {
                    return error::page("Failed to read file data from client");
                }
                buffer
            },
            Err(_) => return error::page("Failed to get file from client"),
        };

        process_mp3(file_data,title,artist,album,username,music_dir)
    } else {
        Redirect::to("/submit/login")
    }
}