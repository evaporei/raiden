use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::str::from_utf8;

use rustyline::error::ReadlineError;

fn repl(mut stream: TcpStream) -> io::Result<()> {
    // this is bad only for windows
    // but who cares...
    #[allow(deprecated)]
    let mut folder = std::env::home_dir().unwrap();
    folder.push(".raiden");
    let mut history = folder;
    history.push("history.txt");

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    let _ = rl.load_history(&history);

    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let input = match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                line
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("readline error: {:?}", err);
                break;
            }
        };

        stream.write_all(input.as_bytes()).unwrap();
        stream.write_all(b"\n").unwrap();
        let mut res = Vec::with_capacity(256);
        reader.read_until(b'\n', &mut res).unwrap();
        let out = from_utf8(&res).unwrap();
        let out = out.trim();
        println!("{out}");
    }

    let _ = rl.save_history(&history);
    Ok(())
}

fn main() {
    match TcpStream::connect("localhost:6969") {
        Ok(stream) => {
            println!("successfully connected to server in port 6969");

            repl(stream).unwrap();
        }
        Err(e) => {
            println!("failed to connect: {}", e);
        }
    }
}
