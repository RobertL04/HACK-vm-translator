mod code_generator;
mod utils;
use code_generator::*;
use core::panic;
use std::{env, path::Path};

pub enum PathType {
    Dir,
    File,
}

// TODO: generate helpful comments
fn main() {
    // syntax:
    // 1) ./program --dir directory_path => output directory.asm
    // 2) ./program --file file_path => output: file.asm
    // 3) ./program {--dir x or --file x} --debug or ./program --debug {--dir x or --file x}

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("[ERROR] incorrect arguments."); // TODO: print help!
        panic!();
    }

    let mut is_debug_option = false;
    if args.contains(&"--debug".to_string()) {
        is_debug_option = true;
    }

    let is_dir_option = args.contains(&"--dir".to_string());
    let is_file_option = args.contains(&"--file".to_string());

    let path_str: String;

    let path_type: PathType;
    // check if file path exists and has vm extension:
    if !(is_dir_option ^ is_file_option) {
        eprintln!("[ERROR] incorrect arguments.");
        // TODO: print help
        panic!();
    } else {
        if is_dir_option {
            let option_index = args.iter().position(|i| *i == "--dir").unwrap();
            let dir_index = option_index + 1;
            path_str = args[dir_index].clone();
            path_type = PathType::Dir;
        } else {
            // file_option
            let option_index = args.iter().position(|i| *i == "--file").unwrap();
            let file_index = option_index + 1;
            path_str = args[file_index].clone();
            path_type = PathType::File;
        }
    }

    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!(
            "Path {} does not exist or couldn't be parsed correctly.",
            path_str
        );
        // TODO: print help / usage
        panic!();
    }

    let mut code_gen = CodeGenerator::new(&path, &path_type, is_debug_option);
    code_gen.generate_code();
}
