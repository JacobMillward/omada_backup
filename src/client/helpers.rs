pub fn parse_file_name_from_content_disposition(header_value: Option<&str>) -> Option<String> {
    if let Some(value) = header_value {
        if value.is_empty() {
            return None;
        }

        let value_str = percent_encoding::percent_decode_str(value)
            .decode_utf8()
            .unwrap();
        let index = value_str.find("fileName=").map(|i| i + 9);

        if let Some(start) = index {
            return Some(value_str[start..].to_owned());
        }
    }

    None
}
