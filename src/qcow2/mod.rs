use std::fs::File;
use std::io::{self, Read};

pub struct Qcow2 {
    file: File,
    version: u32,
    header: Vec<u8>,
}

impl Qcow2 {
    pub fn new(fname: &str) -> io::Result<Self> {
        let mut file = File::open(fname)?;
        const EXPECTED_MAGIC: [u8; 4] = [0x51, 0x46, 0x49, 0xfb];
        let mut magic: [u8; 4] = [0; 4];
        file.read_exact(&mut magic)?;

        if magic != EXPECTED_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic number {:?}", magic),
            ));
        }

        let mut version_bytes: [u8; 4] = [0; 4];
        file.read_exact(&mut version_bytes)?;
        let version = u32::from_be_bytes(version_bytes);

        let mut header = Vec::with_capacity(104);
        header.extend_from_slice(&magic);
        header.extend_from_slice(&version_bytes);

        // We can read the rest of the header. If version is 2 only the
        // first 72 bytes will be relevant.
        let mut rest: [u8; 96] = [0; 96];
        file.read_exact(&mut rest)?;
        header.extend_from_slice(&rest);

        // Now we need to check the real size if version is 3
        if version == 3 {
            let header_length = u32::from_be_bytes(
                header[100..=103]
                    .try_into()
                    .expect("Failed to convert header"),
            );

            // Sanity check
            if header_length < 104 || header_length % 8 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid header length {}", header_length),
                ));
            }

            if header_length > 104 {
                // We need to add extra stuff
                let mut extra = vec![0u8; (header_length - 104) as usize];
                file.read_exact(&mut extra)?;
                header.extend_from_slice(&extra);
            }
        }

        Ok(Self {
            file,
            version,
            header,
        })
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}
