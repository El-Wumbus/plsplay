use std::{
    fs::{File, ReadDir},
    io::BufReader,
    path::PathBuf,
};
use structopt::StructOpt;
mod select;

// fn select_from_dir()

fn dir_to_filename_list(dir: ReadDir) -> Vec<PathBuf>
{
    dir.map(|entry| {
        let entry_path = entry.unwrap().path();
        let file_name = entry_path.file_name().unwrap();
        let file_name_pathbuf = PathBuf::from(file_name);
        file_name_pathbuf
    })
    .collect::<Vec<PathBuf>>()
}

fn get_file(file: PathBuf) -> PathBuf
{
    let mut file = file.clone();
    if file.is_dir()
    {
        loop
        {
            file = file.join(select::select_from_dir(dir_to_filename_list(
                file.read_dir().expect(
                    format!("Error: cannot read from dir '{}'", file.to_string_lossy()).as_str(),
                ),
            )));

            if file.is_file()
            {
                break;
            }
        }
    }
    return file;
}

fn main()
{
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let (_file, volume) = parse_args(Opt::from_args());
    let file = get_file(_file);

    let file_handle = File::open(file.clone())
        .expect(format!("Couldn't open file {}", file.to_string_lossy()).as_str());

    let audio = stream_handle
        .play_once(BufReader::new(file_handle))
        .unwrap();
    audio.set_volume(volume);
    drop(volume);

    audio.play();
    println!(
        "Playing '{}' at {}% volume",
        file.to_string_lossy(),
        (volume * 100.0) as u8
    );
    audio.sleep_until_end();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "plsplay")]
struct Opt
{
    /// The audio file to play
    #[structopt(parse(from_str))]
    file: PathBuf,

    /// The playback volume (from 0 to 100)
    #[structopt(short, long, default_value = "100")]
    volume: u8,
}

fn parse_args(opt: Opt) -> (PathBuf, f32)
{
    let file = opt.file;
    let mut volume = opt.volume;
    if volume > 100
    {
        volume = 100;
    }
    return (file, ((volume / 100) as f32));
}
