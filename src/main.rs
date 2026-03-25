use std::{cell::RefCell, error::Error, path::PathBuf, process::exit, rc::Rc, sync::OnceLock};
use chrono::prelude::*;

//mod read;
mod write;
//mod convert;

mod log;
mod version;
mod world;

slint::include_modules!();

const DEBUG_FLAG: bool = true;

static TIMESTAMP: OnceLock<String> = OnceLock::new();
const DEFAULT_TIMESTAMP: &str = "19700101120000";


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
            exit(0);
        }
    };

    //Setting defaults for conversion
    let input_edition = Rc::new(RefCell::new(list[0].id.clone()));
    let input_version = Rc::new(RefCell::new(list[0].versions[0].id));
    let mut output_edition = list[0].id.clone();
    let mut output_version = list[0].versions[0].id;
    let mut file_path = PathBuf::default();

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

    let clone_edition = input_edition.clone();
    let clone_version = input_version.clone();
    //Handling setting versions
    ui.on_set_version(move |code, edition, version| {
        match code {
            0 => {
                *clone_edition.borrow_mut() = edition.to_string();
                *clone_version.borrow_mut() = version;
            },
            1 => {
                //output_edition = edition.to_string();
                //output_version = version
            },
            _ => ()
        }
    });

    let clone_edition = input_edition.clone();
    let clone_version = input_version.clone();
    ui.on_browse(move ||{
        log::log(-1,format!("Edition is {} and version is {}",*clone_edition.borrow_mut(),*clone_version.borrow_mut()))
    });

    //Handling when the program is closed
    ui.window().on_close_requested(||{
        log::close();
        slint::CloseRequestResponse::HideWindow
    });

    ui.run()?;
    Ok(())

/*
    let level = read::read_classic_file(PathBuf::from("input/level.dat"));

    match level {
        Ok(value) => {
            println!("I guess this worked?");
            let mut name = String::from("");
            if value.world_data.is_some() {
                //if value.world_data.unwrap().
            }
            println!("World Name is: ")
        },
        Err(error) => eprintln!("{error}")
    }

    */
    //mem::drop(session_lock);
}



/*
Blocks
Entities
Items
Player
World Settings





*/



