# zblock

- understanding block devices and image formats
  - first it is to learn Zig
  - then read/write block device and play with image formats (VHD, Qcow2, ...)

## Changelog

- **24-04-2025**
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
