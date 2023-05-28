use std::vec;

use super::{
    arithmetic_logic::generate_a_l_code_block, at, branching::generate_branching_block,
    memory::generate_mem_code_block, DEFAULT_PADDING, SP,
};

// The last 3 registers in the `temp` segment are used by the vm translator
// and shouldn't be used in the actual vm code being translated.

/// Used to temporarily store the endframe address
const TEMP_X: usize = 10;

/// Used to retrieve return address
const TEMP_Y: usize = 11;

/// Used to temporarily store the new stack pointer in order to destroy the stack
const TEMP_Z: usize = 12;

/// Assumes that the parameter `function_name` is unique (Class.function):
pub fn generate_function_def(
    function_name: &str,
    n_vars: usize,
    filename: &str,
    is_debug_option: bool,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];
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

/// Assumption: before calling a function, the neccessary arguments are pushed on the stack
pub fn generate_function_call(
    function_name: &str,
    n_args: usize,
    filename: &str,
    jump_counter_ref: &mut usize,
    is_debug_option: bool,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];

    *jump_counter_ref += 1;

    let return_label = format!("{}_ret_{}", function_name, *jump_counter_ref);
    // push return label/address
    code_block.append(&mut vec![
        at(&return_label),
        //push return label
        "D = A".to_string(),
        at(SP),
        "A = M".to_string(),
        "M = D".to_string(),
        at(SP),
        "M = M+1".to_string(),
    ]);

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

    code_block.append(&mut generate_a_l_code_block(
        "sub",
        filename,
        jump_counter_ref,
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

    code_block.append(&mut generate_branching_block(
        "goto",
        function_name,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));
    code_block.push(format!("({return_label})"));

    return code_block;
}

pub fn generate_function_return(
    filename: &str,
    jump_counter_ref: &mut usize,
    is_debug_option: bool,
) -> Vec<String> {
    *jump_counter_ref += 1;
    // retrieve return address
    let mut code_block: Vec<String> = vec![];

    // push LCL
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

    // leaves (LCL-5) on top of the stack
    code_block.append(&mut generate_a_l_code_block(
        "sub",
        filename,
        jump_counter_ref,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // RAM[TEMP_Y] = LCL-5 = pointer to return address
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        TEMP_Y,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // RAM[TEMP_Y] = *(LCL-5) = real return address
    code_block.append(&mut vec![
        at(TEMP_Y),
        "A = M".to_string(),
        "D = M".to_string(),
        at(TEMP_Y),
        "M = D".to_string(),
    ]);

    // push LCL
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // pop endframe address (equivalent to LCL)
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        TEMP_X,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // *(ARG+0) = return value (is on top of the stack)
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "argument",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // *SP = ARG
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        2,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // *SP = 1
    code_block.append(&mut generate_mem_code_block(
        "push",
        "constant",
        1,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // *SP = ARG + 1
    code_block.append(&mut generate_a_l_code_block(
        "add",
        filename,
        jump_counter_ref,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // RAM[TEMP_Z] = ARG+1
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        TEMP_Z,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // restore stack pointers (THAT, THIS, ARG, LCL)
    for n in 1..=4 {
        if is_debug_option {
            code_block.push("\n".to_string());
            code_block.push(format!("// restoring *(addr - {})", n));
        }

        // push endframe address that was previously saved in RAM[TEMP_X] (see above)
        code_block.append(&mut generate_mem_code_block(
            "push",
            "general",
            TEMP_X,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        ));

        // *SP = n
        code_block.append(&mut generate_mem_code_block(
            "push",
            "constant",
            n,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        ));

        // *SP = endframe address - n
        code_block.append(&mut generate_a_l_code_block(
            "sub",
            filename,
            jump_counter_ref,
            is_debug_option,
            DEFAULT_PADDING,
        ));

        // RAM[endframe address - n] = address to old segment pointers (that belongs to the caller function)
        code_block.append(&mut vec![
            at(SP),
            "A = M - 1".to_string(), // go to endframe address-n
            "A = M".to_string(),     // go to base address stored in the (old) segment pointer
            "D = M".to_string(),     // D = base address of the old segment
            at(SP),
            "A = M-1".to_string(),
            "M = D".to_string(), // replace pointer with base address of the old segment
        ]);

        // replaces the current segment pointer with the base address of the old segment
        code_block.append(&mut generate_mem_code_block(
            "pop",
            "general",
            5 - n,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        ));
    }

    // *SP = new sp stored in RAM[TEMP_Z]
    code_block.append(&mut generate_mem_code_block(
        "push",
        "general",
        TEMP_Z,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // destroy stack
    // RAM[0] = SP = ARG+1
    code_block.append(&mut generate_mem_code_block(
        "pop",
        "general",
        0,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ));

    // jump to return address
    code_block.push(at(TEMP_Y));
    code_block.push("A = M".to_string());
    code_block.push("0; JMP".to_string());
    return code_block;
}
