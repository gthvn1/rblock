const std = @import("std");

const fname = "output.txt";

pub fn main() !void {
    // Let's write Hello, Sailor into a file
    std.log.info("writing {s}", .{fname});

    const f = try std.fs.Dir.createFile(std.fs.cwd(), fname, .{ .truncate = true });
    defer f.close();

    try f.writeAll("Hello, Sailor!");
}
