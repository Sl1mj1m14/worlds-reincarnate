use jni::vm::{InitArgsBuilder, JavaVM};

use crate::{log::log, resources::{Manifest, download_from_manifest}};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Version {
    V8
}

pub fn launch (java_version: Version) -> Option<JavaVM> {

    let path = match download_from_manifest(Manifest::Java(java_version)) {
        Some(p) => p,
        None => {
            log(1, "Downloading java failed, unable to launch JVM!");
            return None
        }
    };

    let jvm_args = InitArgsBuilder::new().build().unwrap(); //Decide what arguments I want or something

    match JavaVM::with_libjvm(jvm_args, || Ok(path)) {
        Ok(jvm) => Some(jvm),
        Err(e) => {
            log(1, format!("{e}"));
            None
        }
    }

}