/// `AudioFile` trait
pub mod audio_file;
/// MP3 implementation (powered by id3)
pub mod mp3;
/// Ogg implementation (powered by lewton)
pub mod ogg;

pub use audio_file::AudioFile;
pub use mp3::MP3;
pub use ogg::OGG;
