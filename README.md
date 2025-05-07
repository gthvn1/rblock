# rblock

## Goals

- Understanding how QCOW2 stores differences (L2/L1 tables, refcounting, backing file chains).
- Efficiently scanning two block devices, detecting changed sectors.
- Writing a valid QCOW2 delta image with proper headers and structure.

## Steps

### Explore and play

- https://github.com/qemu/qemu/blob/master/docs/interop/qcow2.txt

In progess...
- [x] Detect QCOW2 magic.
- [ ] Parse version, backing file name, cluster size.
- [ ] Print L1/L2 table layout (even without reading them yet).
- [ ] Add a `--inspect` option

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

## Changelog

- **25-04-2025**
  - copy after applying rot13 rotation
  - copy input file into output file
  - add `main` function... and it buils successfully
    - now you can do `cargo build && ./target/debug/hello`
  - add missing `src/main.rs`
    - **ERROR**: error[E0601]: `main` function not found in crate `hello`
  - add missing `package.name`
    - See the [Cargo manifest](https://doc.rust-lang.org/cargo/reference/manifest.html)
    - **ERROR**: no targets specified in the manifest
  - add missing `[package]`
    - **ERROR**: missing field `package.name`
  - create an empty `Cargo.toml`
    - **ERROR**: manifest is missing either a `[package]` or a `[workspace]`
  - do the same but using rust
    - start with an empty repo, run `cargo build` and fixes issues reported to reach the hello starting point

- **24-04-2025**
  - start a module rot13
  - read input from `samples/input.txt` and write the `samples/output.txt`
    after transforming using rot13.
  - move source code into `bin/` and renamed file rot13.zig
  - write a string into an output file
  - add the compile step as part of an install step and adds it to the dependencies of the top-level install
    - Now we have the executable: `zig build && ./zig-out/bin/hello.exe`
  - adding *root_source_file* to module but still no executable
  - We use create module to create the *root_module*. It builds but still no executable is generated.
  - In doc target is mentionned as deprecated and prefer *root_module*
  - Adding a name field. Next errors: `@panic("`root_module` and `target` cannot both be null")`
  - In library we found *addExecutable*. Add it without executable options says name is required.
  - adding the missing argument. Next errors: None it builds!!! but nothing is produced :)
  - adding a member named *build*. Next errors: `.../zig-linux-x86_64-0.14.0/lib/std/Build.zig:2427:27: error: expected 0 argument(s), found 1
        .void => build_zig.build(b),`
  - instead of running `zig init` to fix previous issue just create an empty `build.zig`
    and run `zig build` again. It failed with:
    - `root source file struct 'build' has no member named 'build'`
  - run `zig build` in the empty repo and see how it failed:
```
info: initialize build.zig template file with 'zig init'
info: see 'zig --help' for more options
error: no build.zig file found, in the current directory or any parent directories
```
  - create the empty repo
