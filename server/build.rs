////////////////////////////////////////////////////////////////////////////////
//  File:   build.rs
//
//  © Zach Nielsen 2020
//  Pre build script for the Server module
//
////////////////////////////////////////////////////////////////////////////////
//  Included Modules
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
//  Namespaces
////////////////////////////////////////////////////////////////////////////////
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
const MACRO_FILE_NAME: &'static str = "macros.html.tera";
const MACRO_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/macros.html.tera");
const MACRO_IMPORT_STR: &'static str = "{% import \"macros\" as macros %}";

fn main() {
    // Make the dirs
    let out_dir = env::var("OUT_DIR").unwrap();
    let web_dir = out_dir.clone() + "/web";
    let template_dir_name = web_dir.clone() + "/templates/";
    let static_dir_name = web_dir.clone() + "/static/";
    std::fs::create_dir_all(&template_dir_name).expect("can't create template dir");
    std::fs::create_dir_all(&static_dir_name).expect("can't create static dir");

    // Make a symlink to the dirs
    let symlink = Path::new("./web");
    match symlink.symlink_metadata() {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                std::fs::remove_dir_all(&symlink).expect("can't remove symlink");
            }
        }
        Err(_) => (),
    }
    std::os::unix::fs::symlink(Path::new(&web_dir), &symlink).expect("can't create symlink");

    //
    // Loop over all files in the template dir
    // Replace any static macros with the static macro
    //
    let template_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates"));
    println!("template_dir: {:?}", template_dir);
    for file_name_res in template_dir.read_dir().unwrap() {
        let file_name = file_name_res.unwrap();
        // First things first - Tell Cargo to rerun if this file changes
        println!(
            "cargo:rerun-if-changed=templates/{}",
            file_name.file_name().to_str().unwrap()
        );

        if file_name.file_name() == MACRO_FILE_NAME {
            // Just copy over the macro file, don't do anything to it
            let target_file_name =
                template_dir_name.clone() + file_name.file_name().to_str().unwrap();
            let target_file = Path::new(&target_file_name);
            std::fs::copy(file_name.path(), target_file).expect("Could not copy static file");
            continue;
        }

        println!("opening file: {:?}", file_name);
        let mut all_static = true;
        let tmp_file_name = "/tmp/template.build".to_owned();
        // Scope to close file
        {
            let file = File::open(file_name.path())
                .expect(&format!("Could not open file: {:?}", file_name));
            let in_buf = BufReader::new(&file);
            let tmp_file = File::create(&tmp_file_name).expect("Failed to create tmp file");
            let mut out_buf = BufWriter::new(tmp_file);

            for line_res in in_buf.lines() {
                let mut line = line_res.unwrap();
                if line.contains("{{ macros::static_") {
                    // Get the static macro
                    let macro_name: Vec<&str> = line.split("macros::static_").collect();
                    let macro_name = macro_name[1];
                    let macro_name: Vec<&str> = macro_name.split("()").collect();
                    let macro_name = macro_name[0];
                    println!("about to get_static_macro: {}", macro_name);
                    line = get_static_macro(macro_name);
                } else if !line_is_static(&line) {
                    println!("Hit a non-static value: {}", &line);
                    all_static = false;
                }

                line.push_str("\n");
                out_buf
                    .write_all(line.as_bytes())
                    .expect("Error writing to out_buf");
            }
        }

        if all_static {
            // Remove the .tera
            let mut static_file = static_dir_name.clone() + file_name.file_name().to_str().unwrap();
            let iter = static_file.find(".tera").unwrap_or(static_file.len());
            static_file.drain(iter..);
            println!("static file is {}", static_file);
            let dest_file = Path::new(&static_file);

            // Scrub MACRO_IMPORT_STR here
            println!(
                "Writing {:?} out to {:?}, scrubbing the macro import",
                &tmp_file_name, &dest_file
            );
            let newly_static_file =
                File::open(&tmp_file_name).expect(&format!("Could not open file: {:?}", file_name));
            let new_in_buf = BufReader::new(&newly_static_file);
            let new_out_file =
                File::create(&dest_file).expect(&format!("Could not create file: {:?}", dest_file));
            let mut new_out_buf = BufWriter::new(new_out_file);
            for line_res in new_in_buf.lines() {
                let line = line_res.unwrap();
                // println!("line: {}", &line);
                let scrubbed_line = line.replace(MACRO_IMPORT_STR, "");
                // println!("replaced line: {}", &scrubbed_line);
                //if !line.contains("{% import") {
                new_out_buf
                    .write_all(scrubbed_line.as_bytes())
                    .expect("Error writing to new_out_buf");
                //}
            }
        } else {
            let template_file = template_dir_name.clone() + file_name.file_name().to_str().unwrap();
            let dest_file = Path::new(&template_file);
            println!("Moving {:?} to {:?}", tmp_file_name, dest_file);
            match std::fs::rename(Path::new(&tmp_file_name), dest_file) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error moving file: {}", e);
                    #[cfg(feature = "vendored-openssl")]
                    panic!("Copy error occured in non-vendored build");
                },
            }
        }
    }

    // Move all static files to the build dir
    let static_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static"));
    for file_res in static_dir.read_dir().unwrap() {
        let file = file_res.unwrap();
        // First things first - Tell Cargo to rerun if this file changes
        println!(
            "cargo:rerun-if-changed=static/{}",
            file.file_name().to_str().unwrap()
        );

        let target_file_name = static_dir_name.clone() + file.file_name().to_str().unwrap();
        let target_file = Path::new(&target_file_name);
        println!("Copying {:?} to {:?}", file, target_file);
        std::fs::copy(file.path(), target_file).expect("Could not copy static file");
    }

    // Send info to Cargo
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=static");
}

fn line_is_static(line: &str) -> bool {
    if line.contains(MACRO_IMPORT_STR) {
        // Importing the macros does not count as a variable line
        return true;
    }
    else if line.contains("{{") || line.contains("{%") {
        return false;
    }
    return true;
}

enum GetMacroState {
    SearchingForMacro,
    GrabbingMacro,
}

fn get_static_macro(macro_name: &str) -> String {
    let file = File::open(MACRO_FILE).expect(&format!("Could not open file: {:?}", MACRO_FILE));
    let in_buf = BufReader::new(&file);

    let mut ret = String::new();
    let mut state = GetMacroState::SearchingForMacro;
    for line_res in in_buf.lines() {
        let line = line_res.unwrap();
        match state {
            GetMacroState::SearchingForMacro => {
                let test_line = String::from("{% macro static_") + macro_name + "() %}";
                if line.contains(&test_line) {
                    state = GetMacroState::GrabbingMacro;
                    println!("Found the macro: {}", line);
                }
            }

            GetMacroState::GrabbingMacro => {
                let test_line = String::from("{% endmacro static_") + macro_name + " %}";
                if line.contains(&test_line) {
                    println!("macro over: {}", test_line);
                    return ret;
                }
                ret.push_str(&line);
                // println!("{}", line);
            }
        }
    }
    return ret;
}
