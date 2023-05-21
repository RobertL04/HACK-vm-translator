use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::PathType;

// collection of useful methods

pub fn write_to_file(file_pointer: &mut File, code_buffer_pointer: &mut Vec<String>) {
    let code_block_as_str = code_buffer_pointer.join("\n");
    file_pointer
        .write_all(&code_block_as_str.as_bytes())
        .unwrap();
    file_pointer.write_all("\n".as_bytes()).unwrap();
    code_buffer_pointer.clear();
}

//  allows user to create seperate asm file that includes comments but cannot be executed using the CPU emulator included in the NAND To Tetris software suite.
pub fn create_output_file(path: &Path, path_type: &PathType, debug_option: bool) -> String {
    let output_path: String = match path_type {
        PathType::File => {
            if debug_option {
                format!(
                    "{}.debug.asm",
                    path.to_str().unwrap().trim_end_matches(".vm")
                )
            } else {
                format!("{}.asm", path.to_str().unwrap().trim_end_matches(".vm"))
            }
        }
        PathType::Dir => {
            let mut file_name = format!(
                "{}.asm",
                path.to_path_buf().iter().last().unwrap().to_str().unwrap()
            );
            if debug_option {
                file_name = format!(
                    "{}.debug.asm",
                    path.to_path_buf().iter().last().unwrap().to_str().unwrap()
                );
            }
            let mut temp_path = path.to_path_buf().clone();
            temp_path.push(file_name);
            temp_path.as_path().to_str().unwrap().to_string()
        }
    };
    return output_path;
}

pub fn search_vm_files(dir: &Path, files_vec_pointer: &mut Vec<PathBuf>) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry_ = entry.unwrap();
            let path = entry_.path();
            if !path.is_dir() && path.to_str().unwrap().ends_with(".vm") {
                files_vec_pointer.push(path);
            }
        }
    }
}

pub fn add_padding(s: &String, n: usize) -> String {
    // naive padding (for now)
    let mut new_string: String = "".to_owned();
    for _ in 0..n {
        new_string.push(' ');
    }
    new_string.push_str(s);
    return new_string;
}

#[test]
fn it_works() {
    let result = add_padding(&"hello".to_string(), 4);
    assert_eq!(result, "    hello");
}
