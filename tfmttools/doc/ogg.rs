use anyhow::Result;
use std::path::{Path, PathBuf};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;

pub struct OGG {
    pub path: PathBuf,
    comments: Comments,
}

impl OGG {
    fn check_bytes(reader: &mut BufReader<File>, bytes: &[u8]) -> bool {
        let size = bytes.len();

        let mut buffer = vec![0; size];

        if let Err(_) = reader.take(size.into()).read(&mut buffer) {
            return false
        }

        buffer == bytes
    }

    fn validate_page_header(reader: &mut BufReader<File>) -> Result<()> {

        // capture pattern
        if OGG::check_bytes(reader, &[0x4f, 0x67, 0x67, 0x53]) == false {
            anyhow::bail!("Can't validate OGG capture pattern!")
        }

        // stream structure version
        if OGG::check_bytes(reader, &[0x00]) == false {
            anyhow::bail!("Can't validate OGG stream structure version!")
        }

        // header type flag
        reader.seek(SeekFrom::Current(1))?;
        // let mut header_type_flag = [0; 1];
        // reader.read_exact(&mut header_type_flag)?;

        // let header_type_flag = header_type_flag[0];

        // if header_type_flag & 0b_0000_0001 == 0b_0000_0001 {
        //     println!("Continued packet")
        // } else {
        //     println!("Fresh packet")
        // }

        // if header_type_flag & 0b_0000_0010 == 0b_0000_0010 {
        //     println!("first page of logical bitstream (bos)")
        // }

        // if header_type_flag & 0b_0000_0100 == 0b_0000_0100 {
        //     println!("last page of logical bitstream (bos)")
        // }

        // absolute granule position: 8
        // stream serial number: 4
        // page sequence no: 4
        // page checksum: 4
        reader.seek(SeekFrom::Current(20))?;

        let mut page_segments = [0; 1];
        reader.read_exact(&mut page_segments)?;
        let page_segments = page_segments[0];

        // page_segments+26?
        // segment_table (containing packet lacing values)
        reader.seek(SeekFrom::Current(page_segments.into()))?;

        Ok(())
    }


    fn read_from_path(path: &Path) -> Result<()> {
        let file = std::fs::File::open(path)?;

        let mut reader = BufReader::new(file);

        OGG::validate_page_header(&mut reader)?;

        Ok(())
    }
}

pub struct Comments;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ogg_test() -> Result<()> {
        OGG::read_from_path(&PathBuf::from("testdata/music/Welcome To Heaven - Damjan Mravunac.ogg"))
    }
}

// 0x4f    O
// 0x67    g
// 0x67    g
// 0x53    S
// 0x0     stream structure version
// 0x2     header_type_flag
// 0x0     absolute granule position
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0     absolute granule position
// 0xaf    stream serial number
// 0x94
// 0xbd
// 0xac    stream serial number
// 0x0     page sequence no
// 0x0
// 0x0
// 0x0     page sequence no
// 0x59    page checksum
// 0x65
// 0xf
// 0x18    page checksum
// 0x1     page_segments
// 0x1e    page_segments
// 0x1
// 0x76
// 0x6f
// 0x72
// 0x62
// 0x69
// 0x73
// 0x0
// 0x0
// 0x0
// 0x0
// 0x1
// 0x44
// 0xac
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0
// 0x71
// 0x2
// 0x0
// 0x0
// 0x0
// 0x0
// 0x0
// 0xb8
// 0x1
// 0x4f
// 0x67
// 0x67
// 0x53
