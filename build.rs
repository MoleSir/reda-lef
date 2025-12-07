use std::path::Path;
use std::process::Command;

fn main() {
    let lef_source_dir = Path::new("third_party/si2-lef/lef");
    let clef_source_dir = Path::new("third_party/si2-lef/clef");

    let lef_y = lef_source_dir.join("lef.y");
    let lef_tab_cpp = lef_source_dir.join("lef.tab.cpp");
    // let lef_tab_hpp = lef_source_dir.join("lef.tab.hpp");

    // 1. run bison 
    let status = Command::new("bison")
        .args(&[
            "-v",
            "-p", "lefyy",
            "-d",
            lef_y.to_str().unwrap(),
            "-o",
            lef_tab_cpp.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run bison");
    if !status.success() {
        panic!("bison returned an error");
    }

    // 2. collect all .cpp
    let mut cpp_files = vec![lef_tab_cpp.clone()];
    for entry in std::fs::read_dir(lef_source_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|x| x.to_str()) == Some("cpp")
            && path.file_name() != Some(std::ffi::OsStr::new("lef.tab.cpp"))
        {
            cpp_files.push(path);
        }
    }
    for entry in std::fs::read_dir(clef_source_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|x| x.to_str()) == Some("cpp")
            && path.file_name() != Some(std::ffi::OsStr::new("lef.tab.cpp"))
        {
            cpp_files.push(path);
        }
    }

    // 3. compile to lib
    let mut build = cc::Build::new();
    build.cpp(true);
    build.flag_if_supported("-w");
    build.flag("-std=c++17");
    build.include(lef_source_dir);

    for file in &cpp_files {
        build.file(file);
    }

    build.compile("si2_lef");

    println!("cargo:rustc-link-lib=static=si2_lef");
}
