use anyhow::Result;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::Accessor;
use std::time::Duration;

pub struct SongMetaData {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Duration,
    pub lyrics: Option<String>,
}

pub fn extract_metadata(file_path: &str) -> Result<SongMetaData> {
    let tagged_file = read_from_path(file_path)?;

    let properties = tagged_file.properties();
    let duration = Duration::from_millis(properties.duration().as_millis() as u64);

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let metadata = SongMetaData {
        title: tag.as_ref().and_then(|t| t.title().map(String::from)),
        artist: tag.as_ref().and_then(|t| t.artist().map(String::from)),
        album: tag.as_ref().and_then(|t| t.album().map(String::from)),
        duration,
        lyrics: tag
            .as_ref()
            .and_then(|t| t.get_string(&lofty::tag::ItemKey::Lyrics).map(String::from)),
    };

    Ok(metadata)
}
