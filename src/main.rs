use regex::Regex;
use std::fmt;
use std::io;
use std::io::{BufRead, Stdout, Write};
use std::mem;

const COLUMN_USERNAME_SIZE: usize = 32;
const COLUMN_EMAIL_SIZE: usize = 255;
const ROW_SIZE: usize = mem::size_of::<Row>();
const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

fn main() {
    let mut table = Table::new();
    let mut stdout = io::stdout();
    print_prompt(&mut stdout);

    for line in io::stdin().lock().lines() {
        let command = line.unwrap();

        if command.starts_with(".") {
            match execute_meta_command(&command) {
                MetaCommandResult::Ok => {}
                MetaCommandResult::Exit => return,
                MetaCommandResult::Unrecognized => {
                    println!("Unknown command {:?}", command);
                }
            }
        } else {
            match prepare_statement(&command) {
                PrepareResult::Ok(statement) => match table.execute_statement(&statement) {
                    ExecuteResult::Ok => println!("Complete"),
                    ExecuteResult::TableFull => println!("Table full!"),
                },
                PrepareResult::SyntaxError => println!("Syntax error. Could not parse statement."),
                PrepareResult::UnrecognizedStatement => println!("Unrecognized statement"),
                PrepareResult::StringTooLong => println!("String is too long."),
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
        ".exit" => MetaCommandResult::Exit,
        _ => MetaCommandResult::Unrecognized,
    }
}

struct Statement {
    kind: StatementKind,
    row_to_insert: Option<Row>,
}
enum StatementKind {
    Insert,
    Select,
}

enum PrepareResult {
    Ok(Statement),
    StringTooLong,
    SyntaxError,
    UnrecognizedStatement,
}

enum ExecuteResult {
    TableFull,
    Ok,
}
enum MetaCommandResult {
    Ok,
    Exit,
    Unrecognized,
}

struct Table {
    num_rows: usize,
    pages: Vec<Page>,
}
impl Table {
    pub fn new() -> Self {
        Table {
            num_rows: 0,
            pages: Vec::with_capacity(TABLE_MAX_PAGES),
        }
    }

    pub fn execute_statement(&mut self, statement: &Statement) -> ExecuteResult {
        match statement.kind {
            StatementKind::Insert => self.execute_insert(statement),
            StatementKind::Select => self.execute_select(statement),
        }
    }

    fn execute_insert(&mut self, statement: &Statement) -> ExecuteResult {
        if self.num_rows >= TABLE_MAX_ROWS {
            return ExecuteResult::TableFull;
        }

        let page_num = self.page_num(self.num_rows);
        let page_offset = self.page_offset(self.num_rows);

        if self.pages.get(page_num).is_none() {
            self.pages.insert(page_num, Page::new());
        }

        let page = self.pages.get_mut(page_num).unwrap();
        page.data[page_offset] = statement.row_to_insert.expect("No row given!");

        self.num_rows += 1;
        ExecuteResult::Ok
    }

    fn execute_select(&self, statement: &Statement) -> ExecuteResult {
        for i in (0..self.num_rows).into_iter() {
            let page_num = self.page_num(i);
            let page_offset = self.page_offset(i);
            if let Some(page) = self.pages.get(page_num) {
                println!("{}", page.data[page_offset])
            }
        }
        ExecuteResult::Ok
    }

    fn page_num(&self, row_index: usize) -> usize {
        row_index / ROWS_PER_PAGE
    }

    fn page_offset(&self, row_index: usize) -> usize {
        row_index % ROWS_PER_PAGE
    }
}

struct Page {
    data: Box<[Row; ROWS_PER_PAGE]>,
}
impl Page {
    pub fn new() -> Self {
        Page {
            data: Box::new([Row::blank(); ROWS_PER_PAGE]),
        }
    }
}

#[derive(Copy, Clone)]
struct Row {
    id: u32,
    username: [char; COLUMN_USERNAME_SIZE],
    email: [char; COLUMN_EMAIL_SIZE],
}
impl Row {
    pub fn blank() -> Self {
        Row {
            id: 0,
            username: ['\0'; COLUMN_USERNAME_SIZE],
            email: ['\0'; COLUMN_EMAIL_SIZE],
        }
    }
}
impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Must get rid of the the null characters that we pad the array with
        let username: String = self.username.iter().filter(|&c| *c != '\0').collect();
        let email: String = self.email.iter().filter(|&c| *c != '\0').collect();
        write!(f, "({}, '{}', '{}')", self.id, username, email)
    }
}

fn prepare_statement(command: &str) -> PrepareResult {
    if command.starts_with("insert") {
        let regex = Regex::new(r"insert (\d+) (\S+) (\S+)").unwrap();
        if let Some(captures) = regex.captures(command) {
            if captures.len() == 4 {
                let id_result = captures.get(1).unwrap().as_str().parse::<u32>().ok();

                if id_result.is_none() {
                    return PrepareResult::SyntaxError;
                };

                let id = id_result.unwrap();
                let username = captures.get(2).unwrap().as_str();
                let email = captures.get(3).unwrap().as_str();

                if username.len() > COLUMN_USERNAME_SIZE || email.len() > COLUMN_EMAIL_SIZE {
                    return PrepareResult::StringTooLong;
                }

                let mut username_arr: [char; COLUMN_USERNAME_SIZE] = ['\0'; COLUMN_USERNAME_SIZE];
                let mut email_arr: [char; COLUMN_EMAIL_SIZE] = ['\0'; COLUMN_EMAIL_SIZE];

                for i in 0..COLUMN_USERNAME_SIZE {
                    if let Some(character) = username.chars().nth(i) {
                        username_arr[i] = character;
                    } else {
                        break;
                    }
                }

                for i in 0..COLUMN_EMAIL_SIZE {
                    if let Some(character) = email.chars().nth(i) {
                        email_arr[i] = character;
                    } else {
                        break;
                    }
                }

                return PrepareResult::Ok(Statement {
                    row_to_insert: Some(Row {
                        id,
                        username: username_arr,
                        email: email_arr,
                    }),
                    kind: StatementKind::Insert,
                });
            }
        }

        return PrepareResult::SyntaxError;
    } else if command.starts_with("select") {
        return PrepareResult::Ok(Statement {
            row_to_insert: None,
            kind: StatementKind::Select,
        });
    }

    PrepareResult::UnrecognizedStatement
}
