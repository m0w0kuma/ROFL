use crate::parser::block::*;
use crate::parser::chunk::*;

use rayon::prelude::*;

pub fn get_blocks(buffer: Vec<u8>) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();

    let mut chunk_parser = ChunkParser::new(buffer);
    while let Some(chunk) = chunk_parser.next_chunk() {
        if chunk.payload.is_some() && chunk.type_ != 0x2 {
            let mut block_parser = BlockParser::new(chunk.payload.unwrap());
            while let Some(block) = block_parser.next_block() {
                blocks.push(block);
            }
        }
    }

    blocks
}

pub fn get_blocks_with_id(replay_file: &[u8], id: u16) -> Vec<(f32, Vec<u8>)> {
    get_blocks(replay_file.to_vec())
        .par_iter()
        .filter(|b| b.packet_id == id)
        .map(|b| (b.timestamp, b.payload.clone()))
        .collect()
}
