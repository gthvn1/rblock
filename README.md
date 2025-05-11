# rblock

## Goals and ideas

- Understanding how QCOW2 stores data (L2/L1 tables, refcounting, backing file chains).
- Create a QCOW2 file from a block device.
- Efficiently scanning two block devices, detecting changed sectors.
- Writing a valid QCOW2 delta image with proper headers and structure.
- To also play with JSON RPC maybe we can write a server for qcow2 files that accepts JSON RPC method like
```json
{ "jsonrpc": "2.0", "method": "read", "params": { "offset": 4096, "length": 512 }, "id": 1 }
```
- or also control command like
```json
{ "method": "get_version", "id": 42 }
```
- Another idea is to create a NBD server to provide a [NetworkBlockDevice](https://github.com/NetworkBlockDevice/nbd/blob/master/doc/proto.md) and
in the same time have a socket to serve JSON RPC.
- To connect the qcow file to a block device we could then load nbd kernel module and use
nbd client to connect our nbd server to the block device. It should looks like:
```sh
sudo modprobe nbd
sudo nbd-client localhost 10809 /dev/nbd0 
```

## Links

- [nbd protocol](https://github.com/NetworkBlockDevice/nbd/blob/master/doc/proto.md)
- [qcow2 spec](https://github.com/qemu/qemu/blob/master/docs/interop/qcow2.txt)
- [JSON-RPC spec](https://www.jsonrpc.org/specification)

## Tips

- to write on the third guest cluster a hello world string you can mount the disk using `qemu-img` and then `dd`:
```sh
sudo qemu-nbd --connect=/dev/nbd0 disk.qcow2
printf "Hello, world!" | sudo dd of=/dev/nbd0 bs=1 seek=$((3 * 65536)) conv=notrunc
# disconnect and check
sudo qemu-nbd --disconnect /dev/nbd0
sudo hexdump -C -n 64 /dev/nbd0 --skip=$((3 * 65536))
# To check the relation between host and guest cluster:
qemu-img map disk.qcow2
```

## Status

- Currently we are running a NBD server that do the handshake (nothing more)
  - Next enter into transmission mode...
- We are also running a JSON-RPC server and you can do:
```
echo -n '{ \
  "jsonrpc": "2.0", \
  "method": "read_guest_cluster", \
  "params": {"cluster": 2}, \
  "id": 1 }' | nc localhost 1234 | jq
```

## Notes

### Mapping Guest Cluster

- We are considering that cluster are 64K (default) as we don't support another size.
- L1 table address are 8 bytes and cluster aligned. That means with one cluster we have 8129 addresses
- L2 table address are also 8 bytes. So 8192 addresses
- Let `guest_cluster` the number of the cluster where you read or write.
  - For example if you want to read 10 bytes at offset 67000 it will be a read from guest cluster 1
- **L1 Index** = `guest_cluster` / 8192
- **L2 Index** = `guest_cluster` % 8192
- For the refcount table:
  - each refcount is 16 bits (default) = 2 bytes
  - that means on one cluster we can hold 32768 refcounts
  - **refcount_table_index** = `host_cluster` / 32768
  - **refcount_block_index** = `host_cluster` % 32768

```
           Guest Cluster Address
                   ↓
       guest_cluster = floor(guest_addr / cluster_size)

┌───────────────────────────────────────────┐
│               L1 Table                    │
├─────────────┬─────────────────────────────┤
│ L1 Index 0  │ → Pointer to L2 Table #0    │ ← guest_cluster in 0..8191
│ L1 Index 1  │ → Pointer to L2 Table #1    │ ← guest_cluster in 8192..16383
│ ...         │                             │
└─────────────┴─────────────────────────────┘
                         ↓
             L2 Index = guest_cluster % 8192
                         ↓
        ┌─────────────────────────────────┐
        │         L2 Table (64KiB)        │
        ├──────────────┬──────────────────┤
        │ L2 Index N   │ → Host Cluster   │
        └──────────────┴──────────────────┘
                         ↓
            Read/Write from Host Cluster

┌──────────────────────────────┐
│         Refcount Table       │
├──────────────────────────────┤
│ Entry for Refcount Block #X  │ → Refcount Block Offset
└──────────────────────────────┘
                         ↓
        Refcount Block (64 KiB, 32768 entries of 2 bytes)
        [host_cluster] ⇒ entry = refcount_table[X] + host_cluster % 32768 × 2

```
- Example:
  - we want to read at guest 0x3000 (12288)
  - `L1 Index = 12288 / 8192 = 1`
  - `L2 Index = 12288 % 8192 = 4096`
  - let's say that at L1 index 1 -> L2 index 4096 we have the host address 0x50000 (327680)
  - `refcount_table_index = 327680 / 32768 = 10`
  - `refcount_table_offset = 327680 % 32768 = 0`
  - so refcount value will be read at `refcount_table[10] + 2 * 0`
  
## Steps

### Explore and play

In progess...
- [x] Detect QCOW2 magic.
- [x] Parse version, backing file name, cluster size.
- [x] Implement a minimal server with JSON RPC method for inspection...
- [x] access L1/L2 table.
- [ ] return data from a given guest cluster
  - [ ] using JSON-RPC API
  - [ ] using NBD server

### Create a qcow2 file from RAW block device

TODO: Creating a QCOW2 image from a single raw block device (with no backing file)

- Create QCOW2 header
  - [ ] magic = 0x514649fb (QFI\xfb)
  - [ ] version = 2 or 3
  - [ ] cluster size (e.g., 65536 = 64 KiB)
  - [ ] refcount table offset
  - [ ] L1 table offset
  - [ ] total virtual size = size of the raw block device

- Compute where each part starts, cluster-align offsets, and write the actual cluster addresses into the L1/L2 tables.
  - [ ] Header
  - [ ] Refcount table & blocks
  - [ ] L1 table
  - [ ] L2 tables
  - [ ] Data clusters (your raw blocks)

- Read raw device and write clusters. Open the raw device and:
  - [ ] For each 64K chunk (or whatever your cluster size is), write it to the QCOW2 file.
  - [ ] Record the guest-to-host mapping in L2 tables.
  - [ ] Write refcounts for each allocated cluster (even metadata!).

- Finish metadata
  - [ ] Backfill the L1 and refcount tables now that you know the cluster locations.
  - [ ] Sync to disk and optionally validate with qemu-img check or info.

### Diff raw disks

TODO: Scan two raw devices and output the sectors that differ.

- [ ] Treat them as large files (using File + seek/read_exact), compare sector-by-sector.
- [ ] Create rb-diff binary or subcommand.
- [ ] Print sector numbers that differ.
- [ ] Count how many sectors changed.

### Write a minimal qcow2 delta

TODO: Write only changed blocks to a new QCOW2 file with a backing file path.

- [ ] Learn how QCOW2 points to backing files.
- [ ] Write L1/L2 tables and changed clusters.
- [ ] Respect cluster alignment and metadata rules.
- [ ] Generate QCOW2 with only a header and one data cluster.
- [ ] Use qemu-img info to validate.

- Extra steps: Boot a Linux VM with the image as overlay!

###  Polish

TODO
