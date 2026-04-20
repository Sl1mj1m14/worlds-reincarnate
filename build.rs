fn main () {
    slint_build::compile("ui/main.slint").expect("Failed to compile slint ui");

    println!("cargo::rerun-if-changed=java");

    javac::Build::new()
        .source_dir("java")
        .output_dir("target/java")
        .target_version("8")
        //.release("8")     //Release field does not work with Java 8
        .compile();
}