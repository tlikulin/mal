use std::io::{self, Write};

mod printer;
mod reader;
mod types;

use types::MalType;

fn main() -> io::Result<()> {
    loop {
        print!("user> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.is_empty() {
            println!();
            return Ok(());
        } else {
            let output = rep(input);
            println!("{output}");
        }
    }
}

#[allow(non_snake_case)]
fn READ(input: String) -> Result<MalType, String> {
    reader::read_str(input)
}

#[allow(non_snake_case)]
fn EVAL(mal: MalType) -> MalType {
    // dbg!(mal)
    mal
}

#[allow(non_snake_case)]
fn PRINT(mal: MalType) -> String {
    printer::pr_str(mal)
}

fn rep(input: String) -> String {
    match READ(input) {
        Ok(mal) => PRINT(EVAL(mal)),
        Err(e) => format!("mal: parse error: {e}"),
    }
}
