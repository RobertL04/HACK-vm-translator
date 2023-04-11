use core::panic;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
    process::exit,
    vec,
};

// TODO: generate helpful comments
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // print warning
        exit(1);
    }

    // check if file path exists and has vm extension:
    if !args[1].ends_with(".vm") {
        eprintln!("[ERROR] Input file does not have `.vm` extension. Please provide a file that includes HACK vm code and has `.vm` extension.");
        panic!();
    }
    let mut debug_option = false;
    if args.contains(&"--debug".to_string()) {
        debug_option = true;
    }

    let path_str = args[1].clone();
    let path = Path::new(&path_str);
    let file = match File::open(&path) {
        Err(why) => {
            eprintln!("couldn't open {}: {}", path_str, why);
            panic!()
        }
        Ok(file) => file,
    };

    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    let lines: Vec<&str> = contents.split("\n").collect();

    // arithmetic / logical commands:
    let a_l_commands = ["add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not"];

    // memory access commands:
    let mem_commands = ["pop", "push"];

    // memory segments:
    let segments = [
        "local", "argument", "constant", "this", "that", "static", "pointer", "temp", "general",
    ];

    let filename = path // get filename and remove extension.
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_end_matches(".vm");

    // TODO: better error handling
    let mut output_path = format!("{}.asm", path.to_str().unwrap().trim_end_matches(".vm"));

    //  allows user to create seperate asm file that includes comments but cannot be executed using the CPU emulator included in the NAND To Tetris software suite.
    if debug_option {
        output_path = format!(
            "{}.debug.asm",
            path.to_str().unwrap().trim_end_matches(".vm")
        )
    }

    let mut output_file = File::create(output_path).unwrap(); // create asm file.

    const CODE_BUFFER_LIMIT: usize = 100;

    // use it to store around CODE_BUFFER_LIMIT lines of assembly then write all lines to the output file and empty the vector.
    let mut code_buffer: Vec<String> = vec![];

    let mut jump_counter = 0; // in order to produce unique jump labels (for GOTOs).

    for l in lines {
        if code_buffer.len() > CODE_BUFFER_LIMIT {
            write_to_file(&mut output_file, &mut code_buffer);
        }

        if l.trim() == "" || l.starts_with("//") {
            // ignore comments and empty lines.
            continue;
        }

        let l = l.trim_end_matches("\r").trim_end_matches("\n"); // remove extra characters such as \r and \n.

        let l_vec: Vec<&str> = l.split(" ").collect(); // split line in words (operators and arguments) and store them in a vector.

        // figure out which type of operations the line includes.
        if a_l_commands.contains(&l_vec[0]) {
            let a_l_cmd = l_vec[0];

            if l_vec.len() > 1 {
                eprintln!("[ERROR] bad syntax. Line starts with an arithmetic operation but includes more than one word.");
                panic!();
            }

            if debug_option {
                let comment = format!("\n// {}", a_l_cmd);
                code_buffer.push(comment);
            }

            let mut pop1 = generate_mem_code_block("pop", "general", 13, filename); // pop temp 1
            code_buffer.append(&mut pop1);
            // `neg` and `not` operate on one value only, so there is no need for popping a second value from the stack
            if a_l_cmd != "neg" && a_l_cmd != "not" {
                let mut pop2 = generate_mem_code_block("pop", "general", 14, filename);
                // pop temp 2
                code_buffer.append(&mut pop2);
            }
            code_buffer.push("@13".to_string()); // A = 13
            code_buffer.push("D = M".to_string()); // store content in D

            let mut temp_vec: Vec<String>;
            match a_l_cmd {
                "add" => {
                    temp_vec = vec![
                        "@14".to_string(),       // go to temp 2
                        "D = D + M".to_string(), // D  = D + value of temp2
                        "M = D".to_string(),     // replace temp 2 with the value of temp2+temp1
                    ];
                }
                "sub" => {
                    temp_vec = vec![
                        "@14".to_string(), // go to temp 2
                        // TODO: make sure this is correct:
                        "D = M - D".to_string(), // D  = temp2 - temp1
                        "M = D".to_string(),     // replace temp 2 with the value of temp2 - temp1
                    ];
                }
                "neg" => {
                    temp_vec = vec![
                        "D = -D".to_string(), // D  = - temp1
                        "@14".to_string(),    // go to temp 2
                        "M = D".to_string(),  // replace temp 2 with the value of -temp1
                    ];
                }
                "and" => {
                    temp_vec = vec![
                        "@14".to_string(),       // go to temp 2
                        "D = M & D".to_string(), // D  = temp2 & temp1
                        "M = D".to_string(),     // replace temp 2 with the value of temp2 & temp1
                    ];
                }
                "or" => {
                    temp_vec = vec![
                        "@14".to_string(),       // go to temp 2
                        "D = M | D".to_string(), // D  = temp2 | temp1
                        "M = D".to_string(),     // replace temp 2 with the value of temp2 | temp1
                    ];
                }
                "not" => {
                    temp_vec = vec![
                        "D = !D".to_string(), // D = !temp1
                        "@14".to_string(),    // go to temp 2
                        "M = D".to_string(),  // replace temp 2 with the value of !temp1
                    ];
                }
                "eq" => {
                    jump_counter += 1;
                    temp_vec = vec![
                        "@14".to_string(),       // go to temp 2
                        "D = M - D".to_string(), // D  = temp2 - temp1 (order doesn't matter since we're checking for inequality with 0)
                        format!("@true_expression{jump_counter}"),
                        "D;JEQ".to_string(),
                        "@14".to_string(),
                        "M = 0".to_string(),
                        format!("@false_expression{jump_counter}"),
                        "0;JMP".to_string(),
                        format!("(true_expression{jump_counter})"),
                        "@14".to_string(),
                        "M = -1".to_string(),
                        format!("(false_expression{jump_counter})"),
                    ];
                }
                "gt" | "lt" => {
                    jump_counter += 1;
                    code_buffer.push("@14".to_string());

                    if a_l_cmd == "lt" {
                        code_buffer.push("D = D - M".to_string());
                    // D  = temp1 - temp2 (if D>0 then temp2<temp1)
                    } else {
                        code_buffer.push("D = M - D".to_string());
                        // D  = temp2 - temp1 (if D>0 then temp2>temp1)
                    }

                    // the rest is the same
                    temp_vec = vec![
                        format!("@true_expression{jump_counter}"),
                        "D;JGT".to_string(), // jump to (greater) only if D>0
                        "@14".to_string(), // go to temp 2 (this code will be executed if NOT[D > 0])
                        "M = 0".to_string(), // set temp2 to false
                        format!("@false_expression{jump_counter}"),
                        "0;JMP".to_string(),
                        format!("(true_expression{jump_counter})"),
                        "@14".to_string(),    // go to temp 2
                        "M = -1".to_string(), // set temp2 to true since D>0
                        format!("(false_expression{jump_counter})"),
                    ];
                }
                _ => {
                    // shouldn't be reached!
                    eprintln!("[ERROR] unkown error.");
                    panic!();
                }
            }
            code_buffer.append(&mut temp_vec);

            // push temp 2
            let mut push_temp_2 = generate_mem_code_block("push", "general", 14, filename);
            code_buffer.append(&mut push_temp_2);
        } else if mem_commands.contains(&l_vec[0]) {
            let mem_cmd = l_vec[0];
            if l_vec.len() < 3 {
                eprintln!("[ERROR] bad syntax. Line includes memory access operator but doesn't include one of the following arguments: a memory segment or an index.");
                panic!();
            }

            let mem_segment = l_vec[1]; // store the memory segment provided
            if !segments.contains(&mem_segment) {
                eprintln!(
                    "[ERROR] bad syntax. Memory segment {} does not exist or is not supported.",
                    { mem_segment }
                );
                panic!();
            }
            let mem_index: usize = match l_vec[2].parse() {
                Ok(i) => i,
                Err(_) => {
                    eprintln!(
                        "[ERROR] bad syntax. Index {} cannot be parsed as an unsigned integer.",
                        l_vec[2]
                    );
                    panic!();
                }
            };

            if debug_option {
                let comment = format!("\n// {} {} {}", mem_cmd, mem_segment, mem_index); // comment indicates which operation is being translated.
                code_buffer.push(comment);
            }

            let mut code_block = generate_mem_code_block(mem_cmd, mem_segment, mem_index, filename); // generate assembly code using the arguments provided in the vm code.

            code_buffer.append(&mut code_block); // append the generated code to the global buffer.
        } else {
            eprintln!("[ERROR] bad syntax. First word in the line is neither a memory access command nor an arithmetic/logical operator.");
            panic!();
        }
    }
    if code_buffer.len() > 0 {
        write_to_file(&mut output_file, &mut code_buffer);
    }
}

