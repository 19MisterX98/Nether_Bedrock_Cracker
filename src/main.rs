extern crate core;
use std::time::SystemTime;
use bedrock_cracker::{Block, CheckObject};
use bedrock_cracker::BlockType::{BEDROCK, OTHER};

const MASK48: u64 = (1<<48)-1;
const MULT: u64 = 25214903917;

fn main() {
    println!("Hello, world!");

    let mut blocks = Vec::new();

    blocks.push(Block::new(1,4,0,BEDROCK));
    blocks.push(Block::new(1,3,1,BEDROCK));
    blocks.push(Block::new(1,2,2,BEDROCK));
    blocks.push(Block::new(1,1,3,BEDROCK));
    blocks.push(Block::new(1,4,4,BEDROCK));
    blocks.push(Block::new(1,3,5,BEDROCK));
    blocks.push(Block::new(1,2,6,BEDROCK));
    blocks.push(Block::new(1,1,7,BEDROCK));
    blocks.push(Block::new(1,4,8,OTHER));
    blocks.push(Block::new(1,3,9,OTHER));
    blocks.push(Block::new(1,2,10,OTHER));
    blocks.push(Block::new(1,1,11,OTHER));
    blocks.push(Block::new(1,4,12,OTHER));
    blocks.push(Block::new(1,3,13,OTHER));
    blocks.push(Block::new(1,2,14,OTHER));
    blocks.push(Block::new(1,1,15,OTHER));
    blocks.push(Block::new(1,4,16,BEDROCK));
    blocks.push(Block::new(1,4,17,BEDROCK));

    let _floor_seed = compute_floor_seeds(&mut blocks, (1<<12)-1);

}

fn sorting_blocks(blocks: &mut Vec<Block>) -> Vec<CheckObject> {
    let mut checks: Vec<CheckObject> = Vec::new();
    //sort everything by filterpower
    loop {
        let max_filter_power = blocks.iter()
            .map(|block| {
                (0..=12)
                    .map(|bits| block.discarded_seeds(bits))
                    .reduce(f64::max)
                    .unwrap()
            })
            .reduce(f64::max)
            .unwrap();
        if max_filter_power == 0.0 {
            break;
        }
        blocks.iter_mut().for_each(|mut block| {
            (0..=12)
                .for_each(|num| {
                    if block.discarded_seeds(num) == max_filter_power {
                        println!("Filter power: {}, {}", max_filter_power, num);
                        block.check_with_bits(num, &mut checks);
                    }
                })
            });
    }
    checks
}


fn compute_floor_seeds(blocks: &mut Vec<Block>, lower_bits_mask: u64) -> Vec<u64> {
    let checks = sorting_blocks(blocks);

    let mut output = Vec::new();
    let start = SystemTime::now();

    for check in checks.iter() {
        //println!("{:#?}", check);
    }

    println!("length: {}",checks.len());

    /*
    2. group blocks by information/layer
    3. figure out which layer to check and with how many bits to do that.
    4. create lambdas and chain them?

    //roof_seed: ("minecraft:bedrock_roof".hash ^ mult ^ seed)*mult+addend
    //floor_seed: ("minecraft:bedrock_floor".hash ^ mult ^ seed)*mult+addend
     */
    panic!();

    'outer: for upper_bits in (0..MASK48).step_by(lower_bits_mask as usize + 1) {

        for block in blocks.iter() {
            if block.upper_bound == 0 {
                continue 'outer;
            }
        }


        let now = SystemTime::now().duration_since(start).unwrap().as_secs();
        let chance = upper_bits as f64 / (1i64<<48) as f64;
        println!("{chance}, {now}");

        output.push(upper_bits);
    }

    output
}


//todo the addition should be respected aswell

