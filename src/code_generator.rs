use crate::utils::{create_output_path, search_vm_files, write_to_file};
use crate::PathType;
use colored::*;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

mod memory;
use memory::generate_mem_code_block;

mod arithmetic_logic;
use arithmetic_logic::generate_a_l_code_block;

mod branching;
use branching::generate_branching_block;

mod function;
use function::generate_function_call;
use function::generate_function_def;
use function::generate_function_return;

// use it to store around CODE_BUFFER_LIMIT lines of assembly then write all lines to the output file and empty the vector.
const CODE_BUFFER_SOFT_LIMIT: usize = 100;

const DEFAULT_PADDING: usize = 4;
const NO_PADDING: usize = 0;

const SP: usize = 0;
const LCL: usize = 1;
const ARG: usize = 2;
const THIS: usize = 3;
const THAT: usize = 4;

const SP_BASE_ADDRESS: usize = 256;
const LCL_BASE_ADDRESS: usize = 1647;
const ARG_BASE_ADDRESS: usize = 1747;
const THIS_BASE_ADDRESS: usize = 1847;
const THAT_BASE_ADDRESS: usize = 1947;

fn at(n: usize) -> String {
    return format!("@{}", n);
}

// arithmetic / logical commands:
const A_L_KEYWORDS: [&'static str; 9] = ["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"];

// memory access commands:
const MEM_KEYWORDS: [&'static str; 2] = ["pop", "push"];

// branching keywords:
const BRANCHING_KEYWORDS: [&'static str; 3] = ["label", "if-goto", "goto"];

// function keywords
const FUNC_KEYWORDS: [&'static str; 3] = ["return", "function", "call"];

enum Keyword {
    FUNC(String),
    MEM(String),
    AL(String),
    BRANCH(String),
}

impl Keyword {
    fn from(s: &str) -> Option<Keyword> {
        if A_L_KEYWORDS.contains(&s) {
            return Some(Keyword::AL(s.to_string()));
        } else if MEM_KEYWORDS.contains(&s) {
            return Some(Keyword::MEM(s.to_string()));
        } else if FUNC_KEYWORDS.contains(&s) {
            return Some(Keyword::FUNC(s.to_string()));
        } else if BRANCHING_KEYWORDS.contains(&s) {
            return Some(Keyword::BRANCH(s.to_string()));
        } else {
            return None;
        }
    }
}

pub struct CodeGenerator<'a> {
    path_pointer: &'a Path,
    path_type: &'a PathType,
    output_file: File,
    is_debug_option: bool,
    jump_counter: usize,
}

impl CodeGenerator<'_> {
    pub fn new<'a>(
        path_ref: &'a Path,
        path_type: &'a PathType,
        is_debug_option: bool,
    ) -> CodeGenerator<'a> {
        let output_path = create_output_path(path_ref, path_type, is_debug_option);
        println!("Output: {output_path}");

        let output_file = File::create(&output_path).expect(
            format!(
                "[ERROR] couldn't create output file using the following path: {}",
                output_path,
            )
            .as_str(),
        ); // create asm file.

        let jump_counter = 0;

        return CodeGenerator {
            path_pointer: path_ref,
            path_type,
            output_file,
            is_debug_option,
            jump_counter, // in order to produce unique labels (for GOTOs).
        };
    }

    // ideally only this method and `new` should be exposed
    pub fn generate_code(&mut self) {
        let mut files_vec: Vec<PathBuf> = vec![];
        match &self.path_type {
            PathType::File => files_vec.push(self.path_pointer.to_path_buf()),
            PathType::Dir => search_vm_files(self.path_pointer, &mut files_vec),
        }

        let filename_vec: Vec<String> = (&files_vec)
            .iter()
            .map(|p| {
                p.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .trim_end_matches(".vm")
                    .to_string()
            })
            .collect();
        if !filename_vec.contains(&"Sys".to_string()) {
            println!(
                "{}",
                "[WARNING] file Sys.vm does not exist in directory.".purple()
            );
        }
        if !filename_vec.contains(&"Main".to_string()) {
            println!(
                "{}",
                "[WARNING] file Main.vm does not exist in directory.".purple()
            )
        }

        for path_buf in &files_vec {
            self.generate_code_from_file(&path_buf.as_path());
        }
    }

    fn generate_code_from_file(&mut self, file_path: &Path) {
        let mut code_buffer: Vec<String> = vec![];
        let filename = file_path // get filename and remove extension.
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches(".vm");

        let file = match File::open(&file_path) {
            Err(why) => {
                eprintln!("couldn't open {}: {}", file_path.to_str().unwrap(), why);
                panic!()
            }
            Ok(file) => file,
        };

        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();

        buf_reader
            .read_to_string(&mut contents)
            .expect("couldn't parse content of the input file.");

        let lines: Vec<&str> = contents.split("\n").collect();

        code_buffer.append(&mut generate_bootstrapping(
            &mut self.jump_counter,
            self.is_debug_option,
        ));

        for line in lines {
            if code_buffer.len() > CODE_BUFFER_SOFT_LIMIT {
                write_to_file(&mut self.output_file, &mut code_buffer);
            }

            // ignore comments and empty lines.
            if line.trim() == "" || line.starts_with("//") {
                continue;
            }

            let line = line.trim_end_matches("\r").trim_end_matches("\n"); // remove extra characters such as \r and \n.

            let mut line_vec: Vec<&str> = line.split(" ").collect(); // split line in words (operators and arguments) and store them in a vector.

            match Keyword::from(&line_vec[0]) {
                Some(keyword) => match keyword {
                    Keyword::AL(a_l_cmd) => {
                        let mut a_l_code_block = generate_a_l_code_block(
                            &a_l_cmd,
                            filename,
                            &mut self.jump_counter,
                            self.is_debug_option,
                            NO_PADDING,
                        );

                        code_buffer.append(&mut a_l_code_block);
                    }
                    Keyword::MEM(mem_cmd) => {
                        if line_vec.len() < 3 {
                            eprintln!("[ERROR] bad syntax. Line includes memory access operator but doesn't include one of the following arguments: a memory segment or an index.");
                            panic!();
                        }

                        let mem_segment = line_vec[1];

                        line_vec[2] = line_vec[2].trim_end_matches(|c| !char::is_numeric(c));

                        let mem_index: usize = match line_vec[2].parse() {
                            Ok(i) => i,
                            Err(_) => {
                                eprintln!(
                                    "[ERROR] bad syntax. Index {} cannot be parsed as an unsigned integer.",
                                    line_vec[2]
                                );
                                eprintln!("{:#?}", line_vec);
                                panic!();
                            }
                        };

                        let mut code_block = generate_mem_code_block(
                            &mem_cmd,
                            mem_segment,
                            mem_index,
                            filename,
                            self.is_debug_option,
                            NO_PADDING,
                        );
                        code_buffer.append(&mut code_block);
                    }
                    Keyword::BRANCH(branch_cmd) => {
                        // expected: label <str> or if-goto <str> or goto <str>
                        if line_vec.len() < 2 {
                            eprintln!("[ERROR] bad syntax. Line includes branching keyword but does not include a label.");
                            panic!();
                        }

                        let goto_label = line_vec[1];

                        let mut code_block = generate_branching_block(
                            &branch_cmd,
                            goto_label,
                            filename,
                            self.is_debug_option,
                            NO_PADDING,
                        );
                        code_buffer.append(&mut code_block);
                    }
                    Keyword::FUNC(func_keyword) => {
                        match func_keyword.as_str() {
                            "function" | "call" => {
                                if line_vec.len() < 3 {
                                    eprintln!("[ERROR] bad syntax. Expected two arguments for function call or definition.");
                                    panic!()
                                }
                                let function_name = line_vec[1];
                                if self.is_debug_option {
                                    let comment = format!(
                                        "\n// {} {} {}",
                                        func_keyword, function_name, line_vec[2]
                                    ); // line_vec[2] = n_args or n_vars
                                    code_buffer.push(comment);
                                }

                                let mut code_block = if func_keyword == "function" {
                                    let n_vars: usize = match line_vec[2].parse::<usize>() {
                                        Ok(n) => n,
                                        Err(e) => {
                                            eprintln!("{e}");
                                            panic!()
                                        }
                                    };

                                    generate_function_def(
                                        function_name,
                                        n_vars,
                                        filename,
                                        self.is_debug_option,
                                    )
                                } else {
                                    let n_args: usize = match line_vec[2].parse::<usize>() {
                                        Ok(n) => n,
                                        Err(e) => {
                                            eprintln!("{e}");
                                            panic!()
                                        }
                                    };

                                    generate_function_call(
                                        function_name,
                                        n_args,
                                        filename,
                                        &mut self.jump_counter,
                                        self.is_debug_option,
                                    )
                                };
                                code_buffer.append(&mut code_block);
                            }
                            "return" => {
                                if self.is_debug_option {
                                    let comment = format!("\n// {}", func_keyword);
                                    code_buffer.push(comment);
                                }
                                code_buffer.append(&mut generate_function_return(
                                    filename,
                                    &mut self.jump_counter,
                                    self.is_debug_option,
                                ));
                            }
                            _ => {
                                // should not be reachable
                                eprintln!("[ERROR] unkown error.");
                                panic!();
                            }
                        }
                    }
                },
                None => {
                    eprintln!("[ERROR] bad syntax. Line starts with unkown keyword.");
                    panic!();
                }
            }
        }

        if code_buffer.len() > 0 {
            write_to_file(&mut self.output_file, &mut code_buffer);
        }
    }
}

fn generate_bootstrapping(jump_counter_ref: &mut usize, is_debug_option: bool) -> Vec<String> {
    let mut code_block: Vec<String> = vec![
        at(SP_BASE_ADDRESS),
        "D = A".to_string(),
        at(SP),
        "M = D".to_string(),
        at(LCL_BASE_ADDRESS),
        "D = A".to_string(),
        at(LCL),
        "M = D".to_string(),
        at(ARG_BASE_ADDRESS),
        "D = A".to_string(),
        at(ARG),
        "M = D".to_string(),
        at(THIS_BASE_ADDRESS),
        "D = A".to_string(),
        at(THIS),
        "M = D".to_string(),
        at(THAT_BASE_ADDRESS),
        "D = A".to_string(),
        at(THAT),
        "M = D".to_string(),
    ];
    code_block.append(&mut generate_function_call(
        "Sys.init",
        0,
        "Sys",
        jump_counter_ref,
        is_debug_option,
    ));
    return code_block;
}
