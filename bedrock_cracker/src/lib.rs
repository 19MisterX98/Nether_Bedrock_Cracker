mod block_data;
mod layer;
pub mod raw_data;

use std::cmp::min;

use std::{thread};
use std::sync::Arc;
use async_std::task::spawn_blocking;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::channel;
use crate::block_data::{BlockFilter, get_filter_power};
use crate::layer::{create_filter_tree, flat_search};
use crate::raw_data::block::Block;
use crate::raw_data::modes::{BedrockGeneration, OutputMode};
use crate::raw_data::sender::Sender;

/// cbindgen:ignore
const MASK48: u64 = 0xFFFF_FFFF_FFFF;
/// cbindgen:ignore
const ROOF_HASH: u64 = 343340730;
/// cbindgen:ignore
const FLOOR_HASH: u64 = 2042456806;

/// cbindgen:ignore
const CHUNK_SIZE: u64 = (1 << 12) * (1 << 25); // interrupts every 2^25 seeds

/// this estimate is naive
#[no_mangle]
pub extern fn estimate_result_amount(blocks_ptr: *const Block, len: usize) -> u64 {
    let blocks: &[Block] = unsafe {
        std::slice::from_raw_parts(blocks_ptr, len)
    };

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

#[no_mangle]
pub extern fn crack(blocks_ptr: *const Block, len: usize, threads: u64, mode: BedrockGeneration, output_mode: OutputMode) -> VecI64 {
    let blocks: &[Block] = unsafe {
        std::slice::from_raw_parts(blocks_ptr, len)
    };

    let blocks_owned = blocks.to_vec();

    let runtime = Runtime::new().unwrap();

    runtime.block_on(crack_internal(blocks_owned, threads, mode, output_mode)).into()
}

pub async fn crack_internal(blocks: Vec<Block>, threads: u64, mode: BedrockGeneration, output_mode: OutputMode, ) -> Vec<i64> {
    let (sender, mut receiver) = channel(100);

    spawn_blocking(move || search_bedrock_pattern(&*blocks, threads, mode, output_mode, sender));

    let mut seeds = vec![];
    while let Some(pl_event) = receiver.recv().await {
        match pl_event {
            CrackProgress::Progress(_) => {
                seeds = vec![];
            }
            CrackProgress::Seed(num) => {
                seeds.push(num as i64);
            }
        };
    }
    seeds
}

#[derive(Clone, Debug)]
pub enum CrackProgress {
    Seed(u64),
    Progress(u64),
}

#[derive(Debug, Clone)]
pub enum CrackerEvent {
    Started,
    Finished(Vec<i64>),
}

#[repr(C)]
pub struct VecI64 {
    ptr: *const i64,
    len: usize
}

impl Into<VecI64> for Vec<i64> {
    fn into(self) -> VecI64 {
        VecI64 {
            ptr: self.as_ptr(),
            len: self.len(),
        }
    }
}
