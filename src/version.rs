use std::{io::{Error,ErrorKind}, path::PathBuf};

use crate::log::log;

const SAMVID_DIR: &str = "resources";
const FILE_NAME: &str = "samvid.csv";

pub const JAVA_EDITION: &str = "java";

pub const J_PC16: i32 = 40; //pc-161148
pub const J_C12: i32 = 10110; //c0.0.12a_03
pub const J_C13: i32 = 10140; //c0.0.13a-launcher
pub const J_C13_03: i32 = 10150; //c0.0.13a_03-launcher
pub const J_C29: i32 = 10780; //c0.29_02
pub const J_C30: i32 = 10810; //c0.30c-launcher

#[derive(Clone)]
pub struct Samvid {
    pub id: i32,
    pub display: String
}

#[derive(Clone)]
pub struct Edition {
    pub id: String,
    pub versions: Vec<Samvid>
}

pub fn get () -> Result<Vec<Edition>,Error> {
    let path: PathBuf = [SAMVID_DIR,FILE_NAME].iter().collect();

    if !path.exists() {
        //Handle downloading from resource page
        return Err(Error::from(ErrorKind::NotFound))
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let mut editions: Vec<Edition> = Vec::new();
    let mut versions: Vec<Samvid> = Vec::new();
    let mut edition: String = String::default();


    for (index,line) in reader.records().enumerate() {
        let clean = line?;
        let value = clean.get(0).unwrap();
        let id = clean.get(1).unwrap();

        if id == "" {
            if index != 0 {
                editions.push(
                    Edition {
                        id: edition.clone(),
                        versions: versions.clone()
                    });
            }
            edition = value.to_lowercase().to_string();
            versions = Vec::new();
            log(0,format!("Unpacking Edition: {}",value));
        } else {
            let i: i32 = match id.parse() {
                Ok(j) => j,
                Err(_) => {
                    log(1,format!("Unable to read samvid \"{}\" of {} - Skipping",id,value));
                    continue;
                }
            };
            versions.push(
                Samvid { 
                    id: i, 
                    display: value.to_string()
                });
        }
    }

    if edition != "" {
        editions.push(
            Edition {
                id: edition.clone(),
                versions: versions.clone()
            });
    }

    Ok(editions)
}