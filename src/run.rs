use super::*;

pub fn run(file:PathBuf, volume:f32, disable_terminal_controls:bool, tui:bool)
{
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
                    if !tui{

                        audio_control::cli::audio_controls(audio, volume, file, disable_terminal_controls);
                    }
                    else
                    {
                        
                        audio_control::tui::audio_controls(audio, volume, file);
                    }
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
        if !tui{

            audio_control::cli::audio_controls(audio, volume, file, disable_terminal_controls);
        }
        else
        {
            audio_control::tui::audio_controls(audio, volume, file);
        }
    }