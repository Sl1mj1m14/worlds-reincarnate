use std::{collections::HashMap, io::BufRead, num::ParseIntError};

pub fn rotate_array (input_array: BlockArray, output_format: [String; 3]) -> BlockArray {
    let mut output_blocks: Vec<Block> = vec![Block::Byte(0); input_array.blocks.len()];

    /******************************************
    Format of arrays used in this conversion are as follows:
    [0] = x_mult
    [1] = flipped_x
    [2] = y_mult
    [3] = flipped_y
    [4] = z_mult
    [5] = flipped_z
    ******************************************/

    let in_mults: Vec<i32> = format_to_mults(input_array.format, input_array.dims);
    let out_mults: Vec<i32> = format_to_mults(output_format.clone(), input_array.dims);

    for x in 0..input_array.dims[0] {
        for y in 0..input_array.dims[1] {
            for z in 0..input_array.dims[2] {
                output_blocks[(((out_mults[1]-1-x).abs()*out_mults[0])+((out_mults[3]-1-y).abs()*out_mults[2])+((out_mults[5]-1-z).abs()*out_mults[4])) as usize] =
                input_array.blocks[(((in_mults[1]-1-x).abs()*in_mults[0])+((in_mults[3]-1-y).abs()*in_mults[2])+((in_mults[5]-1-z).abs()*in_mults[4])) as usize].clone()
            }
        }
    }

    BlockArray {
        format: output_format,
        dims: input_array.dims,
        blocks: output_blocks
    }
}

//The bounds should be a range to define the size of the world, the dimension arrays are in 'xyz' format
//For example, if the world is going to be 64 blocks from the bottom, the bounds should be 0 and 63
pub fn shrink_world (world: BlockArray, upper_bounds: [i32; 3], lower_bounds: [i32; 3]) -> BlockArray {

    let mults: Vec<i32> = format_to_mults(world.format.clone(), world.dims);
    let mut output_blocks: Vec<Block> = Vec::new();

    for x in 0..world.dims[0] {
        for y in 0..world.dims[1] {
            for z in 0..world.dims[2] {

                //Only pushing blocks if they are within the new world size
                if
                    x >= lower_bounds[0] && x <= upper_bounds[0] &&
                    y >= lower_bounds[1] && y <= upper_bounds[1] &&
                    z >= lower_bounds[2] && z <= upper_bounds[2]
                { 
                    let x1 = (mults[1]-1-x).abs()*mults[0];
                    let y1 = (mults[3]-1-y).abs()*mults[2];
                    let z1 = (mults[5]-1-z).abs()*mults[4];
                    
                    output_blocks.push(world.blocks[(x1+y1+z1) as usize].clone());
                }

            }
        }
    }

    let mut output_dims: [i32; 3] = [0; 3];
    for i in 0..3 {
        output_dims[i] = upper_bounds[i]-lower_bounds[i]+1
    }

    BlockArray {
        format: world.format,
        dims: output_dims,
        blocks: output_blocks
    }

}

pub fn grow_world (world: BlockArray, lower_bounds: [i32; 3], new_dims: [i32; 3], merged_world: Vec<Block>) -> BlockArray {

    let mut upper_bounds: [i32; 3] = [0; 3];
    for i in 0..3 {
        upper_bounds[i] = lower_bounds[i] + world.dims[i] - 1;
    }

    let mults: Vec<i32> = format_to_mults(world.format.clone(), new_dims);
    let mut output_blocks: Vec<Block> = merged_world.clone();
    
    for x in 0..new_dims[0] {
        for y in 0..new_dims[1] {
            for z in 0..new_dims[2] {

                //Only pushing blocks if they are within the old world size
                if
                    x >= lower_bounds[0] && x <= upper_bounds[0] &&
                    y >= lower_bounds[1] && y <= upper_bounds[1] &&
                    z >= lower_bounds[2] && z <= upper_bounds[2]
                { 
                    let x1 = (mults[1]-1-x).abs()*mults[0];
                    let y1 = (mults[3]-1-y).abs()*mults[2];
                    let z1 = (mults[5]-1-z).abs()*mults[4];

                    let x2 =(mults[1]-1-(x-lower_bounds[0])).abs()*mults[0];
                    let y2 =(mults[3]-1-(y-lower_bounds[1])).abs()*mults[2];
                    let z2 =(mults[5]-1-(z-lower_bounds[2])).abs()*mults[4];

                    output_blocks[(x1+y1+z1) as usize] = world.blocks[(x2+y2+z2) as usize].clone()
                }

            }
        }
    }
    
    BlockArray {
        format: world.format,
        dims: new_dims,
        blocks: output_blocks
    }
}

fn format_to_mults (input: [String; 3], dims: [i32; 3]) -> Vec<i32> {

    let mut mults: Vec<i32> = vec![1; 6];
    let mut second_level: i32 = 1;

    match input[0].replace("-","").as_str() {
        "x" => second_level = dims[0],
        "y" => second_level = dims[1],
        "z" => second_level = dims[2],
        _ => () //Log every time I slept with citty's mom
    }

    match input[1].replace("-","").as_str() {
        "x" => mults[0] = second_level,
        "y" => mults[2] = second_level,
        "z" => mults[4] = second_level,
        _ => ()
    }

    match input[2].replace("-","").as_str() {
        "x" => mults[0] = dims[1] * dims[2],
        "y" => mults[2] = dims[0] * dims[2],
        "z" => mults[4] = dims[0] * dims[1],
        _ => ()
    }

    if input.contains(&"-x".to_string()) { mults[1] = dims[0]}
    if input.contains(&"-y".to_string()) { mults[3] = dims[1]}
    if input.contains(&"-z".to_string()) { mults[5] = dims[2]}

    mults
}
