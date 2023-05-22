use std::collections::HashMap;

use crate::utils::add_padding;

pub fn generate_mem_code_block(
    mem_cmd: &str,
    mem_segment: &str,
    mem_index: usize,
    filename: &str,
    is_debug_option: bool,
    padding: usize,
) -> Vec<String> {
    let label_pointers: HashMap<&str, usize> = HashMap::from([
        ("SP", 0),
        ("local", 1),
        ("argument", 2),
        ("this", 3),
        ("that", 4),
    ]);

    let mut code_block: Vec<String> = vec![];

    if is_debug_option {
        let comment = format!("// {} {} {}", mem_cmd, mem_segment, mem_index); // comment indicates which operation is being translated.
        code_block.push("\n".to_string());
        code_block.push(comment);
    }

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
                            eprintln!("[ERROR] bad syntax. Cannot pop a constant.");
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
    code_block = if is_debug_option && padding != 0 {
        code_block.iter().map(|s| add_padding(s, padding)).collect()
    } else {
        code_block
    };
    return code_block;
}
