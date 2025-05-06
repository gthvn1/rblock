use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

pub struct Qcow2 {
    file: File,
}

impl Qcow2 {
    pub fn new(fname: &str) -> io::Result<Self> {
        let file = File::open(fname)?;
        Ok(Self { file })
    }

    pub fn is_qcow(&mut self) -> bool {
        let result: Result<bool, std::io::Error> = (|| {
            let mut magic: [u8; 4] = [0; 4];
            self.file.seek(SeekFrom::Start(0))?;
            self.file.read_exact(&mut magic)?;
            Ok(magic == [0x51, 0x46, 0x49, 0xfb])
        })();

        result.unwrap_or(false)
    }
}
