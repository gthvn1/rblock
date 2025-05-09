# rblock

## Goals and ideas

- Understanding how QCOW2 stores differences (L2/L1 tables, refcounting, backing file chains).
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


## Steps

### Explore and play

- https://github.com/qemu/qemu/blob/master/docs/interop/qcow2.txt

In progess...
- [x] Detect QCOW2 magic.
- [ ] Parse version, backing file name, cluster size.
- [ ] Print L1/L2 table layout (even without reading them yet).
- [ ] Add a `--inspect` option
- [ ] Implement a minimal server with JSON RPC method for inspection...

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
