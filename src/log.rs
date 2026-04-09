use std::{fmt::Error, fs::{File, OpenOptions, create_dir_all}, io::Write, path::PathBuf, sync::OnceLock};

use chrono::Local;
use colored::{ColoredString, Colorize};

use crate::{DEBUG, DEBUG_FLAG, DEFAULT_TIMESTAMP, PROJECT_DIR, TIMESTAMP};

const LOG_DIR: &str = "logs";
const LOG_DEFAULT: &str = "log.txt";

static LOG_NAME: OnceLock<String> = OnceLock::new();

const LIGHT_GREY: &[u8] = &[107; 3];

pub trait Msg {
    fn to_msg (&self) -> String;
}

impl Msg for String {
    fn to_msg (&self) -> String {
        self.to_string()
    }
}

impl Msg for &str {
    fn to_msg (&self) -> String {
        self.to_string()
    }
}

impl Msg for Error {
    fn to_msg (&self) -> String {
        self.to_string()
    }
}

pub fn start () {
    let binding = DEFAULT_TIMESTAMP.to_string();
    let timestamp = TIMESTAMP.get().unwrap_or(&binding);

    let base: &str = "log_";
    let ext: &str = ".txt";

    let mut name: String = format!("{}{}{}",base,timestamp,ext);

    let mut path: PathBuf = PROJECT_DIR.get().unwrap().clone();
    path.push(LOG_DIR);
    path.push(&name);

    let mut ver: i16 = 0;

    while path.exists() {
        path.pop();
        ver += 1;
        name = format!("{}{}-{}{}",base,timestamp,ver,ext);
        path.push(&name);
    }

    LOG_NAME.set(name.clone()).unwrap();
    path.pop();
    let _ = create_dir_all(path.clone());
    path.push(&name);

    match File::create(path) {
        Ok(_) => (),
        Err(_) => panic!("Failed to start logging!")
    }
}

pub fn log (code: i8, input: impl Msg) {
    if code == -1 && !DEBUG_FLAG && !DEBUG.get().unwrap().flag {return}

    let mut msg: String = Msg::to_msg(&input);
    let timestamp: String = Local::now().format("%Y-%m-%d %H:%M:%S : ").to_string();

    let mut path: PathBuf = PROJECT_DIR.get().unwrap().clone();
    path.push(LOG_DIR);
    path.push(LOG_NAME.get().unwrap_or(&LOG_DEFAULT.to_string()));

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path);

    let msg_colored: ColoredString = match code {
        1 => {
            msg = String::from("WARNING: ") + &msg;
            msg.yellow()
        },
        2 => {
            msg = String::from("ERROR: ") + &msg;
            msg.red()
        },
        -1 => {
            msg = String::from("DEBUG: ") + &msg;
            msg.truecolor(LIGHT_GREY[0],LIGHT_GREY[1],LIGHT_GREY[2])
        }
        _ => {
            msg = String::from("INFO: ") + &msg;
            msg.white()
        }
    };

    let _ = match log_file {
        Ok(mut file) => writeln!(file,"{}{}",timestamp,msg),
        Err(_) => Ok(())
    };

    println!("{}{}",timestamp.truecolor(LIGHT_GREY[0],LIGHT_GREY[1],LIGHT_GREY[2]),msg_colored);
}

pub fn close () {
    log(0,format!("Ending Session at {}",Local::now().format("%Y-%m-%d %H:%M:%S")))
}

