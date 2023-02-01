use std::time::Instant;

#[allow(unused_imports)]
use bedrock_cracker::{
    search_bedrock_pattern, world_seeds_from_bedrock_seed, Block,
    BlockType::{BEDROCK, OTHER},
};

fn main() {
    let start = Instant::now();

    let mut blocks = vec![
        Block::new(-98, 4, -469, BEDROCK),
        Block::new(-101, 4, -465, BEDROCK),
        Block::new(-101, 4, -463, BEDROCK),
        Block::new(-101, 4, -457, BEDROCK),
        Block::new(-101, 4, -453, BEDROCK),
        Block::new(-100, 4, -456, BEDROCK),
        Block::new(-100, 4, -449, BEDROCK),
        Block::new(-99, 4, -464, BEDROCK),
        Block::new(-99, 4, -459, BEDROCK),
        Block::new(-99, 4, -455, BEDROCK),
        Block::new(-98, 4, -461, BEDROCK),
        Block::new(-98, 4, -460, BEDROCK),
        Block::new(-96, 4, -467, BEDROCK),
        Block::new(-96, 4, -465, BEDROCK),
        Block::new(-96, 4, -464, BEDROCK),
        Block::new(-96, 4, -452, BEDROCK),
        Block::new(-95, 4, -465, BEDROCK),
        Block::new(-95, 4, -458, BEDROCK),
        Block::new(-95, 4, -449, BEDROCK),
        Block::new(-94, 4, -462, BEDROCK),
        Block::new(-94, 4, -459, BEDROCK),
        Block::new(-94, 4, -454, BEDROCK),
        Block::new(-93, 4, -467, BEDROCK),
        Block::new(-93, 4, -465, BEDROCK),
        Block::new(-93, 4, -463, BEDROCK),
        Block::new(-93, 4, -455, BEDROCK),
        Block::new(-92, 4, -468, BEDROCK),
        Block::new(-92, 4, -467, BEDROCK),
    ];

    let rx = search_bedrock_pattern(&mut blocks, 12);
    println!("Started Cracking");

    for seed in rx {
        let world_seeds = world_seeds_from_bedrock_seed(seed, true);
        for world_seed in world_seeds {
            println!("Found World seed: {world_seed}");
        }
    }
    let execution_time = start.elapsed().as_secs();
    println!("Time elapsed: {execution_time}s");
}
