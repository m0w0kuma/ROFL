use crate::parser::util::*;

use anyhow::Result;

/* 
pub fn get_ward_team_from_id(id: u32, player_id_start: u32) -> String {
    // FIXME: should not be hardcoded. should be on config file
    if id < player_id_start + 5 && id >= player_id_start {
        "Blue".to_string()
    } else {
        "Red".to_string()
    }
}

pub fn get_ward_role_from_id(id: u32, player_id_start: u32) -> String {
    // FIXME: should not be hardcoded. should be on config file
    if id == player_id_start || id == player_id_start + 5 {
        return "Top".to_string();
    } else if id == player_id_start + 1 || id == player_id_start + 6 {
        return "Jungle".to_string();
    } else if id == player_id_start + 2 || id == player_id_start + 7 {
        return "Mid".to_string();
    } else if id == player_id_start + 3 || id == player_id_start + 8 {
        return "Adc".to_string();
    } else if id == player_id_start + 4 || id == player_id_start + 9 {
        return "Support".to_string();
    }

    unreachable!("Invalid id: {}", id);
}
*/

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct PosKey {
    pub x: i32,
    pub y: i32,
}

impl PosKey {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct WardSpawnPacket {
    pub timestamp: f32,
    pub name: String,
    pub id: u32,
    pub owner_id: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub struct PathPacket {
    pub timestamp: f32,
    pub id: u32,
    pub speed: f32,
    pub waypoints: Vec<(f32, f32)>,
}

impl PathPacket {
    pub fn parse(timestamp: f32, payload: Vec<u8>) -> Result<Self> {
        let mut payload_iter = payload.into_iter();

        let parsing_type = parse_u16(&mut payload_iter).unwrap();
        let ent_id = parse_u32(&mut payload_iter).unwrap();
        let ent_speed = parse_f32(&mut payload_iter).unwrap();

        if (parsing_type as u8 & 1) != 0 {
            payload_iter.next().unwrap();
        }

        let temp_arr = payload_iter.clone().collect::<Vec<u8>>();

        let mut encoded_coords: Vec<u16> = vec![];

        let unk = (parsing_type as u8 >> 1) as u32;
        if unk == 0 {
            return Err(anyhow::anyhow!("Invalid parsing type"));
        } else if unk > 1 {
            let unk2 = ((unk - 2) >> 2) + 1;
            payload_iter.nth(unk2 as usize - 1);
        }

        let mut v10 = 0;
        let mut v13 = 0;
        let mut y_coord: u16 = 0;
        let mut x_coord: u16 = 0;
        loop {
            let mut v14 = 2;
            let mut v15 = 2;
            if v10 != 0 {
                let mut v16 = v13;
                let mut v17: i8 = v13 & 7;
                if v13 < 0 {
                    v16 = v13 + 7;
                    v17 -= 8;
                }
                let v18 = temp_arr[v16 as usize >> 3];
                let mut v19 = v13 + 1;
                let v20 = -((1 << v17) & v18 as i8);
                let mut v21 = (v13 + 1) & 7;
                v14 = 2 - (v20 != 0) as i8;
                if v19 < 0 {
                    v19 = v13 + 8;
                    v21 -= 8;
                }
                v15 = 2 - (((1 << v21) & temp_arr[v19 as usize >> 3]) != 0) as i8;
                v13 += 2;
            }

            if v14 == 1 {
                x_coord = x_coord.wrapping_add(payload_iter.next().unwrap() as u16);
            } else {
                x_coord = parse_u16(&mut payload_iter).unwrap();
            }

            if v15 == 1 {
                y_coord = y_coord.wrapping_add(payload_iter.next().unwrap() as u16);
            } else {
                y_coord = parse_u16(&mut payload_iter).unwrap();
            }

            encoded_coords.push(x_coord);
            encoded_coords.push(y_coord);

            v10 += 1;
            if v10 >= unk {
                break;
            }
        }

        let mut path: Vec<(f32, f32)> = vec![];
        if !encoded_coords.is_empty() {
            let mut i = 0;
            for _ in 0..encoded_coords.len() / 2 {
                let x = ((sign_extend(encoded_coords[i] as i16, 16) as f32 * 2.0) + 7358.0) as f32;
                let y =
                    ((sign_extend(encoded_coords[i + 1] as i16, 16) as f32 * 2.0) + 7412.0) as f32;

                path.push((x, y));

                i += 2;
            }
        }

        Ok(Self {
            timestamp,
            id: ent_id,
            speed: ent_speed,
            waypoints: path,
        })
    }

    pub fn get_pos(&self, timestamp: f32) -> (f32, f32) {
        if self.waypoints.is_empty() {
            return (0.0, 0.0);
        }

        if self.waypoints.len() == 1 {
            return *self.waypoints.first().unwrap();
        }

        let delta = timestamp - self.timestamp;

        if delta <= 1.0 {
            return *self.waypoints.first().unwrap();
        }

        let mut remaining_time: f32 = delta;
        for waypoint_pair in self.waypoints.windows(2) {
            let (x1, y1) = waypoint_pair[0];
            let (x2, y2) = waypoint_pair[1];
            let distance = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();

            let dt = distance / self.speed;

            if remaining_time <= dt {
                let t = remaining_time / dt;
                let x = x1 + (x2 - x1) * t;
                let y = y1 + (y2 - y1) * t;
                return (x, y);
            }

            remaining_time -= dt;
        }

        *self.waypoints.last().unwrap()
    }
}
