use std::{
    collections::HashMap,
    env,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde_json::{json, Value};

use chrono::{format::format, Local};
use clap::{Parser, Subcommand};
use colored::*;
use emulator::packet::{PathPacket, PosKey, WardSpawnPacket};
use fern::*;
use log::{info, LevelFilter};
use rayon::prelude::*;

mod emulator;
mod parser;

use crate::emulator::{config::Config, stub_emulator::StubEmulator};
use crate::parser::{metadata::Metadata, parser::get_blocks_with_id, util::read_file};

const BATCH_SIZE: usize = 100;

fn setup_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            let level_color = match record.level() {
                log::Level::Error => "ERROR".red(),
                log::Level::Warn => "WARN".yellow(),
                log::Level::Info => "INFO".green(),
                log::Level::Debug => "DEBUG".blue(),
                log::Level::Trace => "TRACE".purple(),
            };

            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%H:%M:%S"),
                level_color,
                message
            ));
        })
        .level(LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    parsing: Parsing,
}

#[derive(Debug, Subcommand)]
enum Parsing {
    Folder {
        #[clap(short, long, help = "Path to folder with replays")]
        replay_folder: String,
        #[clap(short, long, help = "Path to output folder")]
        output_folder: String,
        #[clap(short, long, help = "Path to to patch file")]
        patch_version: String,
    },
    File {
        #[clap(short, long)]
        replay_file: String,
        #[clap(short, long)]
        output_file: String,
    },
}

