use std::path::PathBuf;

use enum_iterator::Sequence;

use crate::{log::log, version::{self, JAVASCRIPT_EDITION}};

pub static JS_FORMATS: &[&str] = &[
    "Raw (Json)",
    "Firefox",
    "Chrome",
    "Edge"
];

pub static JS_URLS: &[&str] = &[
    "classic.minecraft.net",
    "omniarchive.uk/23a-js/",
    "localhost",
    "Other"
];

pub enum Dir {
    Home,
    Documents
}

#[derive(Clone)]
pub enum Argument {
    JSFormat(JSFormat),
    JSUrl(JSUrl)
}

#[derive(Clone, Sequence)]
pub enum JSFormat {
    Raw,
    Firefox,
    Chrome,
    Edge
}

#[derive(Clone)]
pub enum JSUrl {
    Classic,
    Omniarchive,
    LocalHost(u16),
    Other(String)
}

impl Sequence for JSUrl {
    const CARDINALITY: usize = 4;

    fn first() -> Option<Self> {
        Some(JSUrl::Classic)
    }

    fn last() -> Option<Self> {
        Some(JSUrl::Other(String::default()))
    }

    fn next(&self) -> Option<Self> {
        match self {
            Self::Classic => Some(JSUrl::Omniarchive),
            Self::Omniarchive => Some(JSUrl::LocalHost(0)),
            Self::LocalHost(_) => Some(JSUrl::Other(String::default())),
            Self::Other(_) => None
        }
    }

    fn previous(&self) -> Option<Self> {
        match self {
            Self::Classic => None,
            Self::Omniarchive => Some(JSUrl::Classic),
            Self::LocalHost(_) => Some(JSUrl::Omniarchive),
            Self::Other(_) => Some(JSUrl::LocalHost(0))
        }
    }
}

pub fn get_general_dir(dir: Dir) -> PathBuf {
    match dir {
        Dir::Home => {
            let u = directories::UserDirs::new().unwrap();
            u.home_dir().to_path_buf()
        },
        Dir::Documents => {
            let u = directories::UserDirs::new().unwrap();
            u.document_dir().unwrap().to_path_buf()
        }
    }
}

pub fn filter_files (edition: String, version: i32, args: Option<Vec<Argument>>) -> Option<PathBuf> {

    let mut dialog = rfd::FileDialog::new();
    dialog = dialog.set_directory(get_general_dir(Dir::Documents));

    let mut is_file = false;

    dialog = match edition.as_str() {
        version::JAVA_EDITION => {
            if version <= version::J_PC16 {
                is_file = true;
                dialog.add_filter("PreClassic", &["dat"])
            } else if version <= version::J_C29 {
                is_file = true;
                dialog.add_filter("Classic", &["dat"])
            } else if version <= version::J_C30 {
                is_file = true;
                dialog.add_filter("Classic", &["dat","mine"])
            } else {
                log(1,"Searching for unknown or unsupported version!");
                dialog
            }
        },
        version::JAVASCRIPT_EDITION => {
            let mut mode = JSFormat::Raw;
            if args.is_some() {
                for arg in args.unwrap() {
                    match arg {
                        Argument::JSFormat(f) => mode = f,
                        _ => ()
                    }
                }
            };

            match mode {
                JSFormat::Raw => {
                    is_file = true;
                    dialog.add_filter("Classic JS (Raw)", &["json"])
                },
                _ => dialog
            }
        },
        _ => {
            log(1,"Searching for unknown or unsupported edition!");
            dialog
        }
    };

    if is_file {
        dialog = dialog.add_filter("Any", &["*"]);
        dialog = dialog.set_title("Choose File");
        dialog.pick_file()
    } else {
        dialog = dialog.set_title("Choose Folder");
        dialog.pick_folder()
    }
    

}