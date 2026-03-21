use std::{fmt::Error, fs::{File, OpenOptions}, path::PathBuf, sync::OnceLock, io::Write};

use chrono::Local;
use colored::{ColoredString, Colorize};

use crate::{DEFAULT_TIMESTAMP, TIMESTAMP};

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

    let mut path: PathBuf = [LOG_DIR,&name].iter().collect();

    let mut ver: i16 = 0;

    while path.exists() {
        ver += 1;
        name = format!("{}{}-{}{}",base,timestamp,ver,ext);
        path = [LOG_DIR,&name].iter().collect();
    }

    LOG_NAME.set(name).unwrap();
    
    let _ = File::create(path);
}

pub fn log (code: u8, input: impl Msg) {
    let mut msg: String = Msg::to_msg(&input);
    let timestamp: String = Local::now().format("%Y-%m-%d %H:%M:%S : ").to_string();

    let path: PathBuf = [LOG_DIR,LOG_NAME.get().unwrap_or(&LOG_DEFAULT.to_string())].iter().collect();

    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path);

    let _ = match log_file {
        Ok(mut file) => writeln!(file,"{}{}",timestamp,msg),
        Err(_) => Ok(())
    };

    let msg_colored: ColoredString = match code {
        1 => {
            msg = String::from("WARNING: ") + &msg;
            msg.yellow()
        },
        2 => {
            msg = String::from("ERROR: ") + &msg;
            msg.red()
        },
        _ => msg.white()
    };
    println!("{}{}",timestamp.truecolor(LIGHT_GREY[0],LIGHT_GREY[1],LIGHT_GREY[2]),msg_colored);
}

