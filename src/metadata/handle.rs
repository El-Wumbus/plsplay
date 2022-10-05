use souvlaki::{MediaControls, MediaMetadata, PlatformConfig};
use super::parse::AudioMetadata;


#[allow(unused)]
impl AudioMetadata
{
    pub fn handle(self) -> MediaControls
    {
        handle_controls(self)
    }
}

pub fn handle_controls(metadata: AudioMetadata) -> MediaControls
{
    let (title, artist, album) = (metadata.title, metadata.artist, metadata.album);

    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        use raw_window_handle::windows::WindowsHandle;

        let handle: WindowsHandle = unimplemented!();
        Some(handle.hwnd)
    };

    let config = PlatformConfig {
        dbus_name: "plsplay",
        display_name: "PlsPlay",
        hwnd,
    };

    let mut controls = MediaControls::new(config).unwrap();
    
    controls
        .set_metadata(MediaMetadata {
            title: Some(&title),
            artist: Some(&artist),
            album: Some(&album),
            ..Default::default()
        })
        .unwrap();
    controls

}