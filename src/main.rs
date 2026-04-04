use std::{cell::RefCell, error::Error, path::PathBuf, process::exit, rc::Rc, sync::{Arc, Mutex, OnceLock}, thread, time::Duration};
use chrono::prelude::*;
use rfd;
use slint::SharedString;

mod log;
mod functions;
mod file;

mod version;
mod world;
mod convert;

use crate::{file::Argument, file::JSFormat, version::Edition};

slint::include_modules!();

const DEBUG_FLAG: bool = true;

static TIMESTAMP: OnceLock<String> = OnceLock::new();
const DEFAULT_TIMESTAMP: &str = "19700101120000";

#[derive(Clone, Default)]
struct Handler {
    edition: String,
    version: i32,
    path: PathBuf,
    args: Option<Vec<Argument>>,
}

fn main () -> Result<(),Box<dyn Error>>{

    //Starting Log System
    let timestamp: String = Local::now().format("%Y%m%d%H%M%S").to_string();
    TIMESTAMP.set(timestamp).unwrap();

    log::start();
    log::log(0,format!("Session Started at {}",Local::now().format("%Y-%m-%d %H:%M:%S")));

    //Creating handlers for conversion
    let input_handler: Rc<RefCell<Handler>> = Rc::new(RefCell::new(Handler::default()));
    let output_handler: Rc<RefCell<Handler>> = Rc::new(RefCell::new(Handler::default()));
    let all_editions: Arc<Mutex<Vec<Edition>>> = Arc::new(Mutex::new(Vec::new()));

    //Creating window
    let ui: MainWindow = MainWindow::new()?;
    ui.set_state(State::Load);

    let ui_weak = ui.as_weak();
    let clone_editions = all_editions.clone();
    thread::scope(|s| {
        s.spawn(|| {

            let mut unlocked_editions = clone_editions.lock().unwrap();

            //Retrieving list of all versions
            *unlocked_editions = match version::get() {
                Ok(val) => val,
                Err(e) => {
                    log::log(2,"Error parsing versions:");
                    log::log(2,format!("{}",e));
                    log::close();
                    exit(1);
                }
            };

            if (*unlocked_editions).len() <= 0 {
                log::log(2, "No versions found, unable to run!");
                log::close();
                exit(1);
            }

            //Building lists and menus for the ui
            let mut ui_edition_list: Vec<SharedString> = Vec::new();
            for edition in (*unlocked_editions).clone() { ui_edition_list.push(edition.display.into()); }

            let mut version_list: Vec<Version> = Vec::new();
            for version in (*unlocked_editions).clone()[0].versions.clone() { version_list.push(Version { 
                display: version.display.into(), 
                value: version.id
            });}

            let mut js_format_list: Vec<SharedString> = Vec::new();
            for format in file::JS_FORMATS { js_format_list.push((*format).into())}
            let mut js_url_list: Vec<SharedString> = Vec::new();
            for url in file::JS_URLS { js_url_list.push((*url).into())}

            slint::invoke_from_event_loop(move || {
                let ui = ui_weak.unwrap();

                //Setting ui constants
                ui.global::<Constants>().set_js_value(version::JAVASCRIPT_EDITION.into());

                //Setting list of editions & versions for the ui
                ui.global::<Versions>().set_editions(Rc::new(slint::VecModel::from(ui_edition_list)).into());
                ui.global::<Versions>().set_input_version_list(Rc::new(slint::VecModel::from(version_list.clone())).into());
                ui.global::<Versions>().set_output_version_list(Rc::new(slint::VecModel::from(version_list.clone())).into());

                //Setting sub lists
                ui.global::<Versions>().set_js_format_list(Rc::new(slint::VecModel::from(js_format_list)).into());
                ui.global::<Versions>().set_js_url_list(Rc::new(slint::VecModel::from(js_url_list)).into());

                ui.set_state(State::Convert);
            })
        });

        //Handling updating edition
        let clone_input = input_handler.clone();
        let clone_output = output_handler.clone();
        let clone_editions = all_editions.clone();
        let ui_weak = ui.as_weak();

        ui.global::<Versions>().on_set_edition(move |code, e_index| {
            log::log(-1, "Attempting to change edition");
            let ui = ui_weak.unwrap();
            let clone_editions = clone_editions.lock().unwrap();

            let i = ((e_index.abs() + e_index)/2) as usize; //Index can return -1, this just changes it to 0
            let id = (*clone_editions)[i].id.clone();

            if (code == 0 && clone_input.borrow_mut().edition == id) || (code == 1 && clone_output.borrow_mut().edition == id) || code > 1 {return}
            
            let mut version_list: Vec<Version> = Vec::new();
            for version in (*clone_editions).clone()[i].versions.clone() { version_list.push(Version { 
                display: version.display.into(), 
                value: version.id
            });}

            match code {
                0 => {
                    if clone_input.borrow_mut().edition == (*clone_editions)[i].id.clone() {return}
                    clone_input.replace(Handler::default());
                    clone_input.borrow_mut().edition = (*clone_editions)[i].id.clone();

                    ui.global::<Versions>().set_input_version_list(Rc::new(slint::VecModel::from(version_list.clone())).into());

                    //Handling arguments
                    if id == version::JAVASCRIPT_EDITION {
                        clone_input.borrow_mut().args = Some(vec![Argument::JSFormat(JSFormat::Raw)]);
                        ui.global::<Versions>().set_input_state(version::JAVASCRIPT_EDITION.into());
                    } else {ui.global::<Versions>().set_input_state(SharedString::default());}

                },
                1 => {
                    if clone_output.borrow_mut().edition == (*clone_editions)[i].id.clone() {return}
                    clone_output.replace(Handler::default());
                    clone_output.borrow_mut().edition = (*clone_editions)[i].id.clone();

                    ui.global::<Versions>().set_output_version_list(Rc::new(slint::VecModel::from(version_list.clone())).into());

                    //Handling arguments
                    if id == version::JAVASCRIPT_EDITION {
                        clone_output.borrow_mut().args = Some(vec![Argument::JSFormat(JSFormat::Raw)]);
                        ui.global::<Versions>().set_output_state(version::JAVASCRIPT_EDITION.into());
                    } else {ui.global::<Versions>().set_output_state(SharedString::default());}
                },
                _ => return
            }
        });

        //Handling updating version
        let clone_input = input_handler.clone();
        let clone_output = output_handler.clone();
        let clone_editions = all_editions.clone();
        let ui_weak = ui.as_weak();
        ui.global::<Versions>().on_set_version(move |code, e_index, version| {
            log::log(-1, "Attempting to change version");
            let _ui = ui_weak.unwrap(); //Will be used in the future with certain states
            let clone_editions = clone_editions.lock().unwrap();

            let i = ((e_index.abs() + e_index)/2) as usize; //Index can return -1, this just changes it to 0
            let edition = (*clone_editions)[i].id.clone();

            match code {
                0 => {
                    clone_input.borrow_mut().version = version;
                    clone_input.borrow_mut().edition = edition;
                },
                1 => {
                    clone_output.borrow_mut().version = version;
                    clone_output.borrow_mut().edition = edition;
                },
                _ => return
            }
        });

        ui.run().unwrap();
    });


    //Handling opening a file
    /*let clone_handler = main_handler.clone();
    ui.on_browse(move ||{
        let edition = clone_handler.borrow_mut().input_edition.clone();
        let version = clone_handler.borrow_mut().input_version;

        let path = file::filter_files(edition, version);

        match path {
            Some(val) => {
                clone_handler.borrow_mut().path = val.clone();
                log::log(-1,format!("Opened {}",val.to_string_lossy()))
            },
            None => log::log(-1,"No file was opened!")
        };
    });*/

    //Handling converting
    /*let clone_handler = main_handler.clone();
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

        let out_dir = file::get_general_dir(file::Dir::Documents);
        let output_path = match rfd::FileDialog::new().set_directory(out_dir).set_title("Save Folder").pick_folder() {
            Some(p) => p,
            None => {
                log::log(1,"Unable to convert without choosing output directory, returning");
                return
            }
        };

        convert::convert(handles.input_edition, handles.input_version, handles.path, handles.output_edition, handles.output_version, output_path);
    });*/

    //Handling when the program is closed
    ui.window().on_close_requested(||{
        log::close();
        slint::CloseRequestResponse::HideWindow
    });

    Ok(())

}
