use anyhow::Result;

pub fn read_string(data: &[u8], offset: usize, length: usize) -> Result<String> {
    Ok(String::from_utf8(data[offset..offset + length].to_vec())?
        .trim_end_matches('\0')
        .to_string())
}

pub fn read_i16(data: &[u8], offset: usize) -> Result<i16> {
    Ok(i16::from_le_bytes(data[offset..offset + 2].try_into()?))
}

pub fn read_i32(data: &[u8], offset: usize) -> Result<i32> {
    Ok(i32::from_le_bytes(data[offset..offset + 4].try_into()?))
}
