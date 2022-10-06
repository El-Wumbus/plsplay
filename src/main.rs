use redlux;
use rodio::{OutputStream, Sink};
use souvlaki::{MediaControlEvent, MediaPlayback, MediaControls, PlatformConfig, MediaMetadata};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, Write},
    path::PathBuf,
    process::exit,
};
use structopt::StructOpt;
mod metadata;
use metadata::*;
mod select;

const MAX_VOLUME: u32 = 200;
const PRECENTAGE_CONVERSION: f32 = 100.0;
static mut MODE: Mode = Mode::Play();

enum Mode
{
    Play(),
    Pause(),
    Stop(),
}

fn main()
{
    let (_file, volume, disable_terminal_controls) = parse_args(Opt::from_args());
    let file = select::get_file(_file);

    let file_handle = File::open(file.clone())
        .expect(format!("Couldn't open file {}", file.to_string_lossy()).as_str());

    // Do diffferent things for m4a files.
    match file.extension() // TODO: detect file type from file content https://lib.rs/crates/lofty
    {
        Some(x) => match x.to_str().unwrap()
        {
            "m4a" =>
            {
                let metadata = file_handle.metadata().expect("Error getting file metadata");
                let size = metadata.len();
                let decoder = redlux::Decoder::new_mpeg4(BufReader::new(file_handle), size)
                    .expect("Error creating m4a Decoder");
                let output_stream = OutputStream::try_default();
                let (_stream, handle) = output_stream.expect("Error creating output stream");
                let audio = Sink::try_new(&handle).expect("Error creating sink");
                audio.append(decoder);
                audio_controls(audio, volume, file, disable_terminal_controls);
                return
            }
            _ => (),
        },
        _ => (),
    }
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let audio = stream_handle
        .play_once(BufReader::new(file_handle))
        .unwrap();
    audio_controls(audio, volume, file, disable_terminal_controls);
}

fn audio_controls(sink: Sink, mut volume: f32, file: PathBuf, no_term_controls: bool)
{
    println!(
        "Playing '{}' at {}% volume",
        file.to_string_lossy(),
        (volume * PRECENTAGE_CONVERSION) as u8
    );
    sink.set_volume(volume);
    sink.play();

    let metadata: parse::AudioMetadata = parse::AudioMetadata::from_file(file.clone());
    let (title, artist, album) = (metadata.title, metadata.artist, metadata.album);

    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        use raw_window_handle::Win32WindowHandle;

        let handle: Win32WindowHandle = unimplemented!();
        Some(handle.hwnd)
    };

    let config = PlatformConfig {
        dbus_name: "decator_plsplay",
        display_name: "plsplay",
        hwnd,
    };

    let mut controls = MediaControls::new(config).expect("Error: Unable to create media controls");

    controls
            .attach(move |event: MediaControlEvent| match event
            {
                MediaControlEvent::Pause => unsafe {MODE = Mode::Pause()},
                MediaControlEvent::Play => unsafe {MODE = Mode::Play()},
                MediaControlEvent::Quit => unsafe {MODE = Mode::Stop()},
                _ => (),
            })
            .unwrap();
            
    controls
    .set_metadata(MediaMetadata {
        title: Some(&title),
        album: Some(&album),
        artist: Some(&artist),
        ..Default::default()
    })
    .unwrap();

    loop
    {
        unsafe {
            match MODE
            {
                Mode::Play() =>
                {
                    sink.play();
                    controls
                        .set_playback(MediaPlayback::Playing { progress: None })
                        .unwrap();
                }
                Mode::Pause() =>
                {
                    sink.pause();
                    controls
                        .set_playback(MediaPlayback::Paused { progress: None })
                        .unwrap();
                }
                Mode::Stop() =>
                {
                    sink.stop();
                    controls.set_playback(MediaPlayback::Stopped).unwrap();
                }
            }
        }
        if sink.empty()
        {
            break;
        }
        if no_term_controls
        {
            continue;
        }

        print!("{} by {}:: ", title, artist,);
        stdout().flush().unwrap();
        let mut input: String = String::new();
        stdin().read_line(&mut input).unwrap();
        let input: Vec<&str> = input.trim().split_whitespace().collect();

        if input.len() < 1
        {
            continue;
        }
        match input[0]
        {
            "exit" | "quit" => exit(0),
            "pause" | "pa" => sink.pause(),
            "play" | "pl" => sink.play(),
            "help" =>
            {
                println!("Commands:");
                println!("\tpause  | pa                  [pause playback]");
                println!("\tplay   | pl                  [resume playback]");
                println!("\thelp                         [display help message]");
                println!("\texit   | quit                [Close the program]");
                println!("\tvolume | vol <target volume> [View or adjust volume]");
            }

            "volume" | "vol" =>
            {
                if input.len() > 1
                {
                    let mut parsed: u32 = match input[1].parse()
                    {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                    if parsed > MAX_VOLUME
                    {
                        parsed = MAX_VOLUME;
                        println!("{}", parsed as f32);
                    }

                    volume = parsed as f32 / PRECENTAGE_CONVERSION;
                    sink.set_volume(volume);
                }
                else
                {
                    println!("Volume: {}%", volume * PRECENTAGE_CONVERSION)
                }
            }
            _ => continue,
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "plsplay")]
struct Opt
{
    /// The audio file to play
    #[structopt(parse(from_str))]
    file: PathBuf,

    /// The playback volume (from 0 to 200)
    #[structopt(short, long, default_value = "100")]
    volume: u32,

    /// Disable interactive command line controls
    #[structopt(short, long)]
    disable_terminal_controls: bool,
}

fn parse_args(opt: Opt) -> (PathBuf, f32, bool)
{
    let file = opt.file;
    let mut volume = opt.volume;
    if volume > MAX_VOLUME as u32
    {
        volume = MAX_VOLUME;
    }
    return (
        file,
        ((volume / PRECENTAGE_CONVERSION as u32) as f32),
        opt.disable_terminal_controls,
    );
}

