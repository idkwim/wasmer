//! This file will run at build time to autogenerate the Emscripten tests
//! It will compile the files indicated in TESTS, to:executable and .wasm
//! - Compile using cc and get the output from it (expected output)
//! - Compile using emcc and get the .wasm from it (wasm)
//! - Generate the test that will compare the output of running the .wasm file
//!   with wasmer with the expected output
use glob::glob;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

static BANNER: &str = "// Rust test file autogenerated with cargo build (build/emtests.rs).
// Please do NOT modify it by hand, as it will be reseted on next build.\n";

const EXTENSIONS: [&str; 2] = ["c", "cpp"];
const EXCLUDES: [&str; 0] = [];

pub fn compile(file: &str, ignores: &Vec<String>) -> Option<String> {
    let mut output_path = PathBuf::from(file);
    output_path.set_extension("out");
    //    let output_str = output_path.to_str().unwrap();

    // Compile to .out
    //    Command::new("cc")
    //        .arg(file)
    //        .arg("-o")
    //        .arg(output_str)
    //        .output()
    //        .expect("failed to execute process");

    // Get the result of .out
    //    let output = Command::new(output_str)
    //        .arg(output_str)
    //        .output()
    //        .expect("failed to execute process");

    // Remove executable
    //    fs::remove_file(output_str).unwrap();

    let mut output_path = PathBuf::from(file);
    output_path.set_extension("js");
    let output_str = output_path.to_str().unwrap();

    // Compile to wasm
    let _wasm_compilation = Command::new("emcc")
        .arg(file)
        .arg("-s")
        .arg("WASM=1")
        .arg("-o")
        .arg(output_str)
        .output()
        .expect("failed to execute process");

    // panic!("{:?}", wasm_compilation);
    // if output.stderr {
    //     panic!("{}", output.stderr);
    // }
    // Remove js file

    if Path::new(output_str).is_file() {
        fs::remove_file(output_str).unwrap();
    } else {
        println!("Output JS not found: {}", output_str);
    }

    let mut output_path = PathBuf::from(file);
    output_path.set_extension("output");
    let module_name = output_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    //
    //    let output_str = output_path.to_str().unwrap();

    // Write the output to file
    //    fs::write(output_str, output.stdout).expect("Unable to write file");

    let rs_module_name = module_name.to_lowercase();
    let rust_test_filepath = format!(
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/emtests/{}.rs"),
        rs_module_name.as_str()
    );

    let output_extension = if file.ends_with("c") || module_name.starts_with("test_") {
        "out"
    } else {
        "txt"
    };

    let ignored = if ignores
        .iter()
        .any(|i| &i.to_lowercase() == &module_name.to_lowercase())
    {
        "\n#[ignore]"
    } else {
        ""
    };

    let module_path = format!("emtests/{}.wasm", module_name);
    let test_output_path = format!("emtests/{}.{}", module_name, output_extension);
    if !Path::new(&module_path).is_file() {
        println!("Path not found to test module: {}", module_path);
        None
    } else if !Path::new(&test_output_path).is_file() {
        println!("Path not found to test output: {}", module_path);
        None
    } else {
        let contents = format!(
            "#[test]{ignore}
fn test_{rs_module_name}() {{
    assert_emscripten_output!(
        \"../../{module_path}\",
        \"{rs_module_name}\",
        vec![],
        \"../../{test_output_path}\"
    );
}}
",
            ignore = ignored,
            module_path = module_path,
            rs_module_name = rs_module_name,
            test_output_path = test_output_path
        );

        fs::write(&rust_test_filepath, contents.as_bytes()).unwrap();

        Some(rs_module_name)
    }
    // panic!("OUTPUT: {:?}", output);
}

pub fn build() {
    let rust_test_modpath = concat!(env!("CARGO_MANIFEST_DIR"), "/src/emtests/mod.rs");

    let mut modules: Vec<String> = Vec::new();
    // modules.reserve_exact(TESTS.len());

    let ignores = read_ignore_list();

    for ext in EXTENSIONS.iter() {
        for entry in glob(&format!("emtests/*.{}", ext)).unwrap() {
            match entry {
                Ok(path) => {
                    let test = path.to_str().unwrap();
                    if !EXCLUDES.iter().any(|e| test.ends_with(e)) {
                        if let Some(module_name) = compile(test, &ignores) {
                            modules.push(module_name);
                        }
                    }
                }
                Err(e) => println!("{:?}", e),
            }
        }
    }
    modules.sort();
    let mut modules: Vec<String> = modules.iter().map(|m| format!("mod {};", m)).collect();
    assert!(modules.len() > 0, "Expected > 0 modules found");

    modules.insert(0, BANNER.to_string());
    modules.insert(1, "// The _common module is not autogenerated, as it provides common macros for the emtests\n#[macro_use]\nmod _common;".to_string());
    // We add an empty line
    modules.push("".to_string());

    let modfile: String = modules.join("\n");
    let source = fs::read(&rust_test_modpath).unwrap();
    // We only modify the mod file if has changed
    if source != modfile.as_bytes() {
        fs::write(&rust_test_modpath, modfile.as_bytes()).unwrap();
    }
}

fn read_ignore_list() -> Vec<String> {
    let f = File::open("emtests/ignores.txt").unwrap();
    let f = BufReader::new(f);
    f.lines().filter_map(Result::ok).collect()
}
