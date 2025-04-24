const std = @import("std");

pub fn build(b: *std.Build) void {
    // Now what to do with our Build struct:
    // https://ziglang.org/documentation/master/std/#std.Build

    const m = b.createModule(.{
        .target = b.standardTargetOptions(.{}),
        .root_source_file = b.path("bin/rot13.zig"),
    });

    const compile_step = b.addExecutable(.{
        .name = "rot13.exe",
        .root_module = m,
    });

    b.installArtifact(compile_step);
}
