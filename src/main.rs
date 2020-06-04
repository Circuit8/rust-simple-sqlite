use std::io;
use std::io::{BufRead, Stdout, Write};

fn main() {
    let mut stdout = io::stdout();
    print_prompt(&mut stdout);

    for line in io::stdin().lock().lines() {
        let command = line.unwrap();

        if command.starts_with(".") {
            match execute_meta_command(&command) {
                MetaCommandResult::SUCCESS => {}
                MetaCommandResult::EXIT => return,
                MetaCommandResult::UNRECOGNIZED => {
                    println!("Unknown command {:?}", command);
                }
            }
        } else {
            if let Some(statement) = prepare_statement(&command) {
                execute_statement(&statement);
            } else {
                println!("Unrecognized keyword at start of '{}'", command);
            }
        }

        print_prompt(&mut stdout);
    }
}

fn print_prompt(stdout: &mut Stdout) {
    print!("db > ");
    stdout.flush().unwrap();
}

fn execute_meta_command(command: &str) -> MetaCommandResult {
    match command {
        ".exit" => MetaCommandResult::EXIT,
        _ => MetaCommandResult::UNRECOGNIZED,
    }
}

enum MetaCommandResult {
    SUCCESS,
    EXIT,
    UNRECOGNIZED,
}

struct Statement {
    kind: StatementKind,
}

enum StatementKind {
    INSERT,
    SELECT,
}

fn prepare_statement(command: &str) -> Option<Statement> {
    if command.starts_with("select") {
        Some(Statement {
            kind: StatementKind::SELECT,
        })
    } else if command.starts_with("insert") {
        Some(Statement {
            kind: StatementKind::INSERT,
        })
    } else {
        None
    }
}

fn execute_statement(statement: &Statement) {
    match statement.kind {
        StatementKind::INSERT => println!("Handle insert"),
        StatementKind::SELECT => println!("Handle select"),
    }
}
