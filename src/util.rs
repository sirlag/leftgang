use std::num::ParseIntError;

pub fn parse_channel_id(str: &str) -> Result<u64, ParseIntError> {
    str.parse::<u64>()
        .or_else(|_| str.replace("<#", "").replace(">", "").parse())
}
