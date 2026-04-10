use std::{collections::HashMap, fs::{self, File, create_dir_all, remove_file}, io::Write, path::PathBuf, sync::OnceLock};

use csv;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

use crate::{PROJECT_DIR, log::log};

const SHEET_ID: &str = "1_B371N-C69SWg5LEHa5YXKv9X6IL-EBagwsQxZ9NCTo";

const SAMVID_GID: &str = "1519184165";

const BLOCK_GID: &str = "75311798";
const WORLD_DATA_GID: &str = "2102518304";

pub static HASHES: OnceLock<HashMap<Resource, Info>> = OnceLock::new();
static CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resource {
    Hash,
    Samvid,
    Map(Map),
    Generator(Generator)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Map {
    Block,
    WorldData
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Generator {
    Javascript
}

#[derive(Clone, Debug)]
pub struct Info {
    pub hash: String,
    pub url: String,
    pub path: PathBuf
}

pub fn initialize () {
    let base_url = format!("https://docs.google.com/spreadsheets/d/{SHEET_ID}/export?format=csv");
    let mut path = PROJECT_DIR.get().unwrap().clone();

    let client = reqwest::blocking::Client::new();
    CLIENT.set(client).unwrap();

    let result = CLIENT.get().unwrap().get(base_url.clone()).send();
    match result {
        Ok(resp) => 'result: {
            let status = resp.status();
            if status.as_u16() != 200 {
                log(1, format!("REQUEST {}", status.as_u16()));
                log(1, "Failed to download hash file! Resources may be outdated");
                break 'result
            }

            let content = match resp.bytes() {
                Ok(b) => b,
                Err(e) => {
                    log(1, format!("{e}"));
                    log(1, "Failed to read hash file! Resources may be outdated");
                    break 'result
                }
            };

            path.push("hash.csv");
            if path.exists() {
                log(0,"Updating hash file");
                let _ = remove_file(path.clone());
            } else {
                log(0,"Creating hash file");
            }

            let mut file = File::create_new(path.clone()).unwrap();
            let _ = file.write_all(&content);

        },
        Err(_) => log(1, "Unable to download hash file! Resources may be outdated")
    }

    let mut map: HashMap<String, String> = HashMap::new();
    if let Ok(mut reader) = csv::ReaderBuilder::new().has_headers(false).from_path(path) {
        for line in reader.records().enumerate() {
            let clean = line.1.unwrap();
            map.insert(clean.get(0).unwrap().to_uppercase(), clean.get(1).unwrap().to_lowercase());
        }
    };
    let mut hashes: HashMap<Resource, Info> = HashMap::new();

    hashes.insert(
        Resource::Samvid, Info {
             hash: map.get("SAMVID").unwrap_or(&String::default()).clone(),
             url: format!("{base_url}&gid={SAMVID_GID}"), 
             path: [PROJECT_DIR.get().unwrap().clone(),"samvid.csv".into()].iter().collect()
    });

    hashes.insert(
        Resource::Map(Map::Block), Info {
             hash: map.get("MAP-BLOCK").unwrap_or(&String::default()).clone(),
             url: format!("{base_url}&gid={BLOCK_GID}"), 
             path: [PROJECT_DIR.get().unwrap().clone(),"resources".into(),"maps".into(),"block_ids.csv".into()].iter().collect()
    });

    hashes.insert(
        Resource::Map(Map::WorldData), Info {
             hash: map.get("MAP-WORLDDATA").unwrap_or(&String::default()).clone(),
             url: format!("{base_url}&gid={WORLD_DATA_GID}"), 
             path: [PROJECT_DIR.get().unwrap().clone(),"resources".into(),"maps".into(),"world_data.csv".into()].iter().collect()
    });

    hashes.insert(
        Resource::Generator(Generator::Javascript), Info {
             hash: map.get("GENERATOR-JAVASCRIPT").unwrap_or(&String::default()).clone(),
             url: "https://classic.minecraft.net/assets/js/RandomLevelWorker.js".to_string(), 
             path: [PROJECT_DIR.get().unwrap().clone(),"resources".into(),"generators".into(),"RandomLevelWorker.js".into()].iter().collect()
    });

    HASHES.set(hashes).unwrap();

    if !check_hash(Resource::Samvid) {
        log(0, "Downloading SAMVID List...");
        let _ = download(Resource::Samvid);
    }

    if !check_hash(Resource::Map(Map::Block)) {
        log(0, "Downloading Block ID List...");
        let _ = download(Resource::Map(Map::Block));
    }

    if !check_hash(Resource::Map(Map::WorldData)) {
        log(0, "Downloading World Data List...");
        let _ = download(Resource::Map(Map::WorldData));
    }

}

pub fn check_hash (resource: Resource) -> bool {

    let info = match HASHES.get().unwrap().get(&resource) {
        Some(i) => i.clone(),
        None => {
            log(1, format!("Unknown resource {:?}, assuming hash is false", resource));
            return false
        }
    };

    if !info.path.exists() { return false }

    if let Ok(bytes) = fs::read(info.path) {

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hex = hex::encode(hasher.finalize());

        log(-1, format!("Hash is {}", hex));
        log(-1, format!("Hash should be {}", info.hash));

        return info.hash == hex
    };

    return false
}

pub fn download (resource: Resource) -> bool {

    let info = match HASHES.get().unwrap().get(&resource) {
        Some(i) => i.clone(),
        None => {
            log(1, format!("Unknown resource {:?}, unable to download", resource));
            return false
        }
    };
    let path = info.path.clone();

    let result = CLIENT.get().unwrap().get(info.url.clone()).send();
    match result {
        Ok(resp) => {
            let status = resp.status();
            if status.as_u16() != 200 {
                log(1, format!("REQUEST {}", status.as_u16()));
                log(1, format!("Failed to download {:?}", resource));
                return false
            }

            let content = match resp.bytes() {
                Ok(b) => b,
                Err(e) => {
                    log(1, format!("{e}"));
                    log(1, format!("Failed to parse download of {:?}", resource));
                    return false
                }
            };

            if path.exists() {
                log(0,format!("Updating file at {}", path.clone().to_string_lossy()));
                if remove_file(path.clone()).is_err() {
                    log(1, "Failed to update file");
                    return false
                }
            }

            let mut dir = path.clone();
            dir.pop();
            if create_dir_all(dir).is_err() {
                log(1,format!("Failed to create directory at {}", path.clone().to_string_lossy()));
            }
            
            let mut file = match File::create_new(path.clone()) {
                Ok(f) => f,
                Err(e) => {
                    log(1,format!("Failed to create file at {}", path.clone().to_string_lossy()));
                    log(1, format!("{e}"));
                    return false
                }
            };

            match file.write_all(&content) {
                Ok(_) => return true,
                Err(e) => {
                    log(1,format!("Failed to write file at {}", path.clone().to_string_lossy()));
                    log(1, format!("{e}"));
                    return false
                }
            }

        },
        Err(e) => {
            log(1, format!("Unable to fetch {:?}", resource));
            log(1, format!("{e}"));
            return false
        }
    }
}
