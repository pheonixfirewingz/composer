#[macro_use]
extern crate rocket;

use rocket::{
    form::Form,
    fs::{NamedFile, TempFile},
    response::status::NotFound,
};
use rocket_dyn_templates::{context, Template};
use std::path::PathBuf;

#[derive(Debug, FromForm)]
pub struct UploadForm<'r> {
    pub email: String,
    pub password: String,
    pub artist: String,
    pub title: String,
    pub album: String,
    pub file: TempFile<'r>,
}

#[get("/")]
pub fn upload_form() -> Template {
    Template::render("mp3SubmitForm", context! {})
}

// we add a layer of security be rejecting any request that is not needed to make the website
#[get("/assets/<path..>")]
async fn get_assets(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    // Construct the full path to the file in the web_assets directory
    let temp_copy = path.clone();
    let file_path = PathBuf::from("./web_assets/").join(path);

    // Print out the path for debugging
    println!("Requested file path: {:?}", file_path);

    // Get the file extension and check if it's allowed
    let ext = file_path.extension().and_then(|ext| ext.to_str());
    match ext {
        Some("js") | Some("css") | Some("webp") | Some("html") => {
            // Attempt to open the file; if it fails, return a NotFound error
            let copy= file_path.clone();
            match NamedFile::open(file_path).await {
                Ok(named_file) => {
                    println!("File found: {:?}", copy);
                    Ok(named_file)
                },
                Err(_) => {
                    println!("File not found: {:?}", copy);
                    Err(NotFound(format!("File not found: {:?}", temp_copy)))
                }
            }
        },
        _ => {
            println!("File type not allowed: {:?}", &temp_copy);
            Err(NotFound(format!("File type not allowed: {:?}", temp_copy)))
        }
    }
}


#[post("/", data = "<form>")]
async fn upload<'r>(form: Form<UploadForm<'r>>) -> String {
    let UploadForm {
        email,
        password,
        artist,
        title,
        album,
        file,
    } = form.into_inner();

    // Handle the file: _ and other data here
    format!(
        "Uploaded file: _ '{}' by '{}' from album '{}'",
        title, artist, album
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8081)))
        .mount("/", routes![upload_form, upload, get_assets])
        .attach(Template::fairing())
}
