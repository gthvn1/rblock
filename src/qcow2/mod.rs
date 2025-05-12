mod header;

use log::{debug, error};
use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;

use header::Qcow2Field;

// Only keep fields that are not modified
pub struct Qcow2 {
    file: File,
    version: u64,
}

impl Qcow2 {
    pub fn new(fname: &str) -> io::Result<Self> {
        let mut file = File::open(fname)?;

        const EXPECTED_MAGIC: u64 = 0x514649fb;
        let magic = Qcow2Field::read_header(&Qcow2Field::Magic, &mut file)?;

        if magic != EXPECTED_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic number {:?}", magic),
            ));
        }

        let version = Qcow2Field::read_header(&Qcow2Field::Version, &mut file)?;

        // Sanity check
        // As we don't understand any bits of incompatible features we
        // just fail as soon as one is set.
        let incompatible_features =
            Qcow2Field::read_header(&Qcow2Field::IncompatibleFeatures, &mut file)?;

        if incompatible_features != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!(
                    "Unkown bit set in incompatible_features 0x{:08x}",
                    incompatible_features
                ),
            ));
        }

        // Sanity check for v3 only
        if version == 3 {
            let header_length = Qcow2Field::read_header(&Qcow2Field::HeaderLength, &mut file)?;

            // Sanity check
            if header_length < 104 || header_length % 8 != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid header length {}", header_length),
                ));
            }
        }

        // Print some information before returning
        let mut q = Qcow2 { file, version };

        debug!("== Qcow2 header ==");
        debug!("  header length          : {}", q.header_len());
        debug!("  backing file           : {:?}", q.backing_file());
        debug!("  cluster size           : {}", q.cluster_size());
        debug!("  virtual size           : {}", q.virtual_size());
        debug!("  L1 size                : {}", q.l1_size());
        debug!("  L1 table offset        : 0x{:08x}", q.l1_table_offset());
        debug!("  refcount width         : {}", q.refcount_width());
        debug!(
            "  refcount table offset  : 0x{:08x}",
            q.refcount_table_offset()
        );
        debug!("  refcount table entries : {}", q.refcount_table_clusters());

        if version == 3 {
            debug!("  = Version 3 only");
            debug!(
                "  compatible features are ignored: 0x{:08x}",
                q.compatible_features()
            );
            debug!(
                "  autoclear features are ignored: 0x{:08x}",
                q.autoclear_features()
            );
        }

        // And dump L1 entries
        // TODO: Add RPC to do
        let _ = q.get_l1_entries();

        Ok(q)
    }

    fn read_feature(&mut self, field: Qcow2Field, name: &str) -> u64 {
        Qcow2Field::read_header(&field, &mut self.file).unwrap_or_else(|_| {
            error!("failed to read {}", name);
            0
        })
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn backing_file(&mut self) -> Option<String> {
        let offset = self.read_feature(Qcow2Field::BackingFileOffset, "backing file offset");
        let sz = self.read_feature(Qcow2Field::BackingFileSize, "backing file size");

        if sz == 0 {
            return None;
        }

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

    pub fn cluster_size(&mut self) -> usize {
        let cluster_bits = self.read_feature(Qcow2Field::ClusterBits, "cluster bits");
        (1 << cluster_bits) as usize
    }

    pub fn virtual_size(&mut self) -> u64 {
        self.read_feature(Qcow2Field::Size, "size")
    }

    pub fn crypto_method(&mut self) -> u64 {
        self.read_feature(Qcow2Field::CryptMethod, "crypto method")
    }

    pub fn l1_size(&mut self) -> u64 {
        self.read_feature(Qcow2Field::L1Size, "L1 size")
    }

    pub fn l1_table_offset(&mut self) -> u64 {
        self.read_feature(Qcow2Field::L1TableOffset, "L1 table offset")
    }

    pub fn refcount_table_offset(&mut self) -> u64 {
        self.read_feature(Qcow2Field::RefcountTableOffset, "refcount table offset")
    }

    pub fn refcount_table_clusters(&mut self) -> u64 {
        self.read_feature(Qcow2Field::RefcountTableClusters, "refcount table clusters")
    }

    pub fn nb_snapshots(&mut self) -> u64 {
        self.read_feature(Qcow2Field::NbSnapshots, "number of snapshots")
    }

    pub fn snapshots_offset(&mut self) -> u64 {
        self.read_feature(Qcow2Field::SnapshotsOffset, "snapshots offset")
    }

    pub fn incompatible_features(&mut self) -> u64 {
        self.read_feature(Qcow2Field::IncompatibleFeatures, "incompatible features")
    }

    pub fn compatible_features(&mut self) -> u64 {
        self.read_feature(Qcow2Field::CompatibleFeatures, "compatible features")
    }

    pub fn autoclear_features(&mut self) -> u64 {
        self.read_feature(Qcow2Field::AutoclearFeatures, "autoclear features")
    }

    pub fn refcount_width(&mut self) -> u64 {
        let order = self.read_feature(Qcow2Field::RefcountOrder, "refcount order");
        1 << order
    }

    pub fn header_len(&mut self) -> u64 {
        self.read_feature(Qcow2Field::HeaderLength, "header length")
    }

    pub fn get_l1_entries(&mut self) -> Vec<(usize, u64)> {
        let l1_off = self.l1_table_offset();
        let l1_sz = self.l1_size() as usize;

        // l1_sz gives the number of entries used by l1
        debug!("Reading {} bytes at offset 0x{:016x}", 8 * l1_sz, l1_off);
        let mut buf: Vec<u8> = vec![0u8; l1_sz * 8];
        self.file.read_exact_at(&mut buf, l1_off).unwrap();

        let entries: Vec<(usize, u64)> = buf
            .chunks_exact(8)
            .enumerate()
            .map(|(idx, chunk)| (idx, u64::from_be_bytes(chunk.try_into().unwrap())))
            .filter(|&(_, entry)| entry != 0)
            .collect();

        for (idx, entry) in entries.iter() {
            debug!("L1[{}] -> 0x{:016x}", idx, entry);
        }

        entries
    }

    pub fn read_guest_cluster(&mut self, n: u64) -> Vec<u8> {
        // Read the data corresponding to guest cluster N
        let cluster_sz = self.cluster_size();
        let mut data = vec![0u8; cluster_sz];

        debug!("Reading data from guest cluster {}", n);
        let l1_entries_per_cluster = cluster_sz / 8; // L1 entries are 8 bytes and the size of L1 is one cluster
        // l1 entries per cluster is the same as L2 entries because L2 is a cluster size and addresses are 8 bytes.
        let l1_index = n / l1_entries_per_cluster as u64;
        let l2_index = n % l1_entries_per_cluster as u64;

        debug!(
            "Guest cluster {} is at L1[{}] and L2[{}]",
            n, l1_index, l2_index
        );

        let mut bytes: [u8; 8] = [0u8; 8];
        let l1_offset = self.l1_table_offset();

        self.file
            .read_exact_at(&mut bytes, l1_offset + l1_index * 8)
            .unwrap_or_else(|_| {
                panic!("Failed to read L1 entry at 0x{:016x}", l1_offset + l1_index)
            });

        let l1_entry = u64::from_be_bytes(bytes);
        debug!("Read L1 entry 0x{:016x}", l1_entry);

        if l1_entry == 0 {
            // If entry is null it means there is no data so just returns.
            return data;
        }

        let l2_offset = 0x7FFF_FFFF_FFFF_FFFF & l1_entry;
        debug!("Read L1 entry and get L2 offset 0x{:016x}", l2_offset);

        self.file
            .read_exact_at(&mut bytes, l2_offset + l2_index * 8)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to read L1 entry at 0x{:016x}",
                    l2_offset + l2_index * 8
                )
            });

        let l2_entry = u64::from_be_bytes(bytes);
        debug!("Read L2 entry: 0x{:016x}", l2_entry);

        if l2_entry == 0 {
            return data;
        }

        let data_offset = 0x7FFF_FFFF_FFFF_FFFF & l2_entry;
        let n = self
            .file
            .read_at(&mut data, data_offset)
            .expect("Failed to read data for guest cluster");

        debug!("Read {} bytes of data", n);
        data.truncate(n);
        data
    }
}
