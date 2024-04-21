use std::{collections::HashMap, io::{self, Write}};

#[derive(Debug)]
enum Cmd {
    Get(String),
    Set(String, String),
}

impl Cmd {
    fn parse(input: &str) -> Option<Self> {
        let mut words = input.split_whitespace();
        match words.next() {
            Some("get") => Some(Self::Get(words.next()?.to_owned())),
            Some("set") => {
                let mut kv = words.next().map(|kv| kv.split('='))?;
                let k = kv.next()?.to_owned();
                let v = kv.next()?.to_owned();
                Some(Self::Set(k, v))
            },
            Some(_) | None => None
        }
    }
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut store = HashMap::new();
    let mut input = String::new();
    loop {
        input.clear();
        io::stdout().flush()?;
        stdin.read_line(&mut input)?;
        let cmd = Cmd::parse(&input);
        match cmd {
            Some(Cmd::Get(k)) => {
                let v = store.get(&k);
                println!("value: {v:?}");
            },
            Some(Cmd::Set(k, v)) => {
                let insert = store.insert(k, v).is_none();
                if insert {
                    println!("inserted key");
                } else {
                    println!("updated key");
                }
            },
            None => {},
        };
    }
}
