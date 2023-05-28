use crate::utils::add_padding;

use super::{at, generate_mem_code_block, DEFAULT_PADDING};

/// Assumes that `goto_label` is unique accross all vm files.
///
/// When `if-goto` is used, it is expected that a boolean value is pushed on the stack.
/// The author of the vm code is responsible for ensuring that said condition is true.
pub fn generate_branching_block(
    branch_keyword: &str,
    goto_label: &str,
    filename: &str,
    is_debug_option: bool,
    padding: usize,
) -> Vec<String> {
    let mut code_block: Vec<String> = vec![];

    if is_debug_option {
        let comment = format!("// {} {}", branch_keyword, goto_label);
        code_block.push("\n".to_string());
        code_block.push(comment);
    }

    let unique_label = format!("{}", goto_label);
    match branch_keyword {
        "label" => {
            code_block.push(format!("({})", unique_label));
        }
        "goto" => {
            code_block.push(at(unique_label));
            code_block.push("0;JMP".to_string());
        }
        "if-goto" => {
            code_block.append(&mut generate_mem_code_block(
                "pop",
                "general",
                13,
                filename,
                is_debug_option,
                DEFAULT_PADDING,
            ));
            code_block.push(at(13));
            code_block.push("D = M".to_string());
            code_block.push(at(unique_label));
            code_block.push("D;JNE".to_string());
        }
        _ => {
            eprintln!(
                "[ERROR] keyword {} cannot be recognized as a branching command.",
                branch_keyword
            );
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
