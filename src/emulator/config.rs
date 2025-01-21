use serde_json::*;
use std::{io::Read, path::Path, sync::Arc};

/*
{
    "alloc1": "0xe7bc00",
    "skip": "0xee39a0",
    "ward_spawn_decrypt": {
        "netid": 272,
        "addr": "0xd603e0"
    },
    "text": {
        "rva": "0x1000",
        "size": 22708224
    },
    "data": {
        "rva": "0x195e000",
        "size": 1216512
    },
    "rdata": {
        "rva": "0x15a9000",
        "size": 3887104
    }
}
*/

#[derive(Clone)]
pub struct WardSpawnDecrypt {
    pub netid: u32,

    pub rva: u64,
    pub end_rva: u64,

    pub id_offset: u64,
    pub owner_id_offset: u64,

    pub name_offset: u64,
    pub name_len_offset: u64,

    pub x_offset: u64,
    pub x_write_count: u32,

    pub y_offset: u64,
    pub y_write_count: u32,
}

#[derive(Clone)]
pub struct MovDecrypt {
    pub netid: u32,

    pub rva: u64,
    pub end_rva: u64,

    pub payload_offset: u64,
    pub payload_size_offset: u64,
}

#[derive(Clone)]
pub struct Section {
    pub name: String,
    pub rva: u64,
    pub size: u64,
    pub raw: Vec<u8>,
}

#[derive(Clone)]
pub struct Config {
    pub alloc1: u64,
    pub alloc2: u64,
    pub skip: u64,

    pub ward_spawn_decrypt: WardSpawnDecrypt,
    pub mov_decrypt: MovDecrypt,

    pub base_addr: u64,

    pub player_id_start: u32,

    pub text: Arc<Section>,
    pub data: Arc<Section>,
    pub rdata: Arc<Section>,
}

impl Config {
    pub fn parse(patch_file: &Path) -> Self {
        let zipfile = std::fs::File::open(patch_file).unwrap();
        let mut archive = zip::ZipArchive::new(zipfile).unwrap();

        let json: Value = match archive.by_name("result.json") {
            Ok(mut file) => {
                let mut contents = Vec::new();
                file.read_to_end(&mut contents).unwrap();
                let str_json = String::from_utf8(contents).expect("Invalid UTF-8 in result.json");
                serde_json::from_str(&str_json).unwrap()
            }
            Err(e) => {
                panic!("Error reading result.json: {}", e);
            }
        };

        let raw_text = match archive.by_name("text.bin") {
            Ok(mut file) => {
                let mut content = Vec::new();
                file.read_to_end(&mut content)
                    .expect("Failed to read file contents");
                content
            }
            Err(..) => {
                panic!("Error reading text.bin");
            }
        };

        let raw_data = match archive.by_name("data.bin") {
            Ok(mut file) => {
                let mut content = Vec::new();
                file.read_to_end(&mut content).unwrap();
                content
            }
            Err(..) => {
                panic!("Error reading text.bin");
            }
        };

        let raw_rdata = match archive.by_name("rdata.bin") {
            Ok(mut file) => {
                let mut content = Vec::new();
                file.read_to_end(&mut content)
                    .expect("Failed to read file contents");
                content
            }
            Err(..) => {
                panic!("Error reading text.bin");
            }
        };

        Self {
            alloc1: Self::str_hex_to_u64(json["alloc1_rva"].as_str().unwrap()),
            alloc2: Self::str_hex_to_u64(json["alloc2_rva"].as_str().unwrap()),
            skip: Self::str_hex_to_u64(json["skip_rva"].as_str().unwrap()),
            ward_spawn_decrypt: WardSpawnDecrypt {
                netid: json["ward_spawn_decrypt"]["netid"].as_u64().unwrap() as u32,
                rva: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["rva_start"].as_str().unwrap(),
                ),
                end_rva: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["rva_end"].as_str().unwrap(),
                ),
                id_offset: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["id_offset"].as_str().unwrap(),
                ),
                owner_id_offset: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["owner_id_offset"]
                        .as_str()
                        .unwrap(),
                ),
                name_offset: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["name_offset"].as_str().unwrap(),
                ),
                name_len_offset: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["name_len_offset"]
                        .as_str()
                        .unwrap(),
                ),
                x_offset: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["x_offset"].as_str().unwrap(),
                ),
                x_write_count: Self::str_hex_to_u32(
                    json["ward_spawn_decrypt"]["x_write_count"]
                        .as_str()
                        .unwrap(),
                ),
                y_offset: Self::str_hex_to_u64(
                    json["ward_spawn_decrypt"]["y_offset"].as_str().unwrap(),
                ),
                y_write_count: Self::str_hex_to_u32(
                    json["ward_spawn_decrypt"]["y_write_count"]
                        .as_str()
                        .unwrap(),
                ),
            },
            mov_decrypt: MovDecrypt {
                netid: json["mov_decrypt"]["netid"].as_u64().unwrap() as u32,
                rva: Self::str_hex_to_u64(json["mov_decrypt"]["rva_start"].as_str().unwrap()),
                end_rva: Self::str_hex_to_u64(json["mov_decrypt"]["rva_end"].as_str().unwrap()),
                payload_offset: Self::str_hex_to_u64(
                    json["mov_decrypt"]["payload_offset"].as_str().unwrap(),
                ),
                payload_size_offset: Self::str_hex_to_u64(
                    json["mov_decrypt"]["payload_size_offset"].as_str().unwrap(),
                ),
            },
            base_addr: 0x7ff76afd0000,
            player_id_start: Self::str_hex_to_u32(json["player_id_start"].as_str().unwrap()),
            text: Section {
                name: "text".to_string(),
                rva: Self::str_hex_to_u64(json["text"]["rva"].as_str().unwrap()),
                size: json["text"]["size"].as_u64().unwrap(),
                raw: raw_text,
            }
            .into(),
            data: Section {
                name: "data".to_string(),
                rva: Self::str_hex_to_u64(json["data"]["rva"].as_str().unwrap()),
                size: json["data"]["size"].as_u64().unwrap(),
                raw: raw_data,
            }
            .into(),
            rdata: Section {
                name: "rdata".to_string(),
                rva: Self::str_hex_to_u64(json["rdata"]["rva"].as_str().unwrap()),
                size: json["rdata"]["size"].as_u64().unwrap(),
                raw: raw_rdata,
            }
            .into(),
        }
    }

    pub fn str_hex_to_u64(str: &str) -> u64 {
        u64::from_str_radix(str.trim_start_matches("0x"), 16).unwrap()
    }

    pub fn str_hex_to_u32(str: &str) -> u32 {
        u32::from_str_radix(str.trim_start_matches("0x"), 16).unwrap()
    }
}
