use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::PathType;

/// Writes the content of `code_buffer_ref` to the output file, then empties the buffer.
///
/// Expects the buffer to be made up of many assembly lines but does not check for correctness.
pub fn write_to_file(file_pointer: &mut File, code_buffer_ref: &mut Vec<String>) {
    code_buffer_ref.push("\n".to_string());
    let code_block_as_str = code_buffer_ref.join("\n");
    file_pointer
        .write_all(&code_block_as_str.as_bytes())
        .unwrap();
    code_buffer_ref.clear();
}

/// Constructs[^note] a suitable output path for the assembly file.
///
/// If `path_type` is `File`, the same path but with .asm extension is returned.
///
/// If `path_type` is `Dir`, the file is put inside the directory and has its name.
///
/// If `is_debug_option` is true, the extension is prefixed with `.debug`.
///
/// # Example:
/// ```
/// use std::path::Path;
/// use crate::PathType;
/// let path = Path::new(".../test_directory");
/// let output_path = create_output_path(&path, &PathType::Dir, true);
/// // output_path: .../test_directory/test_directory.debug.asm
/// ```
///
/// [^note]: the function returns a string to the file, but does not create the output file.
/// The caller should use the returned string in order to create the output file.
pub fn create_output_path(path: &Path, path_type: &PathType, is_debug_option: bool) -> String {
    let output_path: String = match path_type {
        PathType::File => {
            if is_debug_option {
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
            if is_debug_option {
                file_name = format!(
                    "{}.debug.asm",
                    path.to_path_buf().iter().last().unwrap().to_str().unwrap()
                );
            }
            let mut temp_path = path.to_path_buf();
            temp_path.push(file_name);
            temp_path.as_path().to_str().unwrap().to_string()
        }
    };
    return output_path;
}

/// Looks for all vm files in a given directory.
///
/// Prints warning if argument is not a directory.
pub fn search_vm_files(dir: &Path, files_vec_ref: &mut Vec<PathBuf>) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_dir() && path.to_str().unwrap().ends_with(".vm") {
                files_vec_ref.push(path);
            }
        }
    } else {
        println!(
            "{}",
            "[WARNING] in function search_vm_files: argument dir is not a directory".purple()
        )
    }
}

/// Adds `n` spaces at the beginning of the supplied string `s`.
///
/// # Example:
/// ```
/// let string_with_padding = add_padding(&String::from("hello"), 2);
/// assert_eq!("  hello", string_with_padding);
/// ```
pub fn add_padding(s: &String, n: usize) -> String {
    let mut new_string: String = "".to_owned();
    for _ in 0..n {
        new_string.push(' ');
    }
    new_string.push_str(s);
    return new_string;
}
