use mc_classic_js::JSLevel;
use mc_classic::Level;

pub fn classic_to_js (classic: Level, seed: i64, opt: u8) -> JSLevel {
    let mut tile_map: Vec<u8> = Vec::new();

    if classic.blocks.is_some() { tile_map = classic.blocks.unwrap().clone() }

    let mut i: usize = 0;
    for tile in tile_map {
        match tile {
            1 => tile_map[i] = 2, //Stone
            2 => tile_map[i] = 1, //Grass Block
            4 => tile_map[i] = 9, //Cobblestone
            5 => tile_map[i] = 4, //Planks
            6 => tile_map[i] = 8, //Sapling
            7 => tile_map[i] = 10, //Bedrock
            8 => tile_map[i] = 7, //Flowing Water
            9 => tile_map[i] = 7, //Stationary Water
            10 => tile_map[i] = 17, //Flowing Lava
            11 => tile_map[i] = 17, //Stationary Lava
            12 => tile_map[i] = 11, //Sand
            13 => tile_map[i] = 12, //Gravel
            14 => tile_map[i] = 18, //Gold Ore
            15 => tile_map[i] = 19, //Iron Ore
            16 => tile_map[i] = 20, //Coal Ore
            17 => tile_map[i] = 13, //Logs
            18 => tile_map[i] = 14, //Leaves
            19 => tile_map[i] = 22, //Sponge
            20 => tile_map[i] = 23, //Glass
            21 => tile_map[i] = 24, //Red Cloth
            22 => tile_map[i] = 25, //Orange Cloth
            23 => tile_map[i] = 26, //Yellow Cloth
            24 => tile_map[i] = 27, //Chartreuse Cloth
            25 => tile_map[i] = 28, //Green Cloth
            26 => tile_map[i] = 29, //Spring Green Cloth
            27 => tile_map[i] = 30, //Cyan Cloth
            28 => tile_map[i] = 31, //Capri Cloth
            29 => tile_map[i] = 32, //Ultramarine Cloth
            30 => tile_map[i] = 34, //Purple Cloth
            31 => tile_map[i] = 33, //Violet Cloth
            32 => tile_map[i] = 35, //Magenta Cloth
            33 => tile_map[i] = 36, //Rose Cloth
            34 => tile_map[i] = 37, //Dark Gray Cloth
            35 => tile_map[i] = 38, //Light Gray Cloth
            36 => tile_map[i] = 39, //White Cloth
            37 => tile_map[i] = 6, //Dandelion
            38 => tile_map[i] = 5, //Rose
            39 => tile_map[i] = 16, //Brown Mushroom
            40 => tile_map[i] = 15, //Red Mushroom
            41 => tile_map[i] = 21, //Block of Gold
            42 => tile_map[i] = 0, //Block of Iron
            43 => tile_map[i] = 0, //Double Slab
            44 => tile_map[i] = 0, //Slab
            45 => tile_map[i] = 0, //Bricks
            46 => tile_map[i] = 0, //TNT
            47 => tile_map[i] = 0, //Bookshelf
            48 => tile_map[i] = 0, //Mossy Cobblestone
            49 => tile_map[i] = 0, //Obsidian
            _ => tile_map[i] = 0,
        }
        i += 1;
    }

    //TO DO: Trim level if height is invalid
    //Rearrange blocks for non-square worlds

    let world_size: i32 = 256; //Replace 256 with a default from a config file

    if classic.width.is_some() && classic.height.is_some() {
        world_size = if classic.width >= classic.height { classic.width } else { classic.height }
    }

    let seed1: i64 = if seed > 0 {seed} else {1}; //Replace 1 with a default seed from a config file

    let temp = 0;
}