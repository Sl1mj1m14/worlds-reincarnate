use std::{cell::RefCell, fs, path::PathBuf, sync::Once};

use javarandom::JavaRandom;
use v8::Local;

use crate::{log::log, resources::{self, Generator, Resource, check_hash, download}, version::{FOURK_1, FOURK_2, FOURK_EDITION, FOURK_JS}, world::{Block, Value}};

static V8_INIT: Once = Once::new();

thread_local! {
    static ISOLATE: RefCell<v8::OwnedIsolate> = RefCell::new(v8::Isolate::new(Default::default()));
}

pub fn air (edition: String, version: i32, dims: [i32; 3]) -> Vec<Block> {
    let mut block = Block {id: Value::UByte(0), block_data: None};

    if edition == FOURK_EDITION && version < FOURK_JS {
        block = Block {id: Value::UInt(0), block_data: None};
    }

    let size = (dims[0]*dims[1]*dims[2]) as usize;
    vec![block; size]
}

pub fn javascript (seed: i64, world_size: i32) -> Vec<u8> {
    let mut tiles: Vec<u8> = Vec::new();
    
    //Initializing js engine
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });


    ISOLATE.with(|isolate|{
        //Building js scope and context
        let mut isolate = isolate.borrow_mut();
        v8::scope!(let handle_scope, &mut *isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let scope = &v8::ContextScope::new(handle_scope, context);

        //Loading in script
        let path: PathBuf = resources::HASHES.get().unwrap()[&Resource::Generator(Generator::Javascript)].path.clone();
        if !check_hash(Resource::Generator(Generator::Javascript)) {
            log(0, "Downloading Javascript Generator!");
            if !download(Resource::Generator(Generator::Javascript)) {
                log(1, "Failed to download generator!");
                return
            }
        }

        let mut string: String = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                log(1, "Unable to read in generator!");
                log(2,format!("{e}"));
                return
            }
        };

        string = string.replace("self","//self");
        string = string.replace("//self.addEventListener", r#"/*self.addEventListener"#);
        string += r#"*/"#;

        //Appending js function to retreive tiles
        string += r#"
            function getTiles (seed, size) {
                var level = new RandomLevel();
                var width = size;
                var depth = size;
                var height = 64;
                level.createLevel(seed, width, depth, height);
                return level.tiles;
            }"#;

        //Compiling script
        let source = v8::String::new(scope, &string).unwrap();
        let script = v8::Script::compile(scope, source, None).unwrap();
        script.run(scope).unwrap();

        //Retreiving function
        let global = context.global(scope);
        let function_name = v8::String::new(scope, "getTiles").unwrap();
        let function_obj = global.get(scope, function_name.into()).unwrap();
        let function = v8::Local::<v8::Function>::try_from(function_obj).unwrap();

        //Setting arguments
        let arg1 = v8::Number::new(scope, seed as f64);
        let arg2 = v8::Integer::new(scope, world_size);
        let args: &[Local<v8::Value>; 2] = &[arg1.into(), arg2.into()];

        //Running function
        let result = function.call(scope, v8::undefined(scope).into(), args).unwrap();
        let arr = match v8::Local::<v8::Array>::try_from(result) {
            Ok(t) => t,
            Err(_) => return
        };

        log(-1, "Something happened and it was good?");

        //Parsing js result
        for i in 0..arr.length() {
            let val = arr.get_index(scope, i).unwrap();
            let tile = val.to_int32(scope).unwrap().value() as u8;
            tiles.push(tile);
        }
    });

    log(-1,format!("Passing over {} tiles",tiles.len()));
    tiles

}

