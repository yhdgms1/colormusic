pub fn create_color_command(red_value: u8, green_value: u8, blue_value: u8) -> [u8; 9] {
    [
        0x7E,
        0x07,
        0x05,
        0x03,
        red_value,
        green_value,
        blue_value,
        0x10,
        0xEF,
    ]
}
