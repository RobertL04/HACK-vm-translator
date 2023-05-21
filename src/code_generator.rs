use crate::utils::{create_output_file, search_vm_files, write_to_file};
use crate::PathType;
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

// TODO: bootstrapping!

// TODO: better error handling

pub struct CodeGenerator<'a> {
    path_pointer: &'a Path,
    path_type: &'a PathType,

    a_l_commands: [&'a str; 9],
    mem_commands: [&'a str; 2],
    branching_keywords: [&'a str; 3],
    func_keywords: [&'a str; 3],

    segments: [&'a str; 9],
    output_file: File,
    is_debug_option: bool,
    jump_counter: usize,
}

impl CodeGenerator<'_> {
    pub fn new<'a>(
        path_pointer: &'a Path,
        path_type: &'a PathType,
        is_debug_option: bool,
    ) -> CodeGenerator<'a> {
        // arithmetic / logical commands:
        let a_l_commands = ["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"];

        // memory access commands:
        let mem_commands = ["pop", "push"];

        // branching keywords:
        let branching_keywords = ["label", "if-goto", "goto"];

        // function keywords
        let func_keywords = ["return", "function", "call"];

        // memory segments:
        let segments = [
            "local", "argument", "constant", "this", "that", "static", "pointer", "temp", "general",
        ];

        let output_path = create_output_file(path_pointer, path_type, is_debug_option);
        println!("{output_path}");

        let output_file = File::create(output_path).unwrap(); // create asm file.
        let jump_counter = 0;

        return CodeGenerator {
            path_pointer,
            path_type,

            a_l_commands,
            mem_commands,
            branching_keywords,
            func_keywords,

            segments,
            output_file,
            is_debug_option,

            jump_counter, // in order to produce unique jump labels (for GOTOs).
        };
    }

    // ideally only this method and `new` should be exposed
    pub fn generate_code(&mut self) {
        let mut files_vec: Vec<PathBuf> = vec![];
        match &self.path_type {
            PathType::File => files_vec.push(self.path_pointer.to_path_buf()),
            PathType::Dir => search_vm_files(self.path_pointer, &mut files_vec),
        }
        for path_buf in files_vec {
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
        buf_reader.read_to_string(&mut contents).unwrap();
        let lines: Vec<&str> = contents.split("\n").collect();

        for l in lines {
            if code_buffer.len() > CODE_BUFFER_SOFT_LIMIT {
                write_to_file(&mut self.output_file, &mut code_buffer);
            }

            if l.trim() == "" || l.starts_with("//") {
                // ignore comments and empty lines.
                continue;
            }

            let l = l.trim_end_matches("\r").trim_end_matches("\n"); // remove extra characters such as \r and \n.

            let mut l_vec: Vec<&str> = l.split(" ").collect(); // split line in words (operators and arguments) and store them in a vector.

            // figure out which type of operations the line includes.
            if self.a_l_commands.contains(&l_vec[0]) {
                let a_l_cmd = l_vec[0];

                let mut a_l_code_block = generate_a_l_code_block(
                    a_l_cmd,
                    filename,
                    &mut self.jump_counter,
                    self.is_debug_option,
                    NO_PADDING,
                );

                code_buffer.append(&mut a_l_code_block);
            } else if self.mem_commands.contains(&l_vec[0]) {
                let mem_cmd = l_vec[0];
                if l_vec.len() < 3 {
                    eprintln!("[ERROR] bad syntax. Line includes memory access operator but doesn't include one of the following arguments: a memory segment or an index.");
                    panic!();
                }

                let mem_segment = l_vec[1]; // store the memory segment provided
                if !self.segments.contains(&mem_segment) {
                    eprintln!(
                        "[ERROR] bad syntax. Memory segment {} does not exist or is not supported.",
                        { mem_segment }
                    );
                    panic!();
                }
                // TODO: fix this bug in a better way
                l_vec[2] = l_vec[2].trim_end_matches(|c| !char::is_numeric(c));

                let mem_index: usize = match l_vec[2].parse() {
                    Ok(i) => i,
                    Err(_) => {
                        eprintln!(
                            "[ERROR] bad syntax. Index {} cannot be parsed as an unsigned integer.",
                            l_vec[2]
                        );
                        eprintln!("{:#?}", l_vec);
                        panic!();
                    }
                };

                let mut code_block = generate_mem_code_block(
                    mem_cmd,
                    mem_segment,
                    mem_index,
                    filename,
                    self.is_debug_option,
                    NO_PADDING,
                ); // generate assembly code using the arguments provided in the vm code.

                code_buffer.append(&mut code_block); // append the generated code to the global buffer.
            } else if self.branching_keywords.contains(&l_vec[0]) {
                let branch_keyword = l_vec[0];

                // expected: label <str> or if-goto <str> or goto <str>
                if l_vec.len() < 2 {
                    eprintln!("[ERROR] bad syntax. Line includes branching keyword but does not include any arguments.");
                    panic!();
                }

                let goto_label = l_vec[1];

                let mut code_block = generate_branching_block(
                    branch_keyword,
                    goto_label,
                    filename,
                    self.is_debug_option,
                    NO_PADDING, // means function does not require tabs
                );
                code_buffer.append(&mut code_block);
            } else if self.func_keywords.contains(&l_vec[0]) {
                let func_keyword = l_vec[0];

                match func_keyword {
                    // error checking
                    "function" | "call" => {
                        if l_vec.len() < 3 {
                            eprintln!("[ERROR] bad syntax. Expected two arguments for function call or definition.");
                            panic!()
                        }
                        let function_name = l_vec[1];
                        if self.is_debug_option {
                            let comment =
                                format!("\n// {} {} {}", func_keyword, function_name, l_vec[2]); // l_vec[2] is local vars number
                                                                                                 // or args number depending on the keyword
                            code_buffer.push(comment);
                        }

                        let mut code_block = if func_keyword == "function" {
                            let n_vars: usize = match l_vec[2].parse::<usize>() {
                                Ok(n) => n,
                                Err(e) => {
                                    eprintln!("{e}");
                                    panic!()
                                }
                            };

                            // line starts with `function`
                            generate_function_def(
                                function_name,
                                n_vars,
                                filename,
                                self.is_debug_option,
                            )
                        } else {
                            let n_args: usize = match l_vec[2].parse::<usize>() {
                                Ok(n) => n,
                                Err(e) => {
                                    eprintln!("{e}");
                                    panic!()
                                }
                            };

                            // line starts with `call`
                            if self.is_debug_option {
                                let comment =
                                    format!("\n// {} {} {}", func_keyword, function_name, n_args); //call foo
                                code_buffer.push(comment);
                            }
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
                            let comment = format!("\n// {}", func_keyword); // return
                            code_buffer.push(comment);
                        }
                        code_buffer.append(&mut generate_function_return(
                            filename,
                            &mut self.jump_counter,
                            self.is_debug_option,
                        ));
                    }
                    _ => {
                        // should be unreachable
                        eprintln!("[ERROR] unkown error.");
                        panic!()
                    }
                }
            } else {
                eprintln!("[ERROR] bad syntax. Line starts with unkown keyword.");
                panic!();
            }
        }
        if code_buffer.len() > 0 {
            write_to_file(&mut self.output_file, &mut code_buffer);
        }
    }
}

// TODO: for debugging, try to include expected PC for each instruction (remember to handle labels correctly, etc.)
