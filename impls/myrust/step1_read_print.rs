use std::io::{self, Write};

mod printer;
mod reader;
mod types;

use types::MalType;

use crate::types::{MalError, MalResult};

fn main() -> io::Result<()> {
    loop {
        print!("user> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.is_empty() {
            println!();
            return Ok(());
        } else if let Some(output) = rep(&input) {
            println!("{output}");
        }
    }
}

#[allow(non_snake_case)]
fn READ(input: &str) -> MalResult {
    reader::read_str(input)
}

#[allow(non_snake_case)]
const fn EVAL(mal: MalType) -> MalType {
    mal
}

#[allow(non_snake_case)]
fn PRINT(mal: MalType) -> String {
    printer::pr_str(mal)
}

fn rep(input: &str) -> Option<String> {
    match READ(input) {
        Ok(mal) => Some(PRINT(EVAL(mal))),
        Err(MalError::EmptyInput) => None,
        Err(MalError::ParseError(msg)) => Some(format!("mal: parse error: {msg}")),
    }
}