fn get_replay_info(file: Vec<u8>, metadata: &Metadata, config: &Config) -> Value {
    let mut game = json!({
        "metadata": metadata.clone(),
        "wards": [],
        "players_state": [],
    });

    let ward_spawn_packets = get_blocks_with_id(&file, config.ward_spawn_decrypt.netid as u16)
        .par_iter()
        .chunks(BATCH_SIZE)
        .map(|payload_chunk| {
            let mut emu = StubEmulator::new(config.clone());

            emu.setup().unwrap();

            payload_chunk
                .into_iter()
                .map(|(timestamp, payload)| {
                    emu.setup_args(payload).unwrap();
                    let packet = emu
                        .call_decrypt_ward_spawn_packet(
                            config.ward_spawn_decrypt.rva,
                            config.ward_spawn_decrypt.end_rva,
                            *timestamp,
                        )
                        .unwrap();
                    emu.reset();
                    packet
                })
                .collect::<Vec<WardSpawnPacket>>()
        })
        .flatten()
        .collect::<Vec<WardSpawnPacket>>();

    let mut placed_wards_map: HashMap<u32, WardSpawnPacket> = HashMap::new();
    let mut pos_id_map: HashMap<PosKey, u32> = HashMap::new();
    for packet in ward_spawn_packets {
        if packet.name.eq("YellowTrinket")
            || packet.name.eq("SightWard")
            || packet.name.eq("JammerDevice")
        {
            placed_wards_map.entry(packet.id).or_insert(packet.clone());
            pos_id_map
                .entry(PosKey::new(packet.x, packet.y))
                .or_insert(packet.id);
        } else if packet.name.contains("Corpse") {
            if let Some((_, id)) = pos_id_map.remove_entry(&PosKey::new(packet.x, packet.y)) {
                if let Some((_, p)) = placed_wards_map.remove_entry(&id) {
                    let owner_player =
                        metadata.get_player_from_id(p.owner_id, config.player_id_start);
                    game["wards"].as_array_mut().unwrap().push(json!({
                        "name": p.name,
                        "team": owner_player.team, 
                        "owner" : json!({ "name": owner_player.name, "team": owner_player.team, "role": owner_player.position}),
                        "timestamp": p.timestamp,
                        "duration": packet.timestamp - p.timestamp,
                        "pos" : [p.x, p.y],
                    }));
                }
            }
        }
    }

    let mut path_packets = get_blocks_with_id(&file, config.mov_decrypt.netid as u16)
        .par_chunks(BATCH_SIZE)
        .map(|payload_chunk| {
            let mut emu = StubEmulator::new(config.clone());

            emu.setup().unwrap();

            payload_chunk
                .iter()
                .filter_map(|(timestamp, payload)| {
                    emu.setup_args(payload).unwrap();
                    let packet = emu.call_decrypt_pos_packet(
                        config.mov_decrypt.rva,
                        config.mov_decrypt.end_rva,
                        *timestamp,
                    );
                    emu.reset();
                    packet.ok()
                })
                .collect::<Vec<PathPacket>>()
        })
        .flatten()
        .collect::<Vec<PathPacket>>();

    path_packets.sort_by(|p1, p2| {
        p1.timestamp
            .partial_cmp(&p2.timestamp)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut players_path_state: HashMap<u32, PathPacket> = HashMap::new();
    let mut timestamp = 0.0;
    for packet in path_packets {
        if packet.id >= config.player_id_start && packet.id <= config.player_id_start + 9 {
            players_path_state.insert(packet.id, packet.clone());
        }

        if packet.timestamp - timestamp >= 1.0 {
            let mut state = json!({
                "timestamp": timestamp,
                "players": json!([]),
            });

            for (_, path) in players_path_state.iter() {
                let (x, y) = path.get_pos(packet.timestamp);
                let player = metadata.get_player_from_id(path.id, config.player_id_start);
                state["players"].as_array_mut().unwrap().push(json!({
                    "role": player.position,
                    "team": player.team,
                    "name": player.name,
                    "champ": player.skin,
                    "pos": [x, y],
                }));
            }
            timestamp = packet.timestamp;

            game["players_state"].as_array_mut().unwrap().push(state);
        }
    }

    game
}

fn parse_batch(replay_folder: String, output_folder: String, patch_version: String) {
    let start = std::time::Instant::now();

    let config = Config::parse(&get_appropiate_patch(patch_version));

    let files: Vec<_> = std::fs::read_dir(replay_folder)
        .unwrap()
        .map(|f| f.ok())
        .collect();

    let file_count = files.len();
    let i = Arc::new(Mutex::new(1));

    files.into_par_iter().for_each(|file| {
        let replay_path = file.as_ref().unwrap().path().display().to_string();
        let name = file
            .as_ref()
            .unwrap()
            .path()
            .file_name()
            .to_owned()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let file = read_file(replay_path.clone());
        let metadata = Metadata::parse(&file);

        let game = get_replay_info(file, &metadata, &config);

        let json_path = PathBuf::from(output_folder.clone()).join(name + ".json");
        let mut json = File::create(json_path).unwrap();
        json.write_all(game.to_string().as_bytes()).unwrap();

        let mut i = i.lock().unwrap();
        info!(
            "[{}/{}] - Processed file '{}'.",
            *i, file_count, replay_path,
        );
        *i += 1;
    });

    let end = start.elapsed().as_secs_f32();
    info!("Total execution time: {:.3}", end);
}

fn get_appropiate_patch(version: String) -> PathBuf {
    let mut patch_file_name = version.replace(".", "-");
    patch_file_name.pop();
    let patch_file = format!("./patch/{}.patch", patch_file_name);

    let patch_path = Path::new(&patch_file);

    if !patch_path.exists() {
        panic!("Patch file not found for version: {}", version);
    }

    info!("Loaded patch config from: {}.patch", patch_file_name);

    patch_path.to_path_buf()
}

fn parse_file(replay_file: String, output_file: String) {
    let start = std::time::Instant::now();

    let file = read_file(replay_file.clone());
    let metadata = Metadata::parse(&file);
    let config = Config::parse(&get_appropiate_patch(metadata.version.clone()));

    let game = get_replay_info(file, &metadata, &config);

    let json_path = PathBuf::from(output_file.clone());
    let mut json = File::create(json_path).unwrap();
    json.write_all(game.to_string().as_bytes()).unwrap();

    let end = start.elapsed().as_secs_f32();
    info!("Output: {}, Total execution time: {:.3}", output_file, end);
}

fn set_cwd() {
    match env::current_exe() {
        Ok(path) => {
            if let Err(e) = env::set_current_dir(&path.parent().unwrap()) {
                panic!("{}: {}", e, path.display());
            }
        }
        Err(e) => {
            panic!("Failed to get current executable path: {}", e);
        }
    };
}

fn main() {
    set_cwd();

    setup_logger().unwrap();

    let args = Cli::parse();

    match args.parsing {
        Parsing::File {
            replay_file,
            output_file,
        } => {
            parse_file(replay_file, output_file);
        }
        Parsing::Folder {
            replay_folder,
            output_folder,
            patch_version,
        } => parse_batch(replay_folder, output_folder, patch_version),
        _ => unimplemented!("Batch parsing not implemented yet"),
    }
}
