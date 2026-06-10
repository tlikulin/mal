use std::io::{self, Write};

fn main() -> io::Result<()> {
    loop {
        print!("user> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.is_empty() {
            return Ok(());
        }
        let output = rep(input);
        print!("{output}");
    }
}

const fn read(input: String) -> String {
    input
}

const fn eval(input: String) -> String {
    input
}

const fn print(input: String) -> String {
    input
}

const fn rep(input: String) -> String {
    print(eval(read(input)))
}
