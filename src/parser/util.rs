use std::fs::File;
use std::io::Read;

pub fn parse_f32(chunk: &mut impl Iterator<Item = u8>) -> Result<f32, ()> {
    let bytes: [u8; 4] = chunk
        .take(4)
        .collect::<Vec<u8>>()
        .try_into()
        .map_err(|_| ())?;
    Ok(f32::from_le_bytes(bytes))
}

pub fn parse_u32(chunk: &mut impl Iterator<Item = u8>) -> Result<u32, ()> {
    let bytes: [u8; 4] = chunk
        .take(4)
        .collect::<Vec<u8>>()
        .try_into()
        .map_err(|_| ())?;
    Ok(u32::from_le_bytes(bytes))
}

pub fn parse_u16(chunk: &mut impl Iterator<Item = u8>) -> Result<u16, ()> {
    let bytes: [u8; 2] = chunk
        .take(2)
        .collect::<Vec<u8>>()
        .try_into()
        .map_err(|_| ())?;
    Ok(u16::from_le_bytes(bytes))
}

pub fn parse_u8(chunk: &mut impl Iterator<Item = u8>) -> Result<u8, ()> {
    let bytes: [u8; 1] = chunk
        .take(1)
        .collect::<Vec<u8>>()
        .try_into()
        .map_err(|_| ())?;
    Ok(bytes[0])
}

pub fn bit_test(value: u32, bit: u8) -> bool {
    (value & (1 << bit)) != 0
}

pub fn sign_extend(num: i16, bits: u32) -> i16 {
    let shift = i16::BITS - bits;
    (num << shift) >> shift
}

pub fn read_file(path: String) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

pub fn point_dist(x: (f32, f32), y: (f32, f32)) -> f32 {
    let (x1, y1) = x;
    let (x2, y2) = y;
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}
