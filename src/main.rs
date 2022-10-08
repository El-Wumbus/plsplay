use human_repr::{HumanDuration};
use lofty::FileType;
use redlux;
use rodio::{OutputStream, Sink};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, Write},
    path::PathBuf,
    process::exit,
    thread,
    time::Duration,
};
use structopt::StructOpt;
mod metadata;

use get::get_metadata;
mod ansi;
mod get;

const MAX_VOLUME: u32 = 200;
const PRECENTAGE_CONVERSION: f32 = 100.0;
static mut MODE: Mode = Mode::Play();
static mut COUNT: f32 = 0.0;
static mut END: bool = false;

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
    let mut pvolume = opt.volume;
    if pvolume > MAX_VOLUME
    {
        pvolume = MAX_VOLUME;
    }
    let volume = pvolume as f32 / PRECENTAGE_CONVERSION;
    return (file, volume, opt.disable_terminal_controls);
}

fn main()
{
    let (_file, volume, disable_terminal_controls) = parse_args(Opt::from_args());
    let file = get::get_file(_file);

    match get::file_type(file.clone()) // TODO: detect file type from file content https://lib.rs/crates/lofty
        {
            Some(x) => match x
            {
                FileType::MPEG | FileType::MP4 =>
                {
                    let metadata = File::open(file.clone())
                    .expect(format!("Couldn't open file {}", file.to_string_lossy()).as_str()).metadata().expect("Error getting file metadata");
                    let size = metadata.len();
                    let source = redlux::Decoder::new_mpeg4(BufReader::new(File::open(file.clone())
                    .expect(format!("Couldn't open file {}", file.to_string_lossy()).as_str())), size)
                        .expect("Error: Failed to decode MPEG file!");
                    let output_stream = OutputStream::try_default();
                    let (_stream, stream_handle) = output_stream.expect("Error: Couldn't create MPEG4 output stream");
                    let audio = Sink::try_new(&stream_handle).expect("Error creating sink");
                    audio.append(source);
                    audio_controls(audio, volume, file, disable_terminal_controls);
                    return;
                },
                _ => (),
            },
            _ => (),
        }
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let audio = stream_handle
        .play_once(BufReader::new(File::open(file.clone()).expect(
            format!("Couldn't open file {}", file.to_string_lossy()).as_str(),
        )))
        .unwrap();
    audio_controls(audio, volume, file, disable_terminal_controls);
    return;
}

enum Mode
{
    Play(),
    Pause(),
    Stop(),
    Volume(f32),
    Nil(),
}

impl Mode
{
    fn clear_actions(&mut self) { *self = Mode::Nil(); }
}

macro_rules! mode_continue {
    () => {
        unsafe { MODE.clear_actions() };
        continue
    };
}

macro_rules! change_mode {
    ($a:expr) => {
        unsafe { MODE = $a }
    };
}

fn audio_controls(sink: Sink, mut volume: f32, file: PathBuf, no_term_controls: bool)
{
    let (title, artist, album, duration) = get_metadata(file);
    println!(
        "Playing '{}{}{}' at {}% volume",
        ansi::Ansi::GRN,
        title.clone(),
        ansi::Ansi::COLOR_END,
        (volume * PRECENTAGE_CONVERSION) as u8
    );
    sink.set_volume(volume);
    sink.play();
    let _counter = thread::spawn(|| {
        loop
        {
            thread::sleep(Duration::from_millis(100));
            unsafe { COUNT += 0.1 };
        }
    });

    #[cfg(target_os = "linux")]
    let mut controls = {
        let config = PlatformConfig {
            dbus_name: "decator_plsplay",
            display_name: "plsplay",
            hwnd: None,
        };

        let mut controls =
            MediaControls::new(config).expect("Error: Unable to create media controls");

        controls
            .attach(move |event: MediaControlEvent| match event
            {
                MediaControlEvent::Pause => change_mode!(Mode::Pause()),
                MediaControlEvent::Play => change_mode!(Mode::Play()),
                MediaControlEvent::Quit => change_mode!(Mode::Stop()),
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
        controls
    };

    loop
    {

        if sink.empty()
        {
            unsafe {END = true};
        }

        // Take actions previously selected.
        // This is unsafe due to use of mutable static globals
        unsafe {
            match MODE
            {
                Mode::Play() =>
                {
                    sink.play();
                    #[cfg(target_os = "linux")]
                    controls
                        .set_playback(MediaPlayback::Playing { progress: None })
                        .unwrap();
                }
                Mode::Pause() =>
                {
                    sink.pause();
                    #[cfg(target_os = "linux")]
                    controls
                        .set_playback(MediaPlayback::Paused { progress: None })
                        .unwrap();
                }
                Mode::Stop() =>
                {
                    sink.stop();
                    #[cfg(target_os = "linux")]
                    controls.set_playback(MediaPlayback::Stopped).unwrap();
                    exit(0);
                }
                Mode::Volume(x) =>
                {
                    sink.set_volume(x);
                }
                Mode::Nil() => (),
            }
        }

        if no_term_controls
        {
            mode_continue!();
        }

        if !unsafe{END}
        {
            print!(
                "{}{}{} [{}/{}]:: ",
                ansi::Ansi::GRN,
                title,
                ansi::Ansi::COLOR_END,
                unsafe { COUNT.human_duration() },
                duration.human_duration()
            );
        }
        else
        {
            print!(
                "{}{}{} [{}END{}]:: ",
                ansi::Ansi::GRN,
                title,
                ansi::Ansi::COLOR_END,
                ansi::Ansi::RED,
                ansi::Ansi::COLOR_END,
            );
        }
        
        stdout().flush().unwrap();
        let mut input: String = String::new();
        stdin().read_line(&mut input).unwrap();
        let input: Vec<&str> = input.trim().split_whitespace().collect();

        if input.len() < 1
        {
            mode_continue!();
        }
        match input[0]
        {
            "exit" | "quit" | "Stop" => exit(0),
            "pause" | "pa" => change_mode!(Mode::Pause()),
            "play" | "pl" => change_mode!(Mode::Play()),
            "remaining" | "rem" =>
            {
                let time_remaining = unsafe { duration - COUNT };
                println!("{}", time_remaining.human_duration());
            }
            "duration" | "dur" =>
            {
                println!("{}", duration.human_duration());
            }
            "help" =>
            {
                println!("Commands:");
                println!("\tpause  | pa                  [pause playback]");
                println!("\tplay   | pl                  [resume playback]");
                println!("\thelp                         [display help message]");
                println!("\texit   | quit                [Close the program]");
                println!("\tvolume | vol <target volume> [View or adjust volume]");
                mode_continue!();
            }

            "volume" | "vol" =>
            {
                if input.len() > 1
                {
                    let mut parsed: u32 = match input[1].parse()
                    {
                        Ok(x) => x,
                        Err(_) =>
                        {
                            mode_continue!();
                        }
                    };

                    if parsed > MAX_VOLUME
                    {
                        parsed = MAX_VOLUME;
                    }

                    volume = parsed as f32 / PRECENTAGE_CONVERSION;
                    change_mode!(Mode::Volume(volume));
                }
                else
                {
                    println!("Volume: {}%", volume * PRECENTAGE_CONVERSION)
                }
            }
            _ =>
            {
                // So any action previously taken isn't repeated, clear MODE.
                mode_continue!();
            }
        }
    }
}
