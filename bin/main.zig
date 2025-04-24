const std = @import("std");
const rot13 = @import("rot13.zig");

const ifname = "samples/input.txt";
const ofname = "samples/output.txt";

pub fn main() !void {
    // Let's read a file and transform it with rot13
    std.log.info("reading {s}", .{ifname});

    std.log.info("writing {s}", .{ofname});

    // get current dir
    const cd = std.fs.cwd();

    const ifile = try std.fs.Dir.openFile(cd, ifname, .{});
    defer ifile.close();

    const f = try std.fs.Dir.createFile(std.fs.cwd(), ofname, .{ .truncate = true });
    defer f.close();

    var carlu: [1]u8 = undefined;

    while (true) {
        const read = try ifile.read(&carlu);
        if (read == 0) break;

        const c = rot13.rotate_char(carlu[0]);
        _ = try f.write(&[1]u8{c});
    }
}