pub fn fourk (version: i32, seed: i64, dims: [i32; 3]) -> Option<Vec<Block>> {
    let volume = dims[0] * dims[1] * dims[2];
    let mut arr: Vec<i32> = Vec::with_capacity(volume as usize);

    match version {
        FOURK_1 => { //See original code here: https://discordapp.com/channels/761001494514237490/1483140248778178651/1491505203688636627
            let mut rand = JavaRandom::new();
            rand.set_seed(seed);

            for x in 0..dims[0] {
                for y in 0..dims[1] {
                    for z in 0..dims[2] {
                        let i = ((z * dims[1] * dims[0]) + (y * dims[0]) + x) as usize;
                        arr[i] = if rand.next_int_with_bound(64) < y {1} else {0};
                        if rand.next_int_with_bound(100) == 0 {arr[i as usize] = 255}
                    }
                }
            }
        },
        FOURK_2 => { //See original code here: https://discordapp.com/channels/761001494514237490/1483140248778178651/1491505203688636627
            let mut rand = JavaRandom::new();
            rand.set_seed(seed);

            for x in 0..dims[0] {
                for y in 0..dims[1] {
                    for z in 0..dims[2] {
                        let i = ((z * dims[1] * dims[0]) + (y * dims[0]) + x) as usize;
                        arr[i] = if y > 32 + rand.next_int_with_bound(8) {rand.next_int_with_bound(8) + 1} else {0}
                    }
                }
            }
        },
        FOURK_JS => { //See original code here: https://discordapp.com/channels/761001494514237490/1483140248778178651/1492520115311476756
            V8_INIT.call_once(|| {
                let platform = v8::new_default_platform(0, false).make_shared();
                v8::V8::initialize_platform(platform);
                v8::V8::initialize();
            });

            ISOLATE.with(|isolate|{
                //Building js scope and context
                let mut isolate = isolate.borrow_mut();
                v8::scope!(let handle_scope, &mut *isolate);
                let context = v8::Context::new(handle_scope, Default::default());
                let scope = &v8::ContextScope::new(handle_scope, context);

                let raw: String = r#"
                function getWorld (xDim, yDim, zDim) {
                    var map = new Array(xDim * yDim * zDim);
                    for ( var x = 0; x < xDim; x++) {
                        for ( var y = 0; y < yDim; y++) {
                            for ( var z = 0; z < zDim; z++) {
                                var i = (z * yDim * xDim) + (y * xDim) + x;
                                var yd = (y - 32.5) * 0.4;
                                var zd = (z - 32.5) * 0.4;
                                map[i] = (Math.random() * 16) | 0;
                                if (Math.random() > Math.sqrt(Math.sqrt(yd * yd + zd * zd)) - 0.8)
                                    map[i] = 0;
                            }
                        }
                    }
                    return map
                }
                "#.to_string();

                //Building script
                let source = v8::String::new(scope, &raw).unwrap();
                let script = v8::Script::compile(scope, source, None).unwrap();
                script.run(scope).unwrap();

                //Retreiving function
                let global = context.global(scope);
                let function_name = v8::String::new(scope, "getWorld").unwrap();
                let function_obj = global.get(scope, function_name.into()).unwrap();
                let function = v8::Local::<v8::Function>::try_from(function_obj).unwrap();

                //Setting arguments
                let arg1 = v8::Integer::new(scope, dims[0]);
                let arg2 = v8::Integer::new(scope, dims[1]);
                let arg3 = v8::Integer::new(scope, dims[2]);
                let args: &[Local<v8::Value>; 3] = &[arg1.into(), arg2.into(), arg3.into()];

                //Calling and running javascript function
                let result = function.call(scope, v8::undefined(scope).into(), args).unwrap();
                let js_arr = match v8::Local::<v8::Array>::try_from(result) {
                    Ok(t) => t,
                    Err(_) => return
                };

                for i in 0..js_arr.length() {
                    let val = js_arr.get_index(scope, i).unwrap();
                    arr[i as usize] = val.to_int32(scope).unwrap().value();
                }

            });
        },
        _ => {
            log(1, format!("Invalid 4k version passed to generator: {version}"));
            return None
        }
    }

    let mut blocks: Vec<Block> = Vec::new();

    for int in arr {
        let block: Block = if version >= FOURK_JS {
            Block {id: Value::UByte(int as u8), block_data: None}
        } else {
            Block {id: Value::UInt(int as u32), block_data: None}
        };
        blocks.push(block);
    }

    Some(blocks)
}