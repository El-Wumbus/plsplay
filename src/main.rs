use redlux;
use rodio::{OutputStream, Sink};
use std::{
    fs::File,
    io::{stdin, stdout, BufReader, Write},
    path::PathBuf,
    process::exit,
};
use structopt::StructOpt;
mod select;

const MAX_VOLUME: u32 = 200;
const PRECENTAGE_CONVERSION: f32 = 100.0;

fn main()
{
    let (_file, volume) = parse_args(Opt::from_args());
    let file = select::get_file(_file);

    let file_handle = File::open(file.clone())
        .expect(format!("Couldn't open file {}", file.to_string_lossy()).as_str());

    // Do diffferent things for m4a files.
    match file.extension()
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
                audio_controls(audio, volume, file);
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
            audio_controls(audio, volume, file);
}

fn audio_controls(sink: Sink, mut volume: f32, file: PathBuf)
{
    println!(
        "Playing '{}' at {}% volume",
        file.to_string_lossy(),
        (volume * PRECENTAGE_CONVERSION) as u8
    );
    sink.set_volume(volume);
    sink.play();

    loop
    {
        if sink.empty()
        {
            break;
        }
        print!(
            "{}:: ",
            match file.file_stem()
            {
                Some(x) => format!("{} ", x.to_string_lossy()),
                _ => "".to_string(),
            }
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
}

fn parse_args(opt: Opt) -> (PathBuf, f32)
{
    let file = opt.file;
    let mut volume = opt.volume;
    if volume > MAX_VOLUME
    {
        volume = MAX_VOLUME;
    }
    return (file, ((volume / PRECENTAGE_CONVERSION as u32) as f32));
}
