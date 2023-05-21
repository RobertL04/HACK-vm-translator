use crate::utils::add_padding;

use super::{generate_mem_code_block, DEFAULT_PADDING};

pub fn generate_a_l_code_block(
    a_l_cmd: &str,
    filename: &str,
    mut jump_counter: usize,
    is_debug_option: bool,
    padding: usize,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];

    if is_debug_option {
        let comment = format!("// {}", a_l_cmd);
        code_block.push("\n".to_string());
        code_block.push(comment);
    }

    let mut pop1 = generate_mem_code_block(
        "pop",
        "general",
        13,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    ); // pop temp 1
    code_block.append(&mut pop1);
    // `neg` and `not` operate on one value only, so there is no need for popping a second value from the stack
    if a_l_cmd != "neg" && a_l_cmd != "not" {
        let mut pop2 = generate_mem_code_block(
            "pop",
            "general",
            14,
            filename,
            is_debug_option,
            DEFAULT_PADDING,
        );
        // pop temp 2
        code_block.append(&mut pop2);
    }
    code_block.push("@13".to_string()); // A = 13
    code_block.push("D = M".to_string()); // store content in D
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
                format!("@true_expression{}", jump_counter),
                "D;JEQ".to_string(),
                "@14".to_string(),
                "M = 0".to_string(),
                format!("@false_expression{}", jump_counter),
                "0;JMP".to_string(),
                format!("(true_expression{})", jump_counter),
                "@14".to_string(),
                "M = -1".to_string(),
                format!("(false_expression{})", jump_counter),
            ];
        }
        "gt" | "lt" => {
            jump_counter += 1;
            code_block.push("@14".to_string());

            if a_l_cmd == "lt" {
                code_block.push("D = D - M".to_string());
            // D  = temp1 - temp2 (if D>0 then temp2<temp1)
            } else {
                code_block.push("D = M - D".to_string());
                // D  = temp2 - temp1 (if D>0 then temp2>temp1)
            }

            // the rest is the same
            temp_vec = vec![
                format!("@true_expression{}", jump_counter),
                "D;JGT".to_string(), // jump to (greater) only if D>0
                "@14".to_string(),   // go to temp 2 (this code will be executed if NOT[D > 0])
                "M = 0".to_string(), // set temp2 to false
                format!("@false_expression{}", jump_counter),
                "0;JMP".to_string(),
                format!("(true_expression{})", jump_counter),
                "@14".to_string(),    // go to temp 2
                "M = -1".to_string(), // set temp2 to true since D>0
                format!("(false_expression{})", jump_counter),
            ];
        }
        _ => {
            // shouldn't be reached!
            eprintln!("[ERROR] unkown error.");
            panic!();
        }
    }
    code_block.append(&mut temp_vec);

    // push temp 2
    let mut push_temp_2 = generate_mem_code_block(
        "push",
        "general",
        14,
        filename,
        is_debug_option,
        DEFAULT_PADDING,
    );
    code_block.append(&mut push_temp_2);

    code_block = if is_debug_option && padding != 0 {
        code_block.iter().map(|s| add_padding(s, padding)).collect()
    } else {
        code_block
    };
    return code_block;
}
