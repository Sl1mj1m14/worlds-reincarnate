use std::{collections::HashMap, path::PathBuf};

use csv::StringRecord;

use crate::{log::log, resources::{self, Map, Resource}};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Data {
    pub id: String,
    pub ktype: String,
}

pub fn create_map(input_edition: String, input_version: i32, output_edition: String, output_version: i32) -> Option<HashMap<Data, Data>> {
    let path: PathBuf = resources::HASHES.get().unwrap()[&Resource::Map(Map::WorldData)].path.clone();

    if !path.exists() {
        log(2, "World data map not found, unable to convert world!");
        return None
    }

    let mut reader = match csv::ReaderBuilder::new().has_headers(false).from_path(path) {
        Ok(r) => r,
        Err(e) => {
            log(2, "Failed to parse world data map, unable to convert world!");
            log(2, format!("{e}"));
            return None
        }
    };

    let mut map: HashMap<Data, Data> = HashMap::new();

    let mut input_indices: Vec<usize> = Vec::new();
    let mut output_indices: Vec<usize> = Vec::new();

    for (index, result) in reader.records().enumerate() {
        let line = match result {
            Ok(r) => r,
            Err(_) => {
                log(1, "Invalid record when building world data map - skipping");
                continue
            }
        };

        if index == 0 {
            let mut i = 1; //Start index at 1, Column 0 is used for readable format
            let mut current = String::default();
            while i < line.iter().count() {
                log(-1, line.get(i).unwrap());
                if line.get(i).unwrap() != String::default() { current = line.get(i).unwrap().to_string(); }
                if current == input_edition.clone() { input_indices.push(i) }
                if current == output_edition.clone() { output_indices.push(i) }
                i += 4; //Each grouping accounts for 4 entries - start version, end version, id, type
            }
            continue
        }

        let key = match record_to_block(input_indices.clone(), line.clone(), input_version) {
            Some(s) => s,
            None => continue
        };
        log(-1, format!("Key: {:?}", key));

        let value = match record_to_block(output_indices.clone(), line.clone(), output_version) {
            Some(s) => s,
            None => continue
        };
        log(-1, format!("Value: {:?}", value));

        map.insert(key, value);
    }

    for (key, value) in map.clone() {
        log(-1, format!("Key: {:?}", key));
        log(-1, format!("Value: {:?}", value));
    }

    Some(map)
}

fn record_to_block (indices: Vec<usize>, record: StringRecord, version: i32) -> Option<Data> {
    for i in indices.clone() {
        let lower: i32 = record.get(i).unwrap().parse().unwrap_or(version + 1);
        let upper: i32 = record.get(i+1).unwrap().parse().unwrap_or(0);

        if version < lower || version > upper { continue }

        let id = match record.get(i+2).unwrap() {
            "" => continue,
            val => val
        };

        let dtype = match record.get(i+3).unwrap() {
            "" => continue,
            val => val
        };

        return Some(Data {id: id.to_string(), ktype: dtype.to_string()});

    }
    return None
}