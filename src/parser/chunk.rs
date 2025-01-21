use std::io::Read;

use zstd::stream::read::Decoder;

use crate::parser::util::*;

#[derive(Debug)]
pub struct Chunk {
    pub id: u32,
    pub type_: u8,
    pub id_2: u32,
    pub uncompressed_len: u32,
    pub compressed_len: u32,

    pub payload: Option<Vec<u8>>,
}

pub struct ChunkParser {
    replay_content: Box<dyn Iterator<Item = u8>>,

    replay_len: u32,
    cursor: usize,
}

impl ChunkParser {
    pub fn new(mut buffer: Vec<u8>) -> Self {
        let (_, last_4_bytes) = buffer.split_at(buffer.len() - 4);
        let metadata_len = u32::from_le_bytes(last_4_bytes.try_into().unwrap());

        buffer.truncate(buffer.len() - metadata_len as usize - 4); // remove metadata
        buffer.truncate(buffer.len() - 0x100); // remove signature
                                               //buffer.drain(0..0x1D); // remove replay header
        Self::skip_rofl_header_size(&mut buffer);

        let replay_len = buffer.len() as u32;

        ChunkParser {
            replay_content: Box::new(buffer.into_iter()),
            replay_len,
            cursor: 0,
        }
    }

    fn skip_rofl_header_size(buffer: &mut Vec<u8>) {
        // FIXME: very bad
        buffer.drain(0..0x10);
        if buffer[0xC] == 1 {
            buffer.drain(0..0xC);
        } else {
            buffer.drain(0..0xD);
        }
    }

    fn parse_chunk_header(&mut self) -> Option<(u32, u8, u32, u32, u32)> {
        Some((
            parse_u32(&mut self.replay_content).unwrap(),
            self.replay_content.next()?,
            parse_u32(&mut self.replay_content).unwrap(),
            parse_u32(&mut self.replay_content).unwrap(),
            parse_u32(&mut self.replay_content).unwrap(),
        ))
    }

    pub fn next_chunk(&mut self) -> Option<Chunk> {
        if self.replay_len <= self.cursor as u32 {
            return None;
        }

        let (chunk_id, chunk_type, chunk_id_2, chunk_uncompressed_len, chunk_compressed_len) =
            self.parse_chunk_header()?;

        self.cursor += 0x11;

        let payload: Option<Vec<u8>> = if chunk_compressed_len != 0 {
            let compressed_payload: Vec<u8> = self
                .replay_content
                .by_ref()
                .take(chunk_compressed_len as usize)
                .collect();

            self.cursor += chunk_compressed_len as usize;

            let mut uncompressed_payload = Vec::new();
            let mut decoder = Decoder::new(compressed_payload.as_slice()).unwrap();
            decoder.read_to_end(&mut uncompressed_payload).unwrap();

            Some(uncompressed_payload)
        } else {
            self.replay_content.nth(chunk_uncompressed_len as usize - 1);
            self.cursor += chunk_uncompressed_len as usize;
            None
        };

        Some(Chunk {
            id: chunk_id,
            type_: chunk_type,
            id_2: chunk_id_2,
            uncompressed_len: chunk_uncompressed_len,
            compressed_len: chunk_compressed_len,
            payload,
        })
    }
}
