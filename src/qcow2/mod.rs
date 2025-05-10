use log::{debug, error};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::FileExt;

// https://github.com/qemu/qemu/blob/master/docs/interop/qcow2.txt
#[derive(Debug)]
pub enum Qcow2Field {
    Magic,
    Version,
    BackingFileOffset,
    BackingFileSize,
    ClusterBits,
    Size,
    CryptMethod,
    L1Size,
    L1TableOffset,
    RefcountTableOffset,
    RefcountTableClusters,
    NbSnapshots,
    SnapshotsOffset,
    // Only for version >= 3:
    IncompatibleFeatures,
    CompatibleFeatures,
    AutoclearFeatures,
    RefcountOrder,
    HeaderLength,
}

impl Qcow2Field {
    pub fn range(&self) -> (usize, usize) {
        match self {
            Qcow2Field::Magic => (0, 3),
            Qcow2Field::Version => (4, 7),
            Qcow2Field::BackingFileOffset => (8, 15),
            Qcow2Field::BackingFileSize => (16, 19),
            Qcow2Field::ClusterBits => (20, 23),
            Qcow2Field::Size => (24, 31),
            Qcow2Field::CryptMethod => (32, 35),
            Qcow2Field::L1Size => (36, 39),
            Qcow2Field::L1TableOffset => (40, 47),
            Qcow2Field::RefcountTableOffset => (48, 55),
            Qcow2Field::RefcountTableClusters => (56, 59),
            Qcow2Field::NbSnapshots => (60, 63),
            Qcow2Field::SnapshotsOffset => (64, 71),
            Qcow2Field::IncompatibleFeatures => (72, 79),
            Qcow2Field::CompatibleFeatures => (80, 87),
            Qcow2Field::AutoclearFeatures => (88, 95),
            Qcow2Field::RefcountOrder => (96, 99),
            Qcow2Field::HeaderLength => (100, 103),
        }
    }
}

#[allow(dead_code)]
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

        // Print some information before returning
        let q = Qcow2 {
            file,
            version,
            header,
        };

        debug!("header length         : {}", q.header_len());
        debug!("backing file          : {:?}", q.backing_file());
        debug!("cluster size          : {}", q.cluster_size());
        debug!("virtual size          : {}", q.virtual_size());
        debug!("L1 size               : {}", q.l1_size());
        debug!("L1 table offset       : 0x{:08x}", q.l1_table_offset());
        debug!("refcount width        : {}", q.refcount_width());
        debug!(
            "refcount table offset : 0x{:08x}",
            q.refcount_table_offset()
        );

        Ok(q)
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn header_len(&self) -> usize {
        self.header.len()
    }

    pub fn virtual_size(&self) -> u64 {
        let (off_begin, off_end) = Qcow2Field::Size.range();

        match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u64::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to convert virtual size: {}", e);
                0
            }
        }
    }

    pub fn backing_file(&self) -> Option<String> {
        let (off_begin, off_end) = Qcow2Field::BackingFileOffset.range();
        let (sz_begin, sz_end) = Qcow2Field::BackingFileSize.range();

        let offset = match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u64::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to convert backing file offset: {}", e);
                return None;
            }
        };

        let sz = match self.header[sz_begin..=sz_end].try_into() {
            Ok(bytes) => u32::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to convert backing file size: {}", e);
                return None;
            }
        };

        let mut buf = vec![0u8; sz as usize];
        let _bytes_read = match self.file.read_at(&mut buf, offset) {
            Ok(n) => n,
            Err(e) => {
                error!("Failed to read backing file name: {}", e);
                return None;
            }
        };

        let filename = match String::from_utf8(buf) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to convert backing file name to string: {}", e);
                return None;
            }
        };

        Some(filename)
    }

    pub fn cluster_size(&self) -> usize {
        let (off_begin, off_end) = Qcow2Field::ClusterBits.range();

        let cluster_bits = match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u32::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to get cluser bits: {}", e);
                return 0;
            }
        };

        (1 << cluster_bits) as usize
    }

    pub fn l1_size(&self) -> u32 {
        let (off_begin, off_end) = Qcow2Field::L1Size.range();

        match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u32::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to get L1 size: {}", e);
                0
            }
        }
    }

    pub fn l1_table_offset(&self) -> u64 {
        let (off_begin, off_end) = Qcow2Field::L1TableOffset.range();

        match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u64::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to get L1 table offset: {}", e);
                0
            }
        }
    }

    pub fn refcount_table_offset(&self) -> u64 {
        let (off_begin, off_end) = Qcow2Field::RefcountTableOffset.range();

        match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u64::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to get refcount table offset: {}", e);
                0
            }
        }
    }

    pub fn refcount_width(&self) -> u32 {
        let (off_begin, off_end) = Qcow2Field::RefcountOrder.range();

        let order = match self.header[off_begin..=off_end].try_into() {
            Ok(bytes) => u32::from_be_bytes(bytes),
            Err(e) => {
                error!("failed to get refcount order: {}", e);
                0
            }
        };

        1 << order
    }
}
