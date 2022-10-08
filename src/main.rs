
use lofty::FileType;
use redlux;
use rodio::{OutputStream, Sink};
use std::{fs::File, io::BufReader, path::PathBuf};
use structopt::StructOpt;

mod ansi;
mod audio_control;
mod get;
mod metadata;
mod run;

const MAX_VOLUME: u32 = 200;
const PRECENTAGE_CONVERSION: f32 = 100.0;

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

    /// Use TUI instead of CLI
    #[structopt(short, long)]
    tui: bool,
}

fn parse_args(opt: Opt) -> (PathBuf, f32, bool, bool)
{
    let file = opt.file;
    let mut pvolume = opt.volume;
    if pvolume > MAX_VOLUME
    {
        pvolume = MAX_VOLUME;
    }
    let volume = pvolume as f32 / PRECENTAGE_CONVERSION;
    return (file, volume, opt.disable_terminal_controls, opt.tui);
}

fn main()
{
    let (_file, volume, disable_terminal_controls, tui) = parse_args(Opt::from_args());
    let file = get::get_file(_file);

    run::run(file, volume, disable_terminal_controls, tui);
    return;
}
