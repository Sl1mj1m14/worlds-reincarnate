use std::fmt::Error;
use directories;

use main;

pub fn append_line (variant: u8, line: String) {
    
    if let Some(cache) = directories::ProjectDirs::from(main::QUALIFIER, main::ORGANIZATION, main::APPLICATION) {
        cache.cache_dir()

    }
    if main::DEBUG_FLAG {

    }
}