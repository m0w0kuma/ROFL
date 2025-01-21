use serde_json::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub skin: String,
    pub team: String,
    pub position: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub version: String,
    pub game_len: u64,
    pub winning_team: String,
    pub players: Vec<Player>,
}

impl Metadata {
    pub fn parse(buffer: &[u8]) -> Self {
        let version = String::from_utf8(buffer[16..20].to_vec()).unwrap();

        let len = buffer.len();

        let raw_json_medata_size = &buffer[len - 4..];
        let json_metadata_size = u32::from_le_bytes(raw_json_medata_size.try_into().unwrap());

        let raw_medata_json = &buffer[len - 4 - json_metadata_size as usize..len - 4];
        let metadata_json = String::from_utf8(raw_medata_json.to_vec()).unwrap();

        let json: Value = serde_json::from_str(&metadata_json).unwrap();

        let game_len: u64 = json.get("gameLength").unwrap().as_i64().unwrap() as u64;
        let stats_json: Value = serde_json::from_str(json["statsJson"].as_str().unwrap()).unwrap();

        let players: Vec<_> = stats_json
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, player)| {
                let name = player["NAME"].as_str().unwrap().to_string();
                let skin = player["SKIN"].as_str().unwrap().to_string();
                let team = match player["TEAM"].as_str().unwrap() {
                    "100" => "Blue".to_string(),
                    "200" => "Red".to_string(),
                    _ => panic!("Invalid team"),
                };

                let position = vec!["Top", "Jungle", "Mid", "Adc", "Support"][i % 5].to_string();

                Player {
                    name,
                    skin,
                    team,
                    position,
                }
            })
            .collect();

        let first_entry = stats_json.as_array().unwrap()[0].as_object().unwrap();
        let winning_team = match (
            first_entry.get("TEAM").unwrap().as_str().unwrap(),
            first_entry.get("WIN").unwrap().as_str().unwrap(),
        ) {
            ("100", "Win") => "Blue".to_string(),
            _ => "Red".to_string(),
        };

        Metadata {
            version,
            game_len,
            winning_team,
            players,
        }
    }
}
