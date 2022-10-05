use lofty::{self, Accessor, AudioFile};
use std::path::PathBuf;

pub struct AudioMetadata
{
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f32,
}

impl AudioMetadata
{
    pub fn from_file(file_path: PathBuf) -> AudioMetadata
    {
        let file = lofty::Probe::open(file_path)
            .expect("ERROR: Bad path provided!")
            .read(true)
            .expect("ERROR: Failed to read file!");
        let tag = match file.primary_tag()
        {
            Some(x) => x,
            None => file.first_tag().unwrap(),
        };

        let properties = file.properties();
        let data: AudioMetadata = AudioMetadata {
            title: tag.title().unwrap_or("NULL").to_string(),
            artist: tag.artist().unwrap_or("NULL").to_string(),
            album: tag.album().unwrap_or("NULL").to_string(),
            duration: properties.duration().as_secs_f32(),
        };

        data
    }
}
