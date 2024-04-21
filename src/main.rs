use std::{collections::HashMap, fs, io, os::unix::fs::MetadataExt};

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

use rustyline::error::ReadlineError;

fn main() -> io::Result<()> {
    // this is bad only for windows
    // but who cares...
    #[allow(deprecated)]
    let mut folder = std::env::home_dir().unwrap();
    folder.push(".raiden");
    let mut file = folder.clone();
    file.push("data");
    let mut history = folder.clone();
    history.push("history.txt");

    let exists = std::path::Path::new(&file).exists();
    if !exists {
        fs::create_dir_all(folder)?;
        fs::File::create(&file)?;
        fs::File::create(&history)?;
    }

    let size = fs::metadata(&file)?.size();
    let mut store = if size == 0 {
        fs::write(&file, "{}")?;
        HashMap::new()
    } else {
        let contents = fs::read_to_string(&file)?;
        ron::from_str(&contents).unwrap()
    };

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    let _ = rl.load_history(&history);

    loop {
        let input = match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                line
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("error: {:?}", err);
                break
            }
        };
        let cmd = Cmd::parse(&input);
        match cmd {
            Some(Cmd::Get(k)) => {
                let contents = fs::read_to_string(&file)?;
                store = ron::from_str(&contents).unwrap();
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
                fs::write(&file, &ron::to_string(&store).unwrap())?;
            },
            None => {},
        };
    };

    let _ = rl.save_history(&history);
    Ok(())
}
