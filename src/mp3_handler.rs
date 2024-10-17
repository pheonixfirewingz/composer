use crate::error;
use id3::{frame, Tag, TagLike, Version};
use rocket::response::Redirect;
use rocket::yansi::Paint;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Cursor, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Command;

#[inline(always)]
fn get_bitrate(file_path: &str) -> Option<u32> {
    // Construct the ffmpeg command to get the bitrate
    let output = Command::new("ffmpeg")
        .arg("-i").arg(file_path).arg("-f")
        .arg("null").arg("-").output().ok()?;
    // Parse the ffmpeg output for the bitrate information
    let stdout = String::from_utf8_lossy(&output.stderr);
    for line in stdout.lines() {
        if let Some(pos) = line.find("kb/s") {
            // Extract the bitrate
            let pass1 = line[..pos].trim();
            let pass2 = pass1.split_at(pass1.rfind(":")? + 2).1;
            let int = pass2.parse::<u32>();
            if !int.is_err(){
                return Some(int.unwrap());
            }
        }
    }
    None
}

#[inline(always)]
fn re_encode_mp3(file_path: &str,temp_file: &str) -> bool {
    let status = Command::new("ffmpeg")
        .arg("-i").arg(file_path).arg("-b:a").arg("128k")
        .arg("-y").arg(temp_file).status();
    if status.is_err() {
        return false
    }
    let mov_states = Command::new("mv")
        .arg(temp_file).arg(file_path).status();
    !mov_states.is_err()
}

#[inline(always)]
fn mp3_file_exists(music_dir: &str, song_title: &str) -> bool {
    let filename = format!("{}.mp3", song_title);
    let file_path = Path::new(music_dir).join(filename);
    file_path.exists()
}

#[inline(always)]

fn find_or_create_artist_directory(music_dir: &str, artist: &str) -> PathBuf {
    let artist_dir = Path::new(music_dir).join(&artist);
    if !artist_dir.is_dir() {
        fs::create_dir_all(&artist_dir).unwrap();
        println!("Created directory: {}", artist_dir.display());
    }
    artist_dir
}

#[inline(always)]
fn update_mp3_metadata_in_place(mp3_data: &mut Vec<u8>, new_artist: &str, new_title: &str,user: &str, new_album: &str) {
    let mut cursor = Cursor::new(mp3_data);
    let mut tag = Tag::read_from2(&mut cursor).unwrap_or_else(|_| Tag::new());
    tag.resetting();
    for i in tag.clone().comments()
    {
        let text = Option::from(i.description.as_str());
        let description = Option::from(i.description.as_str());
        tag.remove_comment(text, description);
    }
    tag.set_artist(new_artist);
    tag.set_title(new_title);
    tag.set_album(new_album);
    tag.add_frame(frame::Comment{
        lang: "eng".to_owned(),
        description: "Register".to_owned(),
        text: format!("{} Registered By {}",new_title,user)
    });
    cursor.set_position(0);
    tag.write_to(&mut cursor, Version::Id3v24).expect("Failed to write updated tag");
}

pub fn process_mp3(mut file_data: Vec<u8>, title:&str, artist:&str, album:&str,user: &str, music_dir:&str) -> Redirect {
    update_mp3_metadata_in_place(&mut file_data,artist,title,user,album);
    let transformed_title = title.replace(' ', "_").to_lowercase();
    let transformed_artist = artist.replace(' ', "_").to_lowercase();
    let artist_dir: PathBuf = find_or_create_artist_directory(music_dir,&transformed_artist);
    if mp3_file_exists(artist_dir.as_path().to_str().unwrap(),&transformed_title) {
        return error::page("This MP3/Song is already Registered with Navidrome");
    }

    let file_path: PathBuf = artist_dir.join(transformed_title.clone() + ".mp3");
    let file = OpenOptions::new().write(true).create_new(true).open(&file_path);
    if file.is_err() {
        eprintln!("{}",file.err().unwrap());
        return error::page("500 - failed to save mp3 to server memory");
    }
    let _ = file.unwrap().write_all(file_data.deref());

    let kbps = get_bitrate(file_path.to_str().unwrap());
    if kbps.is_none() {
        let _ = fs::remove_file(file_path);
        return error::page("failed to convert file for storage could not determine file quality");
    }
    let bit_rate = kbps.unwrap();
    if bit_rate < 128 {
        let _ = fs::remove_file(file_path);
        return error::page("failed to convert file for storage file quality to low needs to be 128kbps or higher");
    }
    let temp_path: PathBuf = artist_dir.join(transformed_title + "_temp.mp3");
    if bit_rate != 128 {
        if !re_encode_mp3(file_path.to_str().unwrap(), temp_path.to_str().unwrap()) {
            let _ = fs::remove_file(file_path);
            return error::page("failed to convert file for storage file");
        }
    }
    Redirect::to("/submit/success")
}