use std::time::Instant;

#[allow(unused_imports)]
use bedrock_cracker::{
    search_bedrock_pattern, world_seeds_from_bedrock_seed, Block,
    BlockType::{BEDROCK, OTHER},
};

fn read_coord(coord: Option<&str>, mut line: usize, name: &str) -> i32 {
    line += 1; // enumerate starts at 0
    coord.unwrap_or_else(|| panic!("n {} coord in line {}", name, line))
        .parse().unwrap_or_else(|_| panic!("{} coord in line {} is not a number", name, line))
}

fn main() {
    let start = Instant::now();

    let filepath = std::env::args().nth(1).expect("no filepath given, use: cargo run -- your_coords.txt");

    
    let contents = std::fs::read_to_string(filepath);
    match contents {
        Ok(contents) => {
            let mut blocks: Vec<Block> = Vec::new();
            for (line_number, position) in contents.lines().enumerate().filter(|(_, line)| !line.is_empty()) {
                let mut position = position.split(" ");
                let x = read_coord(position.next(), line_number, "x");
                let y = read_coord(position.next(), line_number, "y");
                let z = read_coord(position.next(), line_number, "z");
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
