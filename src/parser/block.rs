use std::fmt as display;

use crate::parser::util::*;

#[derive(Debug)]
pub struct Block {
    // HEADER
    pub length: u32,
    pub timestamp: f32,
    pub packet_id: u16,
    pub param: u32,
    // PAYLOAD
    pub payload: Vec<u8>,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            length: 0,
            timestamp: 0.0,
            packet_id: 0,
            param: 0,
            payload: Vec::new(),
        }
    }
}

impl display::Display for Block {
    fn fmt(&self, f: &mut display::Formatter<'_>) -> display::Result {
        write!(
            f,
            "Block {{ length: {}, timestamp: {}, packet_id: {}, param: 0x{:x}, payload: {:?} }}",
            self.length, self.timestamp, self.packet_id, self.param, self.payload
        )
    }
}

pub struct BlockParser {
    chunk: Box<dyn Iterator<Item = u8>>,

    acc_time: f32, // accumulated time in seconds

    previous_block_packet_id: u16,
    previous_block_param: u32,
}

impl BlockParser {
    pub fn new(chunk: Vec<u8>) -> BlockParser {
        BlockParser {
            chunk: Box::new(chunk.into_iter()),
            acc_time: 0.0,
            previous_block_packet_id: 0,
            previous_block_param: 0,
        }
    }

    pub fn next_block(&mut self) -> Option<Block> {
        let mut block: Block = Block::default();

        let marker = self.chunk.next()?;

        // TIMESTAMP
        if marker & 0x80 != 0 {
            // time relative to previous block
            let timestamp = self.chunk.next().unwrap();
            self.acc_time += timestamp as f32 * 0.001;
        } else {
            // absolute time
            self.acc_time = parse_f32(&mut self.chunk).expect("failed to parse timestamp");
        }
        block.timestamp = self.acc_time;

        // BLOCK LENGTH
        if marker & 0x10 != 0 {
            // u8 length
            let block_len = self.chunk.next().unwrap();
            block.length = block_len as u32;
        } else {
            // u32 length
            block.length = parse_u32(&mut self.chunk).unwrap();
        }

        // PACKET ID
        if marker & 0x40 != 0 {
            // previous packet id
            block.packet_id = self.previous_block_packet_id
        } else {
            // u16 packet_id
            block.packet_id = parse_u16(&mut self.chunk).unwrap();
        }

        // BLOCK PARAM
        if marker & 0x20 != 0 {
            // relative to previous block
            let block_param = self.chunk.next().unwrap();
            block.param = block_param as u32 + self.previous_block_param;
        } else {
            // u32 block_param
            block.param = parse_u32(&mut self.chunk).unwrap();
        }

        block.payload = self.chunk.by_ref().take(block.length as usize).collect();

        self.previous_block_packet_id = block.packet_id;
        self.previous_block_param = block.param;

        Some(block)
    }
}