fn generate_mem_code_block(
    mem_cmd: &str,
    mem_segment: &str,
    mem_index: usize,
    filename: &str,
) -> Vec<String> {
    let label_pointers: HashMap<&str, usize> = HashMap::from([
        ("SP", 0),
        ("local", 1),
        ("argument", 2),
        ("this", 3),
        ("that", 4),
    ]);

    let mut code_block: Vec<String> = vec![];
    match mem_cmd {
        "push" => {
            // command: push segment i
            // implementation:
            // addr = segmentPointer +i;
            // *sp = *addr; sp++;
            match label_pointers.contains_key(mem_segment) {
                true => {
                    // TODO: better error handling

                    let mut temp_vec: Vec<String> = vec![
                        format!("@{}", label_pointers.get(mem_segment).unwrap()),
                        "D = M".to_string(), // D = base address
                        format!("@{}", mem_index),
                        "D = D + A".to_string(), // D  = base address + index
                        "@15".to_string(),       // set A = 15; result: M is now RAM[temp3].
                        format!("M = D"),        // calc_addr  = D = base address + index
                        "A = M".to_string(),     // go to the calc_addr
                        "D = M".to_string(), // store content of the variable that calc_addr points to
                        format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                        "A = M".to_string(), // go to the variable that SP points to
                        "M = D".to_string(), // set the content of the variable to the value previously fetched
                        format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                        "M = M + 1".to_string(), // increment SP
                    ];
                    code_block.append(&mut temp_vec);
                }
                false => {
                    match mem_segment {
                        "constant" => {
                            // mem_index in this case is actually a constant value
                            // which means that the name `mem_index`
                            // isn't accurate in this case

                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", mem_index),                         // A = constant
                                "D = A".to_string(),                               // D = constant
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "M = D".to_string(), // set the content of the variable to D
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M + 1".to_string(), // increment SP
                            ];

                            code_block.append(&mut temp_vec);
                        }
                        "temp" => {
                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", 5 + mem_index),                     // A = 5 + i
                                "D = M".to_string(),                               // D = RAM[5+i]
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "M = D".to_string(), // set the content of the variable to D
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M + 1".to_string(), // increment SP
                            ];

                            code_block.append(&mut temp_vec);
                        }
                        "pointer" => {
                            if mem_index != 1 && mem_index != 0 {
                                eprintln!("[ERROR] bad syntax.");
                                panic!();
                            }
                            let label: &str = if mem_index == 0 { "THIS" } else { "THAT" };

                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", label), // A = THIS/THAT (this/that are pointers to their respective segments)
                                "D = M".to_string(), // D = *(this/that) which means D = (base address of this/that segment))
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "M = D".to_string(), // set the content of the variable to D
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M + 1".to_string(), // increment SP
                            ];

                            code_block.append(&mut temp_vec);
                        }
                        "static" => {
                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}.{}", filename, mem_index),            // @Foo.i
                                "D = M".to_string(), // D = content at RAM[Foo.i]
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "M = D".to_string(), // set the content of the variable to D
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M + 1".to_string(), // increment SP
                            ];

                            code_block.append(&mut temp_vec);
                        }
                        "general" => {
                            // directly access any RAM[n]
                            // be careful when using this
                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", mem_index),                         // A = n
                                "D = M".to_string(), // D = content at RAM[n]
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "M = D".to_string(), // set the content of the variable to D
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M + 1".to_string(), // increment SP
                            ];
                            code_block.append(&mut temp_vec);
                        }
                        _ => {
                            eprintln!("[ERROR] bad syntax.");
                            panic!();
                        }
                    }
                }
            }
        }
        "pop" => {
            // command: pop segment i
            // implementation:
            // addr = segmentPointer +i;
            // sp--; *addr = *sp;
            match label_pointers.contains_key(mem_segment) {
                true => {
                    // TODO: better error handling
                    let mut temp_vec: Vec<String> = vec![
                        format!("@{}", label_pointers.get(mem_segment).unwrap()),
                        "D = M".to_string(), // D = base address
                        format!("@{}", mem_index),
                        "D = D + A".to_string(), // D  = base address + index
                        "@15".to_string(),       // set A = temp3; result: M is now RAM[temp3].
                        format!("M = D"),        // calc_addr  = D = base address + index
                        format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                        "M = M - 1".to_string(), // decrement SP
                        format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                        "A = M".to_string(),     // go to the variable that SP points to
                        "D = M".to_string(),     // store the content in D
                        "@15".to_string(),       // A = temp3
                        "A = M".to_string(),     // go to the RAM[calc_addr]
                        "M = D".to_string(), // store content of D (now contains data fetched from the stack)
                    ];
                    code_block.append(&mut temp_vec);
                }
                false => {
                    match mem_segment {
                        "constant" => {
                            eprintln!("[ERROR] bad syntax. Cannot use pop with `constant`.");
                            panic!();
                        }
                        "temp" => {
                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M - 1".to_string(),                           // decrement SP
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "D = M".to_string(), // store the content in D
                                format!("@{}", 5 + mem_index), // A = 5 + i
                                "M = D".to_string(), // store the content in RAM[5+i]
                            ];

                            code_block.append(&mut temp_vec);
                        }

                        "pointer" => {
                            if mem_index != 1 && mem_index != 0 {
                                eprintln!("[ERROR] bad syntax.");
                                panic!();
                            }
                            let label: &str = if mem_index == 0 { "THIS" } else { "THAT" };

                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M - 1".to_string(),                           // decrement SP
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "M = D".to_string(), // store the value in D
                                format!("@{}", label), // A = THIS/THAT (this/that are pointers to their respective segments)
                                "M = D".to_string(), // THIS/THAT = the highest number in the stack (stored in D)
                            ];

                            code_block.append(&mut temp_vec);
                        }

                        "static" => {
                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M - 1".to_string(),                           // decrement SP
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "D = M".to_string(), // store the content of the variable in D
                                format!("@{}.{}", filename, mem_index), // @Foo.i
                                "M = D".to_string(), // RAM[Foo.i] = D
                            ];
                            code_block.append(&mut temp_vec);
                        }
                        "general" => {
                            let mut temp_vec: Vec<String> = vec![
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "M = M - 1".to_string(),                           // decrement SP
                                format!("@{}", label_pointers.get("SP").unwrap()), // A = 0
                                "A = M".to_string(), // go to the variable that SP points to
                                "D = M".to_string(), // store the content in D
                                format!("@{}", mem_index), // A = n
                                "M = D".to_string(), // store the content in RAM[n]
                            ];

                            code_block.append(&mut temp_vec);
                        }
                        _ => {
                            eprintln!("[ERROR] bad syntax.");
                            panic!();
                        }
                    }
                }
            }
        }
        _ => {
            // shouldn't be reached!
            eprintln!("[ERROR] bad syntax.");
            panic!();
        }
    }

    return code_block;
}

fn write_to_file(file_pointer: &mut File, code_buffer_pointer: &mut Vec<String>) {
    let code_block_as_str = code_buffer_pointer.join("\n");
    file_pointer
        .write_all(&code_block_as_str.as_bytes())
        .unwrap();
    file_pointer.write_all("\n".as_bytes()).unwrap();
    code_buffer_pointer.clear();
}
