pub fn rotate_char(c: u8) u8 {
    return switch (c) {
        'A'...'Z' => 'A' + @mod(c - 'A' + 13, 26),
        'a'...'z' => 'a' + @mod(c - 'a' + 13, 26),
        else => c,
    };
}
