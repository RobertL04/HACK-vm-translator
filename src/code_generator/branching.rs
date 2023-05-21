use crate::utils::add_padding;

use super::{generate_mem_code_block, DEFAULT_PADDING};

pub fn generate_branching_block(
    branch_keyword: &str,
    goto_label: &str,
    filename: &str,
    is_debug_option: bool,
    padding: usize,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];

    if is_debug_option {
        let comment = format!("// {} {}", branch_keyword, goto_label); // comment indicates which operation is being translated.
        code_block.push("\n".to_string());
        code_block.push(comment);
    }
    // assumption: the same exact label cannot be used in multiple vm files
    // that means if the same label is found in more than one file, each declaration is considered unique
    let unique_label = format!("{}_{}", goto_label, filename);
    match branch_keyword {
        "label" => {
            code_block.push(format!("({})", unique_label));
        }
        "goto" => {
            code_block.push(format!("@{}", unique_label));
            code_block.push("0;JMP".to_string());
        }
        "if-goto" => {
            // expects that a value is pushed on the stack
            code_block.append(&mut generate_mem_code_block(
                "pop",
                "general",
                13,
                filename,
                is_debug_option,
                DEFAULT_PADDING,
            )); // RAM[13]  = value on stack
            code_block.push("@13".to_string()); // A = 13
            code_block.push("D = M".to_string()); // D = RAM[13]
            code_block.push(format!("@{}", unique_label));
            code_block.push("D;JNE".to_string()); // if D!=0 jump
        }
        _ => {
            // shouldn't be reached
            eprintln!("[ERROR] unkown error.");
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
