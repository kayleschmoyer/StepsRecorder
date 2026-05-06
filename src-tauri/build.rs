use std::fs;
use std::path::Path;

const ICON_SIZE: usize = 32;
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

fn main() {
    ensure_generated_icon();
    tauri_build::build();
}

fn ensure_generated_icon() {
    let icon_path = Path::new("icons/icon.png");

    if icon_path.exists() {
        return;
    }

    fs::create_dir_all("icons").expect("failed to create Tauri icons directory");
    fs::write(icon_path, create_icon_png()).expect("failed to write generated Tauri icon");
}

fn create_icon_png() -> Vec<u8> {
    let raw_image = create_raw_rgba_rows();
    let compressed_image = create_zlib_stored_block(&raw_image);

    let mut png = Vec::new();
    png.extend_from_slice(PNG_SIGNATURE);
    append_chunk(&mut png, b"IHDR", &create_ihdr());
    append_chunk(&mut png, b"IDAT", &compressed_image);
    append_chunk(&mut png, b"IEND", &[]);
    png
}

fn create_ihdr() -> Vec<u8> {
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(ICON_SIZE as u32).to_be_bytes());
    ihdr.extend_from_slice(&(ICON_SIZE as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    ihdr
}

fn create_raw_rgba_rows() -> Vec<u8> {
    let row_length = 1 + ICON_SIZE * 4;
    let mut rows = Vec::with_capacity(row_length * ICON_SIZE);

    for y in 0..ICON_SIZE {
        rows.push(0);

        for x in 0..ICON_SIZE {
            let is_mark = x.abs_diff(y) < 4 || x + y == ICON_SIZE - 1;
            let pixel = if is_mark {
                [204, 145, 102, 255]
            } else {
                [8, 8, 10, 255]
            };

            rows.extend_from_slice(&pixel);
        }
    }

    rows
}

fn create_zlib_stored_block(data: &[u8]) -> Vec<u8> {
    let mut zlib = Vec::with_capacity(data.len() + 11);
    zlib.extend_from_slice(&[0x78, 0x01]);

    let length = u16::try_from(data.len()).expect("generated icon data is too large");
    zlib.push(0x01);
    zlib.extend_from_slice(&length.to_le_bytes());
    zlib.extend_from_slice(&(!length).to_le_bytes());
    zlib.extend_from_slice(data);
    zlib.extend_from_slice(&adler32(data).to_be_bytes());
    zlib
}

fn append_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(chunk_type);
    png.extend_from_slice(data);

    let mut crc_input = Vec::with_capacity(chunk_type.len() + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    png.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;

    let mut a = 1_u32;
    let mut b = 0_u32;

    for byte in data {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;

    for byte in data {
        crc ^= u32::from(*byte);

        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }

    !crc
}
