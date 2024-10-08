mod block_data;
mod layer;
pub mod raw_data;

use std::cmp::min;

use std::{thread};
use std::sync::Arc;


use crate::block_data::{BlockFilter, get_filter_power};
use crate::layer::{create_filter_tree, flat_search};
use crate::raw_data::block::Block;
use crate::raw_data::modes::{BedrockGeneration, OutputMode};
use crate::raw_data::sender::Sender;

const MASK48: u64 = 0xFFFF_FFFF_FFFF;
const ROOF_HASH: u64 = 343340730;
const FLOOR_HASH: u64 = 2042456806;

const CHUNK_SIZE: u64 = (1 << 12) * (1 << 25); // interrupts every 2^25 seeds

/// this estimate is naive
pub fn estimate_result_amount(blocks: &[Block]) -> u64 {
    let filters: Vec<_> = blocks.iter()
        .map(|block | BlockFilter::from(block, BedrockGeneration::Normal))
        .collect();
    get_filter_power(&filters)
}

pub fn search_bedrock_pattern_with_list<S: Sender + 'static>(blocks: &[Block], thread_count: u64, seed_list: &[u64], mode: BedrockGeneration, sender: S) {
    let mut roof_blocks = Vec::new();
    let mut floor_blocks = Vec::new();

    for block in blocks.iter() {
        let check = BlockFilter::from(&block, mode).create_check(0);
        if block.y > 5 {
            roof_blocks.push(check);
        } else {
            floor_blocks.push(check);
        }
    }
    let roof_blocks = Arc::new(roof_blocks);
    let floor_blocks = Arc::new(floor_blocks);


    let chunk_size = seed_list.len() as f64 / thread_count as f64;


    let chunks = seed_list.chunks(chunk_size.ceil() as usize);

    for chunk in chunks.into_iter() {

        let chunk = Vec::from(chunk);
        let sender = sender.clone();
        let roof_blocks = roof_blocks.clone();
        let floor_blocks = floor_blocks.clone();

        thread::spawn(move||{

            flat_search(&chunk, roof_blocks, floor_blocks, sender);
        });
    }
}


pub fn search_bedrock_pattern<S: Sender + 'static>(blocks: &[Block], thread_count: u64, mode: BedrockGeneration, output: OutputMode, sender: S) {
    let checks = create_filter_tree(blocks, mode, output, sender.clone());

    for thread in 0..thread_count {
        let mut start_bits = (thread * (1 << 36)) / thread_count;
        let mut end_bits = ((thread + 1) * (1 << 36)) / thread_count;
        start_bits <<= 12;
        end_bits <<= 12;

        let checks = checks.clone();

        let sender = sender.clone();

        thread::spawn(move || {
            while start_bits < end_bits {
                let chunk_end = min(start_bits + CHUNK_SIZE, end_bits);
                for upper_bits in (start_bits..chunk_end).step_by(1 << 12) {
                    checks.run_checks(upper_bits);
                }
                //dropping the receiver stops the threads
                if !sender.send(CrackProgress::Progress(chunk_end - start_bits)) {
                    return;
                }

                start_bits = chunk_end;
            }
        });
    }
}

#[derive(Clone, Debug)]
pub enum CrackProgress {
    Seed(u64),
    Progress(u64),
}
