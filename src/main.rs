extern crate core;

use std::time::SystemTime;
use bedrock_cracker::{Block, search_bedrock_pattern, world_seeds_from_bedrock_seed};
#[allow(unused_imports)]
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

    let rx = search_bedrock_pattern(&mut blocks, 12);
    println!("Started Cracking");

    for seed in rx {
        let world_seeds = world_seeds_from_bedrock_seed(seed, true);
        for world_seed in world_seeds {
            println!("Found World seed: {}", world_seed);
        }
    }

    let delta = SystemTime::now().duration_since(start).expect("Couldnt stop time");
    println!("Time elapsed: {}", delta.as_secs());
}