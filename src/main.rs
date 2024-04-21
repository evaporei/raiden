use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut input = String::new();
    let stdin = io::stdin();
    loop {
        input.clear();
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        print!("{input}");
    }
}
