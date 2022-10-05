use redlux;
use rodio::{OutputStream, Sink};
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

    // TODO: parse metadata 
    let metadata:parse::AudioMetadata = parse::AudioMetadata::from_file(file.clone());

    // let mut controls = handle::handle_controls(metadata);
    // controls
    //     .attach(|event: MediaControlEvent| match event
    //     {
    //         MediaControlEvent::Pause => sink.pause(),
    //         MediaControlEvent::Play => sink.play(),
    //         MediaControlEvent::Quit => sink.stop(),
    //         _ => (),
    //     })
    //     .unwrap();

    loop
    {
        if sink.empty()
        {
            break;
        }
        if no_term_controls
        {
            continue;
        }

        print!(
            "{}:: ",
            metadata.title
        );
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
            },

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
            },
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
    #[structopt(short,long)]
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
    return (file, ((volume / PRECENTAGE_CONVERSION as u32) as f32), opt.disable_terminal_controls);
}

// Todo: Parse metadata