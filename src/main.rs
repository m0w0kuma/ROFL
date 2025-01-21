use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde_json::{json, Value};

use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use emulator::packet::{get_role_from_id, get_team_from_id, PathPacket, PosKey, WardSpawnPacket};
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
    #[clap(short, long, help = "Path to to patch file")]
    patch_file: String,
}

#[derive(Debug, Subcommand)]
enum Parsing {
    Folder {
        #[clap(short, long, help = "Path to folder with replays")]
        replay_folder: String,
        #[clap(short, long, help = "Path to output folder")]
        output_folder: String,
    },
    File {
        #[clap(short, long)]
        replay_file: String,
        #[clap(short, long)]
        output_file: String,
    },
}

fn get_replay_info(replay_path: String, config: &Config) -> Value {
    let file = read_file(replay_path);
    let metadata = Metadata::parse(&file);

    let mut game = json!({
        "metadata": metadata,
        "wards": [],
        "players": {
            "Red": {
                "Top": [],
                "Jungle": [],
                "Mid": [],
                "Adc": [],
                "Support": []
            },
            "Blue": {
                "Top": [],
                "Jungle": [],
                "Mid": [],
                "Adc": [],
                "Support": []
            }
        },
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
                    game["wards"].as_array_mut().unwrap().push(json!({
                        "name": p.name,
                        "team": get_team_from_id(p.owner_id, config.player_id_start),
                        "owner_role": get_role_from_id(p.owner_id, config.player_id_start),
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
            for (_, path) in players_path_state.iter() {
                let (x, y) = path.get_pos(packet.timestamp);
                let role = get_role_from_id(path.id, config.player_id_start);
                let team = get_team_from_id(path.id, config.player_id_start);
                game["players"][team][role]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({
                        "timestamp": packet.timestamp,
                        "pos": [x, y]
                    }));
            }
            timestamp = packet.timestamp;
        }
    }

    game
}

fn parse_batch(replay_folder: String, output_folder: String, config: Config) {
    let start = std::time::Instant::now();

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

        let game = get_replay_info(replay_path.clone(), &config);

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
    info!("Output: {}, Total execution time: {:.3}", "dump.json", end);
}

fn parse_file(replay_file: String, output_file: String, config: Config) {
    let start = std::time::Instant::now();

    let game = get_replay_info(replay_file.clone(), &config);

    let json_path = PathBuf::from(output_file.clone());
    let mut json = File::create(json_path).unwrap();
    json.write_all(game.to_string().as_bytes()).unwrap();

    let end = start.elapsed().as_secs_f32();
    info!("Output: {}, Total execution time: {:.3}", output_file, end);
}

fn main() {
    setup_logger().unwrap();

    let args = Cli::parse();
    let config = Config::parse(std::path::Path::new(&args.patch_file));

    info!("Loaded patch config from: {}", args.patch_file);

    match args.parsing {
        Parsing::File {
            replay_file,
            output_file,
        } => {
            parse_file(replay_file, output_file, config);
        }
        Parsing::Folder {
            replay_folder,
            output_folder,
        } => parse_batch(replay_folder, output_folder, config),
    }
}
