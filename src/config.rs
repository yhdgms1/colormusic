use std::fs;

pub fn get_devices_list() -> Option<Vec<String>> {
    if let Ok(content) = fs::read_to_string("./devices.txt") {
        let devices: Vec<String> = content
            .split_terminator(&['\n', '\r'][..])
            .map(|v| String::from(v).trim().to_string())
            .collect();

        return Some(devices);
    }

    None
}
