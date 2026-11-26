use std::io::{self, BufRead, Write};

use composition_mcp::{project_root_from_args, McpServer};

fn main() {
    let root = project_root_from_args();
    let mut server = McpServer::new(&root);
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("composition-mcp: read error: {error}");
                std::process::exit(2);
            }
        };
        if let Some(response) = server.handle_line(&line) {
            if let Err(error) = writeln!(stdout, "{response}") {
                eprintln!("composition-mcp: write error: {error}");
                std::process::exit(2);
            }
            if let Err(error) = stdout.flush() {
                eprintln!("composition-mcp: flush error: {error}");
                std::process::exit(2);
            }
        }
    }
}
