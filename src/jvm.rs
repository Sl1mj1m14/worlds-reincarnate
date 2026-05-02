use jni::vm::{InitArgsBuilder, JavaVM};

use crate::{jvm, log::log, resources::{Manifest, download_from_manifest}};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Version {
    V8
}

pub fn launch (java_version: Version, args: &[&str]) -> Option<JavaVM> {

    let Some(path) = download_from_manifest(Manifest::Java(java_version)) else {
        log(1, "Downloading java failed, unable to launch JVM!");
        return None
    };

    let mut builder = InitArgsBuilder::new();
    for arg in args {
        builder = builder.option(*arg);
    }

    let jvm_args = builder.build().unwrap();

    match JavaVM::with_libjvm(jvm_args, || Ok(path)) {
        Ok(jvm) => Some(jvm),
        Err(e) => {
            log(1, format!("{e}"));
            None
        }
    }

}