//ENSURE SAMVIDS START AFTER 0 - 0 WILL BREAK

use std::{io::{Error,ErrorKind}, path::PathBuf};

use crate::{log::log, resources::{self, Resource}};

pub const JAVA_EDITION: &str = "java";
pub const JAVASCRIPT_EDITION: &str = "classicjs";
pub const FOURK_EDITION: &str = "fourk";

pub const J_PC16: i32 = 40; //pc-161148
pub const J_C12: i32 = 10110; //c0.0.12a_03
//pub const J_C13: i32 = 10140; //c0.0.13a-launcher
pub const J_C13_03: i32 = 10150; //c0.0.13a_03-launcher
pub const J_C14: i32 = 10160; //C0.0.14a
pub const J_C29: i32 = 10780; //c0.29_02
pub const J_C30: i32 = 10810; //c0.30c-launcher

pub const JS_E620: i32 = 700; //cjs-e6201baab01dbc98b4ad

pub const FOURK_1: i32 = 10; //4k-0217 (Modded)
pub const FOURK_2: i32 = 20; //4k-0144 (Modded)
pub const FOURK_JS: i32 = 1000; //4k-js (Modded)

#[derive(Clone)]
pub struct Samvid {
    pub id: i32,
    pub display: String
}

#[derive(Clone)]
pub struct Edition {
    pub id: String,
    pub display: String,
    pub versions: Vec<Samvid>
}

pub fn get () -> Result<Vec<Edition>,Error> {
    let path: PathBuf = resources::HASHES.get().unwrap()[&Resource::Samvid].path.clone();

    if !path.exists() { return Err(Error::from(ErrorKind::NotFound)) }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    let mut editions: Vec<Edition> = Vec::new();
    let mut versions: Vec<Samvid> = Vec::new();
    let mut edition_display: String = String::default();
    let mut edition_id: String = String::default();


    for (index, line) in reader.records().enumerate() {
        let clean = line?;
        let identifier = clean.get(0).unwrap();
        let display = clean.get(1).unwrap();
        let id = clean.get(2).unwrap();

        match identifier {
            "edition" => {
                if is_valid(edition_id.clone(), None) && versions.len() > 0 {
                    editions.push ( Edition {
                        id: edition_id,
                        display: edition_display,
                        versions: versions
                    });
                } else if index != 0 {
                    log(1, format!("Current version of the converter does not support any versions for {} - skipping", edition_display.clone()));
                }
                edition_display = display.to_string();
                edition_id = id.to_string();
                versions = Vec::new();
                log(0,format!("Unpacking Edition: {}",display));
            },
            "version" => {
                let i: i32 = match id.parse() {
                    Ok(j) => j,
                    Err(_) => {
                        log(1,format!("Unable to read samvid \"{}\" of {} - Skipping",id,display));
                        continue;
                    }
                };
                if is_valid(edition_id.clone(), Some(i)) {
                    versions.push(
                        Samvid { 
                            id: i, 
                            display: display.to_string()
                        });
                } else { 
                    log(1, format!("Current version of the converter does not support {} {} - skipping", edition_display.clone(), display)) 
                }
            }
            _ => {
                log(1,format!("Unrecognized identifier found: {}",identifier));
                log(1,format!("Samvid file may be broken..."));
            }
        }
    }

    if is_valid(edition_id.clone(), None) {
        editions.push(
            Edition {
                id: edition_id.clone(),
                display: edition_display.clone(),
                versions: versions.clone()
            });
    } else {
        log(1, format!("Current version of the converter does not support any versions for {} - skipping", edition_display.clone()));
    }

    Ok(editions)
}

fn is_valid (edition: String, version:Option<i32>) -> bool {
    match edition.as_str() {
        JAVA_EDITION => {
            if version.is_some() {
                if version.unwrap() < J_C14 {true}
                else {false}
            } else {true}
        },
        JAVASCRIPT_EDITION => true,
        FOURK_EDITION => true,
        _ => false
    }
}