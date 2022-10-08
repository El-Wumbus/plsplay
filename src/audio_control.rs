const TICK_RATE: u64 = 200;
#[allow(unused)]
use {
    super::{ansi, get::get_metadata, MAX_VOLUME, PRECENTAGE_CONVERSION},
    human_repr::HumanDuration,
    rodio::Sink,
    souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig},
    std::{
        io::{stdin, stdout, Write},
        path::PathBuf,
        process::exit,
        thread,
        time::Duration,
    },
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

#[allow(unused)]
macro_rules! mode_continue {
    () => {
        unsafe { MODE.clear_actions() };
        continue
    };
}

#[allow(unused)]
macro_rules! change_mode {
    ($a:expr) => {
        unsafe { MODE = $a }
    };
}

pub mod cli
{
    use super::*;

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
                    if !PAUSED || !END
                    {
                        COUNT += 0.2;
                    }
                }
                thread::sleep(Duration::from_millis(TICK_RATE));
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
                true =>
                unsafe { PAUSED = true },
                false =>
                unsafe { PAUSED = false },
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
                        println!("Volume: {}%", (volume * PRECENTAGE_CONVERSION) as u32)
                    }
                }
                _ =>
                {
                    unsafe { MODE.clear_actions() };
                    continue;
                }
            }
        }
    }
}

pub mod tui
{
    use super::*;
    enum Event<I>
    {
        Input(I),
        Tick,
    }

    use std::{
        io,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    use super::TICK_RATE;
    use ::tui::{
        backend::CrosstermBackend,
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Style},
        text::{Span, Spans},
        widgets::{Block, BorderType, Borders, Paragraph},
        Terminal,
    };
    use crossterm::{
        event::{self, Event as CEvent, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    };

    pub fn audio_controls(sink: Sink, volume: f32, file: PathBuf)
    {
        let (title, artist, album, duration) = get_metadata(file);
        sink.set_volume(volume);
        sink.play();
        let _counter = thread::spawn(|| {
            loop
            {
                unsafe {
                    if !PAUSED || END
                    {
                        COUNT += 1.0;
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        if sink.empty()
        {
            unsafe { END = true };
        }
        match sink.is_paused()
        {
            true =>
            unsafe { PAUSED = true },
            false =>
            unsafe { PAUSED = false },
        }

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
                    title: Some(&title.clone()),
                    album: Some(&album),
                    artist: Some(&artist),
                    ..Default::default()
                })
                .unwrap();
            controls
        };

        enable_raw_mode().expect("Error: can't run in raw mode (Try using the cli instead).");
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(TICK_RATE);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop
            {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll doesn't work")
                {
                    if let CEvent::Key(key) = event::read().expect("can't read events")
                    {
                        tx.send(Event::Input(key)).expect("can't send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate
                {
                    if let Ok(_) = tx.send(Event::Tick)
                    {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();
        // let menu_titles = vec!["Main"];
        let active_menu_item = MenuItem::Main;

        loop
        {
            if sink.empty()
            {
                unsafe { END = true };
            }
            match sink.is_paused()
            {
                true =>
                unsafe { PAUSED = true },
                false =>
                unsafe { PAUSED = false },
            }
            let volume = sink.volume();

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

            terminal
                .draw(|rect| {
                    let size = rect.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Min(2),
                                Constraint::Length(3),
                            ]
                            .as_ref(),
                        )
                        .split(size);

                    match active_menu_item
                    {
                        MenuItem::Main => rect.render_widget(
                            render_main(title.clone(), volume, duration, unsafe { COUNT }),
                            chunks[1],
                        ),
                    }
                })
                .unwrap();

            match rx.recv().unwrap()
            {
                Event::Input(event) => match event.code
                {
                    // KeyCode::Char('h') => active_menu_item = MenuItem::Main,
                    KeyCode::Char('q') =>
                    {
                        // Quit
                        terminal.clear().unwrap();
                        disable_raw_mode().unwrap();
                        terminal.show_cursor().unwrap();
                        change_mode!(Mode::Stop());
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::PageUp =>
                    {
                        let mut vol = (volume * PRECENTAGE_CONVERSION) as u32 + 10;
                        if vol > MAX_VOLUME
                        {
                            vol = MAX_VOLUME;
                        }
                
                        change_mode!(Mode::Volume(vol as f32 / PRECENTAGE_CONVERSION));
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::PageDown =>
                    {
                        let mut vol:u32 = ((volume * PRECENTAGE_CONVERSION) as u32 - 10) as u32;
                        if vol > MAX_VOLUME
                        {
                            vol = MAX_VOLUME;
                        }
                        change_mode!(Mode::Volume(vol as f32 / PRECENTAGE_CONVERSION));
                    }
                    KeyCode::Char('p') => match unsafe { PAUSED }
                    {
                        true => change_mode!(Mode::Play()),
                        false => change_mode!(Mode::Pause()),
                    },
                    // TODO: Allow for selecting a file to play like in the cli version
                    KeyCode::Down =>
                    {
                        // TODO: Go down on select page
                    }
                    KeyCode::Up =>
                    {
                        // TODO: Go up on select page
                    }
                    _ =>
                    {}
                },
                Event::Tick =>
                {}
            }
        }
    }
    #[derive(Copy, Clone)]
    enum MenuItem
    {
        Main,
    }

    impl From<MenuItem> for usize
    {
        fn from(input: MenuItem) -> usize
        {
            match input
            {
                MenuItem::Main => 0,
            }
        }
    }

    fn render_main<'a>(track: String, volume: f32, duration: f32, count: f32) -> Paragraph<'a>
    {
        let time_style = match unsafe { END }
        {
            true => Style::default().fg(Color::Red),
            false => Style::default(),
        };
        Paragraph::new(vec![
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![
                Span::raw(match unsafe {PAUSED}{
                    true => "[Paused] ",
                    false => "[Playing] ",
                }),
                Span::styled(track, Style::default().fg(Color::Green)),
                Span::raw(" ["),
                Span::styled(format!("{}", (count as u32).human_duration()), time_style),
                Span::raw("/"),
                Span::styled(
                    format!("{}", (duration as u32).human_duration()),
                    time_style,
                ),
                Span::raw("]"),
            ]),
            Spans::from(vec![
                Span::raw("Volume: "),
                Span::styled(
                    format!("{}", (volume * PRECENTAGE_CONVERSION) as u32),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw(
                "Press 'p' to toggle play status, '+' or '=' to increase volume,",
            )]),
            Spans::from(vec![Span::raw(
                " '-' or '_' to decrease volume, and 'q' to quit the program.",
            )]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("PlsPlay")
                .border_type(BorderType::Plain),
        )
    }
}
