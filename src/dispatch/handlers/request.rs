use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, Position};

pub fn handle_completions(text: &str, pos: &Position) -> Vec<CompletionItem> {
    let line = text.lines().nth(pos.line as usize).unwrap_or("");
    let prefix = &line[..pos.character as usize].trim_start();

    if prefix.starts_with('.') {
        get_directive_completions()
    } else if prefix.ends_with("ADD ") || prefix.ends_with("AND ") {
        get_register_completions()
    } else if prefix.ends_with("TRAP ") {
        get_trap_completions()
    } else {
        let mut resp = get_opcode_completions();
        resp.append(&mut get_directive_completions());
        resp
    }
}

fn get_opcode_completions() -> Vec<CompletionItem> {
    let opcodes = [
        ("ADD", "Add two values"),
        ("AND", "Bitwise AND"),
        ("BR", "Branch unconditionally"),
        ("BRn", "Branch if negative"),
        ("BRz", "Branch if zero"),
        ("BRp", "Branch if positive"),
        ("BRnz", "Branch if negative or zero"),
        ("BRnp", "Branch if negative or positive"),
        ("BRzp", "Branch if zero or positive"),
        ("BRnzp", "Branch unconditionally"),
        ("JMP", "Jump to address"),
        ("JSR", "Jump to subroutine"),
        ("JSRR", "Jump to subroutine (register)"),
        ("LD", "Load from PC-relative address"),
        ("LDI", "Load indirect"),
        ("LDR", "Load base+offset"),
        ("LEA", "Load effective address"),
        ("NOT", "Bitwise NOT"),
        ("RET", "Return from subroutine"),
        ("RTI", "Return from interrupt"),
        ("ST", "Store to PC-relative address"),
        ("STI", "Store indirect"),
        ("STR", "Store base+offset"),
        ("TRAP", "System call"),
    ];

    opcodes
        .iter()
        .map(|(label, detail)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect()
}

fn get_directive_completions() -> Vec<CompletionItem> {
    let directives = [
        (".ORIG", "Set program origin address"),
        (".END", "End of program"),
        (".FILL", "Fill memory location with value"),
        (".BLKW", "Allocate block of words"),
        (".STRINGZ", "Null-terminated string"),
    ];

    directives
        .iter()
        .map(|(label, detail)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        })
        .collect()
}

fn get_register_completions() -> Vec<CompletionItem> {
    (0..8)
        .map(|i| CompletionItem {
            label: format!("R{}", i),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(format!("General purpose register {}", i)),
            ..Default::default()
        })
        .collect()
}

fn get_trap_completions() -> Vec<CompletionItem> {
    let traps = [
        ("GETC", "x20", "Read single character (no echo)"),
        ("OUT", "x21", "Output character in R0"),
        ("PUTS", "x22", "Output null-terminated string"),
        ("IN", "x23", "Read character with echo and prompt"),
        ("PUTSP", "x24", "Output packed string"),
        ("HALT", "x25", "Halt program execution"),
    ];

    traps
        .iter()
        .map(|(label, code, desc)| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some(format!("{} - {}", code, desc)),
            ..Default::default()
        })
        .collect()
}
