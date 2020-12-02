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
use std::path::Path;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

////////////////////////////////////////////////////////////////////////////////
//  Code
////////////////////////////////////////////////////////////////////////////////
const macro_file_name: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates/macros.html.tera");

fn main() {
    //
    // Loop over all files in the template dir
    // Replace any static macros with the static macro
    //
    let template_dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates"));
    for file_name_res in template_dir.read_dir().unwrap() {
        let file_name = file_name_res.unwrap();
        let file = File::open(file_name.path()).expect(&format!("Could not open file: {:?}", file_name));
        let in_buf = BufReader::new(&file);
        let tmp_file_name = "/tmp/template.build".to_owned();
        let tmp_file = File::create(&tmp_file_name).expect("Failed to create tmp scrub file");
        let mut out_buf = BufWriter::new(tmp_file);
        for line_res in in_buf.lines() {
            let mut line = line_res.unwrap();
            if line.contains("macros::static_") {
                // Get the static macro
                let macro_name: Vec<&str> = line.split("macros::static_").collect();
                let macro_name = macro_name[1];
                let macro_name: Vec<&str> = macro_name.split("()").collect();
                let macro_name = macro_name[0];
                line = get_static_macro(macro_name);
            }
            out_buf.write_all(line.as_bytes());
        }

        // Replace original with scrubbed file
        // std::fs::rename(std::path::Path::new(&tmp_file_name),
        //                 std::path::Path::new(&file_name))
        //      .expect("Failed to overwrite file");
        // If all static macros, move it to the static build dir
        // otherwise move it to the templates build dir
    }

    // Move all static files to the build dir
    let dir = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates"));
    let static_dir = Path::read_dir(dir);
    for file in static_dir {

    }

    // Send info to Cargo
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=templates");
    println!("cargo:rerun-if-changed=static");
}


enum GetMacroState {
    SearchingForMacro,
    GrabbingMacro
}

fn get_static_macro(macro_name: &str) -> String {
    let file = File::open(macro_file_name).expect(&format!("Could not open file: {:?}", macro_file_name));
    let in_buf = BufReader::new(&file);

    let mut ret = String::new();
    let mut state = GetMacroState::SearchingForMacro;
    for line_res in in_buf.lines() {
        let line = line_res.unwrap();
        match state {
            GetMacroState::SearchingForMacro => {
                let test_line = String::from("{% macro static_") + macro_name +"() %}";
                if line.contains(&test_line) {
                    state = GetMacroState::GrabbingMacro;
                }
            },

            GetMacroState::GrabbingMacro => {
                let test_line = String::from("{% endmacro static_") + macro_name + "%}";
                if line.contains(&test_line) {
                    return ret;
                }
                ret.push_str(&line);
            },
        }
    }
    return ret;
}
