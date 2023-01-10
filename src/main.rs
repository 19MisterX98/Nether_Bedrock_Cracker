extern crate core;

use std::time::SystemTime;
use java_random::{JAVA_LCG, Random};
use next_long_reverser::get_next_long;
use bedrock_cracker::{Block, create_filter_tree, Layer};
use bedrock_cracker::BlockType::{BEDROCK, OTHER};

fn main() {
    let start = SystemTime::now();

    let mut blocks = Vec::new();

    blocks.push(Block::new(-98, 4, -469, BEDROCK));
    blocks.push(Block::new(-101, 4, -465, BEDROCK));
    blocks.push(Block::new(-101, 4, -463, BEDROCK));
    blocks.push(Block::new(-101, 4, -457, BEDROCK));
    blocks.push(Block::new(-101, 4, -453, BEDROCK));
    blocks.push(Block::new(-100, 4, -456, BEDROCK));
    blocks.push(Block::new(-100, 4, -449, BEDROCK));
    blocks.push(Block::new(-99, 4, -464, BEDROCK));
    blocks.push(Block::new(-99, 4, -459, BEDROCK));
    blocks.push(Block::new(-99, 4, -455, BEDROCK));
    blocks.push(Block::new(-98, 4, -461, BEDROCK));
    blocks.push(Block::new(-98, 4, -460, BEDROCK));
    blocks.push(Block::new(-96, 4, -467, BEDROCK));
    blocks.push(Block::new(-96, 4, -465, BEDROCK));
    blocks.push(Block::new(-96, 4, -464, BEDROCK));
    blocks.push(Block::new(-96, 4, -452, BEDROCK));
    blocks.push(Block::new(-95, 4, -465, BEDROCK));
    blocks.push(Block::new(-95, 4, -458, BEDROCK));
    blocks.push(Block::new(-95, 4, -449, BEDROCK));
    blocks.push(Block::new(-94, 4, -462, BEDROCK));
    blocks.push(Block::new(-94, 4, -459, BEDROCK));
    blocks.push(Block::new(-94, 4, -454, BEDROCK));
    blocks.push(Block::new(-93, 4, -467, BEDROCK));
    blocks.push(Block::new(-93, 4, -465, BEDROCK));
    blocks.push(Block::new(-93, 4, -463, BEDROCK));
    blocks.push(Block::new(-93, 4, -455, BEDROCK));
    blocks.push(Block::new(-92, 4, -468, BEDROCK));
    blocks.push(Block::new(-92, 4, -467, BEDROCK));

    let checks = create_filter_tree(&mut blocks);

    for upper_bits in (0..0xFFFF_FFFF_FFFF).step_by(1<<12) {
        checks.run_checks(upper_bits);
    }

    let delta = SystemTime::now().duration_since(start).expect("Couldnt stop time");
    println!("Time elapsed: {}", delta.as_secs());
}