use std::vec;

use super::{
    arithmetic_logic::generate_a_l_code_block, branching::generate_branching_block,
    memory::generate_mem_code_block, DEFAULT_PADDING,
};

// must return a value
pub fn generate_function_def(
    function_name: &str,
    n_vars: usize,
    filename: &str,
    is_debug_option: bool,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];
    // let function_label = format!("{filename}.{function_name}"); // agreed upon format Foo.bar
    // assumption: function_name = Class.method

    code_block.push(format!("({function_name})"));

    for _ in 0..n_vars {
        code_block.append(&mut generate_mem_code_block(
            "push",
            "constant",
            0,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        ));
    }

    return code_block;
}

// assumption: before calling a function, the neccessary args are pushed
pub fn generate_function_call(
    function_name: &str,
    n_args: usize,
    filename: &str,
    mut jump_counter: usize,
    is_debug_option: bool,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];

    jump_counter += 1;

    // push return label/address
    let return_label = format!("{}_ret_{}", filename, jump_counter);
    code_block.push(format!("({return_label})")); // (return_label)
    code_block.push(format!("@{return_label}")); // A = return label
                                                 //push return label
    code_block.push("D = A".to_string());
    code_block.push("@0".to_string());
    code_block.push("A = M".to_string());
    code_block.push("M = D".to_string());
    code_block.push("@0".to_string());
    code_block.push("M = M+1".to_string());

    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // push LCL
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        2,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // push ARG
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        3,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // push THIS
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        4,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // push THAT

    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // push SP (to calculate ARG)

    let x: usize = n_args + 5;
    code_block.append(&mut generate_mem_code_block(
        "push",
        "constant",
        x,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // TODO: check for off-by-one errors
    code_block.append(&mut generate_a_l_code_block(
        "sub",
        filename,
        jump_counter,
        is_debug_option,
        DEFAULT_PADDING,
    )); // on top of the stack: SP - (n_args+5)
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        2,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // ARG = SP-(n_args+5)

    // LCL = SP
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    let function_label = format!("{}.{}", filename, function_name); // should be changed

    code_block.append(&mut generate_branching_block(
        "goto",
        function_label.as_str(),
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    return code_block;
}

pub fn generate_function_return(
    filename: &str,
    jump_counter: usize,
    is_debug_option: bool,
) -> Vec<String> {
    // retrieve return address
    let mut code_block: Vec<String> = vec![];
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));
    code_block.append(&mut generate_mem_code_block(
        "push",
        "constant",
        5,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    code_block.append(&mut generate_a_l_code_block(
        "sub",
        filename,
        jump_counter,
        is_debug_option,
        DEFAULT_PADDING,
    )); // ret_addr_pointer = LCL - 5
        // n => (RAM[n] = ret_addr) now on top of the stack

    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        15,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // RAM[15] = ret_addr_pointer

    code_block.append(&mut generate_mem_code_block(
        "pop",
        "argument",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // *(ARG+0) = return value (is on top of the stack)

    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        2,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // *SP = ARG
    code_block.append(&mut generate_mem_code_block(
        "push",
        "constant",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // *SP = 1
    code_block.append(&mut generate_a_l_code_block(
        "add",
        filename,
        jump_counter,
        is_debug_option,
        DEFAULT_PADDING,
    )); // *SP = ARG + 1

    // destroy stack
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // RAM[0] = SP = ARG+1

    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    code_block.append(&mut generate_mem_code_block(
        "pop",
        "temp",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    )); // pop LCL

    // restore stack pointers (THAT, THIS, ARG, LCL)
    for n in 1..=4 {
        if is_debug_option {
            code_block.push("\n".to_string());
            code_block.push(format!("// restoring *(addr - {})", n));
        }
        code_block.append(&mut generate_mem_code_block(
            "push",
            "temp",
            0,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        )); // push endframe

        code_block.append(&mut generate_mem_code_block(
            "push",
            "constant",
            n,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        )); //  if n = 1 => THAT
        code_block.append(&mut generate_a_l_code_block(
            "sub",
            filename,
            jump_counter,
            is_debug_option,
            DEFAULT_PADDING,
        )); // SP -> ret_addr_pointer - n
        code_block.append(&mut vec![
            "@0".to_string(),
            "A = M - 1".to_string(), // A =SP -1
            "A = M".to_string(),     // go to pointer
            "D = M".to_string(),     // go to address pointed to
            "@0".to_string(),
            "A = M-1".to_string(), // A = SP-1
            "M = D".to_string(),   // replace pointer with value
        ]);
        code_block.append(&mut generate_mem_code_block(
            "pop",
            "general",
            5 - n,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        ));
    }

    code_block.push("@0".to_string());
    code_block.push("A = M".to_string()); // A = RAM[0]
    code_block.push("M = M -1".to_string()); // *(RAM[0]) -=1 (annul top most value on the stack)
    code_block.push("@15".to_string());
    code_block.push("A = M".to_string()); // A = RAM[13]
    code_block.push("A = M".to_string()); // A = *RAM[13]
    code_block.push("0; JMP".to_string());
    return code_block;
}