use std::path::PathBuf;
use rocket::fs::NamedFile;
use rocket::response::status::NotFound;
use rocket::State;
use crate::config::UploadConfig;

// we add a layer of security be rejecting any request that is not needed to make the website
#[get("/assets/<path..>")]
pub async fn get_assets(config:&State<UploadConfig>,path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    // Construct the full path to the file in the web_assets directory
    let file_path = PathBuf::from(config.web_assets_dir.clone()).join(&path);

    // Print out the path for debugging
    if cfg!(debug_assertions) {
        println!("Requested file path: {:?}", file_path);
    }
    // Get the file extension and check if it's allowed
    let ext = file_path.extension().and_then(|ext| ext.to_str());
    match ext {
        Some("css") | Some("webp") | Some("html") => {
            // Attempt to open the file; if it fails, return a NotFound error
            let copy= file_path.clone();
            match NamedFile::open(file_path).await {
                Ok(named_file) => {
                    if cfg!(debug_assertions) {
                        println!("File found: {:?}", copy);
                    }
                    Ok(named_file)
                },
                Err(_) => {
                    Err(NotFound(format!("File not found: {:?}", &path)))
                }
            }
        },
        _ => {
            Err(NotFound(format!("File type not allowed: {:?}", &path)))
        }
    }
}