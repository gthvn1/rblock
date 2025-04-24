const std = @import("std");

pub fn build(b: *std.Build) void {
    // Now what to do with our Build struct:
    // https://ziglang.org/documentation/master/std/#std.Build

    const m = b.createModule(.{
        .target = b.standardTargetOptions(.{}),
    });

    _ = b.addExecutable(.{
        .name = "hello.exe",
        .root_module = m,
    });
}
