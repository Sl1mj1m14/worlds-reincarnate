use std::{cell::RefCell, error::Error, path::PathBuf, process::exit, rc::Rc, sync::OnceLock};
use chrono::prelude::*;
use rfd;

use crate::functions::{Dir, get_general_dir};

mod read;
mod write;
mod convert;
mod generate;

mod log;
mod version;
mod world;
mod functions;

slint::include_modules!();

const DEBUG_FLAG: bool = true;

static TIMESTAMP: OnceLock<String> = OnceLock::new();
const DEFAULT_TIMESTAMP: &str = "19700101120000";

#[derive(Clone)]
struct MainHandler {
    input_edition: String,
    input_version: i32,
    path: PathBuf,
    output_edition: String,
    output_version: i32,
}

fn main () -> Result<(),Box<dyn Error>>{

    //Starting Log System
    let timestamp: String = Local::now().format("%Y%m%d%H%M%S").to_string();
    TIMESTAMP.set(timestamp).unwrap();

    log::start();
    log::log(0,format!("Session Started at {}",Local::now().format("%Y-%m-%d %H:%M:%S")));

    //Retrieving list of all versions
    let list = match version::get() {
        Ok(val) => val,
        Err(e) => {
            log::log(2,"Error parsing versions:");
            log::log(2,format!("{}",e));
            log::close();
            exit(1);
        }
    };

    //Setting defaults for conversion
    let main_handler = Rc::new(RefCell::new(MainHandler { 
        input_edition: list[0].id.clone(), 
        input_version: list[0].versions[0].id, 
        path: PathBuf::default(), 
        output_edition: list[0].id.clone(), 
        output_version: list[0].versions[0].id 
    }));

    //Building window
    let ui: MainWindow = MainWindow::new()?;

    let mut ui_versions: Vec<Version> = Vec::new();

    for edition in list {
        //Add filters in settings in the future in order to allow preferred versions
        for version in edition.versions {
            log::log(-1,format!("Edition: {} Version: {} ID: {}",&edition.id,&version.display,version.id));
            ui_versions.push(Version { 
                edition: edition.id.clone().into(), 
                name: version.display.clone().into(), 
                samvid: version.id 
                });
        }
    }

    let version_model = Rc::new(slint::VecModel::from(ui_versions));

    ui.set_input_versions(version_model.clone().into());
    ui.set_output_versions(version_model.clone().into());

    //Handling setting versions
    let clone_handler = main_handler.clone();
    ui.on_set_version(move |code, edition, version| {
        match code {
            0 => {
                clone_handler.borrow_mut().input_edition = edition.to_string();
                clone_handler.borrow_mut().input_version = version;
            },
            1 => {
                clone_handler.borrow_mut().output_edition = edition.to_string();
                clone_handler.borrow_mut().output_version = version;
            },
            _ => ()
        }
    });

    //Handling opening a file
    let clone_handler = main_handler.clone();
    ui.on_browse(move ||{
        let edition = clone_handler.borrow_mut().input_edition.clone();
        let version = clone_handler.borrow_mut().input_version;

        let path = filter_files(edition, version);

        match path {
            Some(val) => {
                clone_handler.borrow_mut().path = val.clone();
                log::log(-1,format!("Opened {}",val.to_string_lossy()))
            },
            None => log::log(-1,"No file was opened!")
        };
    });

    //Handling converting
    let clone_handler = main_handler.clone();
    ui.on_convert(move ||{
        let handles = clone_handler.borrow_mut().clone();
        if !handles.path.clone().exists() {
            if handles.path.clone() == PathBuf::default() {
                log::log(0, "Please choose a file before converting")
            } else {
                log::log(2, format!("File at {} no longer exists",handles.path.clone().to_str().unwrap_or_default()))
            }
            return
        }

        let out_dir = get_general_dir(Dir::Documents);
        let output_path = match rfd::FileDialog::new().set_directory(out_dir).set_title("Save Folder").pick_folder() {
            Some(p) => p,
            None => {
                log::log(1,"Unable to convert without choosing output directory, returning");
                return
            }
        };

        convert::convert(handles.input_edition, handles.input_version, handles.path, handles.output_edition, handles.output_version, output_path);
    });

    ui.on_test(move ||{
        generate::javascript(42,128);
        generate::javascript(420,128);
    });

    //Handling when the program is closed
    ui.window().on_close_requested(||{
        log::close();
        slint::CloseRequestResponse::HideWindow
    });

    ui.run()?;
    Ok(())

}

fn filter_files (edition: String, version: i32) -> Option<PathBuf> {

    let mut dialog = rfd::FileDialog::new();

    dialog = match edition.as_str() {
        version::JAVA_EDITION => {
            if version <= version::J_PC16 {
                dialog.add_filter("PreClassic", &["dat"])
            } else if version <= version::J_C29 {
                dialog.add_filter("Classic", &["dat"])
            } else if version <= version::J_C30 {
                dialog.add_filter("Classic", &["dat","mine"])
            } else {
                log::log(1,"Searching for unknown or unsupported version!");
                rfd::FileDialog::new()
            }
        },
        _ => {
            log::log(1,"Searching for unknown or unsupported edition!");
            rfd::FileDialog::new()
        }
    };

    dialog = dialog.add_filter("Any", &["*"]);

    dialog.pick_file()

}



/*
Blocks
Entities
Items
Player
World Settings





*/



