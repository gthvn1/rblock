# rblock

- understanding block devices and image formats
  - read/write block device and play with image formats (VHD, Qcow2, ...)

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
