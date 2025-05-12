use std::io;
use std::os::unix::fs::FileExt;

// https://github.com/qemu/qemu/blob/master/docs/interop/qcow2.txt
#[derive(Debug, Copy, Clone)]
pub enum Qcow2Field {
    Magic = 0,                  // 4 bytes
    Version = 4,                // 4
    BackingFileOffset = 8,      // 8
    BackingFileSize = 16,       // 4
    ClusterBits = 20,           // 4
    Size = 24,                  // 8
    CryptMethod = 32,           // 4
    L1Size = 36,                // 4
    L1TableOffset = 40,         // 8
    RefcountTableOffset = 48,   // 8
    RefcountTableClusters = 56, // 4
    NbSnapshots = 60,           // 4
    SnapshotsOffset = 64,       // 8
    // Only for version >= 3:
    IncompatibleFeatures = 72, // 8
    CompatibleFeatures = 80,   // 8
    AutoclearFeatures = 88,    // 8
    RefcountOrder = 96,        // 4
    HeaderLength = 100,        // 4
}

impl Qcow2Field {
    fn size(&self) -> usize {
        match self {
            Qcow2Field::Magic => 4,
            Qcow2Field::Version => 4,
            Qcow2Field::BackingFileOffset => 8,
            Qcow2Field::BackingFileSize => 4,
            Qcow2Field::ClusterBits => 4,
            Qcow2Field::Size => 8,
            Qcow2Field::CryptMethod => 4,
            Qcow2Field::L1Size => 4,
            Qcow2Field::L1TableOffset => 8,
            Qcow2Field::RefcountTableOffset => 8,
            Qcow2Field::RefcountTableClusters => 4,
            Qcow2Field::NbSnapshots => 4,
            Qcow2Field::SnapshotsOffset => 8,
            Qcow2Field::IncompatibleFeatures => 8,
            Qcow2Field::CompatibleFeatures => 8,
            Qcow2Field::AutoclearFeatures => 8,
            Qcow2Field::RefcountOrder => 4,
            Qcow2Field::HeaderLength => 4,
        }
    }

    pub fn read_header(&self, file: &mut std::fs::File) -> io::Result<u64> {
        let offset = *self as u64;
        let mut buf = vec![0u8; self.size()];
        file.read_exact_at(&mut buf, offset)?;

        let res = match buf.len() {
            4 => u32::from_be_bytes(buf.try_into().unwrap()) as u64,
            8 => u64::from_be_bytes(buf.try_into().unwrap()),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unreachable")),
        };

        Ok(res)
    }
}
