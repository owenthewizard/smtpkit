use rustyline::DefaultEditor;

use snail_smtp::*;

fn main() {
    let mut rl = DefaultEditor::new().expect("Failed to init editor");
    while let Ok(ref line) = rl.readline(">> ") {
        match parse_command(line.as_bytes()) {
            Ok(cmd) => {
                println!("Parsed command: {cmd:?}");
            }
            Err(e) => {
                println!("Error parsing command: {e}");
            }
        }
        rl.add_history_entry(line).expect("Failed to save history");
    }
}
