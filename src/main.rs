use std::io;
use std::io::{BufRead, Stdout, Write};

fn main() {
    let mut stdout = io::stdout();
    print_prompt(&mut stdout);

    for line in io::stdin().lock().lines() {
        let command = line.unwrap();
        match command.as_str() {
            ".exit" => return,
            _ => println!("Unknown command {:?}", command),
        }

        print_prompt(&mut stdout);
    }
}

fn print_prompt(stdout: &mut Stdout) {
    print!("db > ");
    stdout.flush().unwrap();
}
