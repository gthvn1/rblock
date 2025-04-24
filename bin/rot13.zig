const std = @import("std");

const ifname = "samples/input.txt";
const ofname = "samples/output.txt";

fn rot13_of_char(c: u8) u8 {
    return switch (c) {
        'A'...'Z' => 'A' + @mod(c - 'A' + 13, 26),
        'a'...'z' => 'a' + @mod(c - 'a' + 13, 26),
        else => c,
    };
}

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

        const c = rot13_of_char(carlu[0]);
        _ = try f.write(&[1]u8{c});
    }
}
