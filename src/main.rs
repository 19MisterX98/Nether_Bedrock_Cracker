use std::time::Instant;

#[allow(unused_imports)]
use bedrock_cracker::{
    search_bedrock_pattern, world_seeds_from_bedrock_seed, Block,
    BlockType::{BEDROCK, OTHER},
};

fn main() {
    let start = Instant::now();

    let filepath = std::env::args().nth(1).expect("no filepath given");

    
    let contents = std::fs::read_to_string(filepath);
    match contents {
        Ok(contents) => {
            let mut blocks: Vec<Block> = Vec::new();
            for position in contents.split("\n") {
                let mut position = position.split(" ");
                let x = position.next().unwrap().parse::<i32>().unwrap();
                let y = position.next().unwrap().parse::<i32>().unwrap();
                let z = position.next().unwrap().parse::<i32>().unwrap();
                blocks.push(Block::new(x, y, z, BEDROCK));
            }

            let rx = search_bedrock_pattern(&mut blocks, num_cpus::get() as u64);

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
        Err(e) => println!("Could nto read file: {e}"),
    }
}
