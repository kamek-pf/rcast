use std::fs::File;
use std::path::{Path, PathBuf};

use matroska::Matroska;

// Supported :
// MP4 and WebM encoded with Video codecs H.264 High Profile Level 4.1, 4.2 and 5, VP8
// and audio codecs HE-AAC, LC-AAC, CELT/Opus, MP3, Vorbis,
// Containers : AVI, MKV, FLV, MOV, VOB, 3G2

// Unsupported :
// M2TS, 3GP, DIVX, RM, RMVB, ASF, TS, DV, F4V, OGV, TOD
// Audio : AC3 ?
// watch number of channels ?

#[derive(Debug)]
pub struct Media {
    pub path: PathBuf,
    pub info: MediaInfo,
}

type Error = MediaError;

impl Media {
    pub fn new(from: &str) -> Result<Self, Error> {
        let path = Path::new(from);
        let mut file = File::open(&path).map_err(|_| MediaError::FileNotFound)?;
        let file = Matroska::open(file).map_err(|_| MediaError::UnsupportedFormat)?;

        let media = Media {
            path: path.to_path_buf(),
            info: Self::get_info(&path)?,
        };

        Ok(media)
    }

    fn get_info(path: &Path) -> Result<MediaInfo, MediaError> {
        unimplemented!()
    }

    fn get_container(file: &Path) -> Option<Container> {
        file.extension()
            .and_then(|os_str| os_str.to_str())
            .and_then(|e| match e.to_lowercase().as_ref() {
                "mkv" => Some(Container::Mkv),
                "webm" => Some(Container::Webm),
                _ => None
            })
    }

    fn get_video_codec(file: &Matroska) -> VideoCodec {
        unimplemented!()
    }

    fn get_audio_codec(file: &Matroska) -> AudioCodec {
        unimplemented!()
    }

    fn get_audio_channels(file: &Matroska) -> u8 {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq)]
pub struct MediaInfo {
    pub container: Container,
    pub video_codec: VideoCodec,
    pub audio_codec: AudioCodec,
    pub audio_channels: u8,
}

#[derive(Debug, PartialEq)]
pub enum Container {
    Mkv,
    Webm,
}

#[derive(Debug, PartialEq)]
pub enum VideoCodec {
    Mpeg4Avc, // h.264
    Vp8,
}

#[derive(Debug, PartialEq)]
pub enum AudioCodec {
    Aac,
    Ac3,
    Mp3,
    Opus,
    Vorbis,
}

#[derive(Debug, Fail, PartialEq)]
pub enum MediaError {
    #[fail(display = "File not found")]
    FileNotFound,
    #[fail(display = "Unsupported file format")]
    UnsupportedFormat,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn container_parser() {
        let c1 = Media::get_container(Path::new("jaja.MKV"));
        let c2 = Media::get_container(Path::new("jaja.webm"));
        let c3 = Media::get_container(Path::new("jaja.noob"));
        let c4 = Media::get_container(Path::new("jaja"));
        assert_eq!(c1, Some(Container::Mkv));
        assert_eq!(c2, Some(Container::Webm));
        assert!(c3.is_none());
        assert!(c4.is_none());
    }
}
