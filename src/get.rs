use super::metadata::*;
use lofty::{FileType, Probe};
use std::fs::File;
use std::io::{stdin, stdout, BufReader, Write};
use std::path::PathBuf;
use std::process::exit;
use super::ansi::Ansi;

pub fn get_file(mut file: PathBuf) -> PathBuf
{
    if file.is_dir()
    {
        loop
        {
            file = file.join(select_from_dir(dir_to_filename_list(file.clone())));

            if file.is_file()
            {
                break;
            }
        }
    }
    return file;
}

pub fn file_type(file: PathBuf) -> Option<FileType>
{
    let probe = Probe::new(BufReader::new(
        File::open(file.clone())
            .expect(format!("Couldn't open file {}", file.to_string_lossy()).as_str()),
    ))
    .guess_file_type()
    .unwrap();
    probe.file_type()
}

pub fn get_metadata(file: PathBuf) -> (String, String, String)
{
    let metadata: parse::AudioMetadata = parse::AudioMetadata::from_file(file);
    (metadata.title, metadata.artist, metadata.album)
}

struct Track
{
    track_title: String,
    track_file: PathBuf,
    is_dir: bool,
}

impl Track
{
    fn from_metadata(metadata: parse::AudioMetadata) -> Track
    {
        Track {
            track_file: metadata.path,
            track_title: metadata.title,
            is_dir: false,
        }
    }
}

fn select_from_dir(mut path_list: Vec<PathBuf>) -> PathBuf
{
    path_list.sort_unstable();
    let mut track_list: Vec<Track> = Vec::new();

    let mut i =0;
    for path in path_list.clone()
    {
        if path.is_dir() && i == 0
        {
            track_list.push(Track {
                is_dir: true,
                track_title: "..".to_string(),
                track_file: path.parent().unwrap().parent().unwrap_or(&path).to_path_buf(),
            })
        }
        else if path.is_file() && i == 0
        {
            track_list.push(Track {
                is_dir: true,
                track_title: "..".to_string(),
                track_file: path.parent().unwrap().parent().unwrap_or(&path).to_path_buf(),
            })
        }

        if path.is_file()
            && match path.extension().unwrap().to_str()
            {
                Some("mp4") | Some("flac") | Some("m4a") | Some("ogg") | Some("wav")
                | Some("mp3") => true,
                _ => false,
            }
        {
            track_list.push(Track::from_metadata(parse::AudioMetadata::from_file(path)));
        }
        else if path.is_dir()
        {

            track_list.push(Track {
                is_dir: true,
                track_file: path.clone(),
                track_title: path
                    .file_name()
                    .unwrap_or(PathBuf::from("/").as_os_str())
                    .to_string_lossy()
                    .to_string(),
            })
        }
        i+=1;
    }

    if track_list.len() == 0
    {
        eprintln!("Error: Provided was an empty directory");
        exit(1);
    }

    // Use a while loop becuase foreach (for) loops cause issues,
    // and this is effectively the same thing anyway.

    println!("");

    let mut i = 0;
    while i < track_list.len()
    {
        let track = track_list.get(i).unwrap();
        if track.is_dir
        {
            println!("[{}] {}{}{}", i, Ansi::BLU, track.track_title, Ansi::COLOR_END);
        }
        else
        {
            println!("[{}] {}{}{}", i, Ansi::GRN, track.track_title, Ansi::COLOR_END);
        }
        stdout().flush().expect("Error: Cannot flush stdout");
        i += 1;
    }
    let choice: u32;

    loop
    {
        let mut input = String::new();
        print!("ID: ");
        stdout().flush().expect("Error: Cannot flush stdout");
        stdin()
            .read_line(&mut input)
            .expect("Error: Cannot read from stdin");
        input = input.trim().to_string();
        if input == "exit" || input.trim() == "quit"
        {
            exit(0);
        }
        match input.trim().parse()
        {
            Ok(x) =>
            {
                if x >= track_list.len() as u32
                {
                    eprintln!("Error: Invalid choice: {}", x);
                    continue;
                }
                choice = x;
                break;
            }
            Err(x) =>
            {
                eprintln!("Error: Invalid choice: {}", x);
                continue;
            }
        };
    }
    let chosen_track = track_list.get(choice as usize ).unwrap();
    chosen_track.track_file.clone()
}

fn dir_to_filename_list(path: PathBuf) -> Vec<PathBuf>
{
    let dir = path
        .read_dir()
        .expect(format!("Error: cannot read from dir '{}'", path.to_string_lossy()).as_str());

    dir.map(|entry| {
        let entry_path = entry.unwrap().path();
        let file_name = entry_path.file_name().unwrap();
        let mut file_name_pathbuf = PathBuf::from(file_name);
        file_name_pathbuf = path.join(file_name_pathbuf);
        file_name_pathbuf
    })
    .collect::<Vec<PathBuf>>()
}
