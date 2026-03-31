use std::{cell::RefCell, fs, path::PathBuf, sync::Once};

use v8::Local;

use crate::log::log;

const GEN_DIR: &str = "resources/generators";

static V8_INIT: Once = Once::new();

thread_local! {
    static ISOLATE: RefCell<v8::OwnedIsolate> = RefCell::new(v8::Isolate::new(Default::default()));
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
        let path: PathBuf = [GEN_DIR,"RandomLevelWorker.js"].iter().collect();
        if !path.exists() {
            log(1, "Javascript Generator Missing!");
            return
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