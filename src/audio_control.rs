pub mod cli
{
    use super::super::{ansi, get::get_metadata, MAX_VOLUME, PRECENTAGE_CONVERSION};
    use human_repr::HumanDuration;
    use rodio::Sink;
    use souvlaki::{
        MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
    };
    use std::{
        io::{stdin, stdout, Write},
        path::PathBuf,
        process::exit,
        thread,
        time::Duration,
    };

    static mut MODE: Mode = Mode::Play();
    static mut COUNT: f32 = 0.0;
    static mut END: bool = false;
    static mut PAUSED: bool = true;

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

    pub fn audio_controls(sink: Sink, mut volume: f32, file: PathBuf, no_term_controls: bool)
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
                
                unsafe {
                    if !PAUSED
                    {
                        COUNT += 0.1;
                    }
                }
                thread::sleep(Duration::from_millis(100));
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
                unsafe { END = true };
            }
            match sink.is_paused()
            {
                true => unsafe {PAUSED=true},
                false => unsafe {PAUSED=false},
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

            if !unsafe { END }
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
                _ => {
                    unsafe { MODE.clear_actions() };
                    continue},
            }
        }
    }
}
