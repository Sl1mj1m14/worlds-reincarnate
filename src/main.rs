use std::{cell::RefCell, error::Error, path::PathBuf, process::exit, rc::Rc, sync::{Arc, Mutex, OnceLock}, thread};
use chrono::prelude::*;
use enum_iterator::{all};
use rfd;
use slint::{Model, SharedString};

mod log;
mod functions;
mod file;

mod version;
mod world;
mod convert;

use crate::{file::*, version::Edition};

slint::include_modules!();

const DEBUG_FLAG: bool = true;

static TIMESTAMP: OnceLock<String> = OnceLock::new();
const DEFAULT_TIMESTAMP: &str = "19700101120000";

#[derive(Clone, Default)]
pub struct Handler {
    pub edition: String,
    pub version: i32,
    pub path: PathBuf,
    pub args: Option<Vec<Argument>>,
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
    ui.global::<UIState>().set_state(State::Load);

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

                ui.global::<UIState>().set_state(State::Convert);
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

        //Handling updating args
        let clone_input = input_handler.clone();
        let clone_output = output_handler.clone();

        ui.global::<Versions>().on_update_args(move |code, state, args| {
            log::log(-1, "Attempting to update args");
            let mut arg_arr: Vec<Argument> = Vec::new();
            
            match state.as_str() {
                version::JAVASCRIPT_EDITION => 'js: {

                    let args: Vec<i32> = args.iter().collect();
                    let len = args.len();
                    let mut i = if len >= 1 {args[0] as usize} else {0};
                    let mut j = if len >= 2 {args[1] as usize} else {0};

                    let formats = all::<JSFormat>().collect::<Vec<_>>();
                    if i >= formats.len() {i = 0};
                    arg_arr.push(Argument::JSFormat(formats[i].clone()));

                    if i == 0 {break 'js;} //0 should always be raw json, so a url doesn't matter
                    let urls = all::<JSUrl>().collect::<Vec<_>>();
                    if j >= urls.len() {j = 0};
                    arg_arr.push(Argument::JSUrl(urls[j].clone()));

                },
                _ => return
            }

            match code {
                0 => {
                    clone_input.borrow_mut().args = Some(arg_arr);
                },
                1 => {
                    clone_output.borrow_mut().args = Some(arg_arr);
                },
                _ => return
            }
        });

        //Handling browsing for a file
        let clone_input = input_handler.clone();

        ui.global::<Versions>().on_browse(move || {
            let edition = clone_input.borrow_mut().edition.clone();
            let version = clone_input.borrow_mut().version;
            let args = clone_input.borrow_mut().args.clone();

            if edition == String::default() || version == i32::default() {
                log::log(1, "Unable to browse for file until a version & edition are set!");
                return
            }

            match filter_files(edition, version, args) {
                Some(f) => {
                    clone_input.borrow_mut().path = f.clone();
                    log::log(-1,format!("Opened path at {}",f.to_string_lossy()))
                },
                None => log::log(0, "No file chosen")
            }
        });

        //Handling converting
        let clone_input = input_handler.clone();
        let clone_output = output_handler.clone();
        let ui_weak = ui.as_weak();

        ui.global::<Versions>().on_convert(move || {
            let ui = ui_weak.unwrap();

            if  clone_input.borrow_mut().edition.clone() == String::default() ||
                clone_output.borrow_mut().edition.clone() == String::default() ||
                clone_input.borrow_mut().version == i32::default() ||
                clone_output.borrow_mut().version == i32::default() ||
                clone_input.borrow_mut().path.clone() == PathBuf::default() {
                    log::log(1, "Unable to convert - not all fields are appropriately set");
                    ui.global::<UIState>().set_state(State::Convert); 
                    return
                }

            let dir = get_general_dir(Dir::Documents);
            let path = match rfd::FileDialog::new().set_directory(dir).set_title("Choose Output Folder").pick_folder() {
                Some(p) => p,
                None => {
                    log::log(0, "No output directory chosen - canceling converion");
                    ui.global::<UIState>().set_state(State::Convert); 
                    return
                }
            };

            clone_output.borrow_mut().path = path;
            //In certain cases, arguments may need to be updated...

            let _ = convert::convert(clone_input.borrow_mut().clone(), clone_output.borrow_mut().clone());

            ui.global::<UIState>().set_state(State::Convert);
        });

        //Handling when the program is closed
        ui.window().on_close_requested(||{
            log::close();
            slint::CloseRequestResponse::HideWindow
        });

        ui.run().unwrap();
    });

    Ok(())

}
