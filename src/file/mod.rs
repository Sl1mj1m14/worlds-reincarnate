use std::path::PathBuf;

use crate::{log::log, version};

pub enum Dir {
    Home,
    Documents
}

#[derive(Clone)]
pub enum Argument {
    JSFormat,
    JSUrl
}

#[derive(Clone)]
pub enum JSFormat {
    Raw,
    Firefox,
    Chrome,
    Edge
}

#[derive(Clone)]
pub enum JSUrl {
    Classic,
    LocalHost(u16),
    Omniarchive,
    Other(String)
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

pub fn filter_files (edition: String, version: i32) -> Option<PathBuf> {

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
                log(1,"Searching for unknown or unsupported version!");
                rfd::FileDialog::new()
            }
        },
        _ => {
            log(1,"Searching for unknown or unsupported edition!");
            rfd::FileDialog::new()
        }
    };

    dialog = dialog.add_filter("Any", &["*"]);

    dialog.pick_file()

}