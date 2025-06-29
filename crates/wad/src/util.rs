use anyhow::Result;

pub fn bytes_to_i32(bytes: &[u8]) -> Result<i32> {
    let bytes: &[u8; 4] = bytes.try_into()?;
    Ok(i32::from_le_bytes(*bytes))
}

pub fn bytes_to_i16(bytes: &[u8]) -> Result<i16> {
    let bytes: &[u8; 2] = bytes.try_into()?;
    Ok(i16::from_le_bytes(*bytes))
}

pub fn bytes_to_str(bytes: &[u8]) -> Result<&str> {
    let string = std::str::from_utf8(bytes)?;
    Ok(string.trim_matches('\0'))
}
