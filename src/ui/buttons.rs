use eframe::egui::WidgetText;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Button Failed")]
    Generic()
}

pub trait Button {
    fn action (&self) -> Result<(),ActionError>;
    fn settings (&self) -> Settings;
}

pub struct Settings {
    pub name: WidgetText,
    pub shortcut: Option<WidgetText>,
    pub enabled: bool,
}

/******************************
 *       File Section
******************************/
pub struct Open {}
impl Open { pub fn new() -> Self { Open {}}}
impl Button for Open {

    fn action (&self) -> Result<(),ActionError> {
        //To Do - Open A File
        Ok(())
    }

    fn settings (&self) -> Settings {
        Settings {
            name: "Open".into(),
            shortcut: None,
            enabled: false
        }
    }
}

/******************************
 *       Settings Section
******************************/
pub struct Theme {}
impl Theme { pub fn new() -> Self { Theme {}}}
impl Button for Theme {
    fn action (&self) -> Result<(),ActionError> {
        //To Do - Pull Up Theme Window
        Ok(())
    }

    fn settings (&self) -> Settings {
        Settings {
            name: "Theme".into(),
            shortcut: None,
            enabled: false
        }
    }
}

/******************************
 *       Convert Section
******************************/
pub struct JS {}
impl JS { pub fn new() -> Self { JS {}}}
impl Button for JS {
    fn action (&self) -> Result<(),ActionError> {
        //To Do - Change to Convert
        Ok(())
    }

    fn settings (&self) -> Settings {
        Settings {
            name: "To Classic JS".into(),
            shortcut: None,
            enabled: false
        }
    }
}