use std::{collections::HashMap, fs, io, os::unix::fs::MetadataExt};

#[derive(Debug)]
enum Cmd {
    Get(String),
    Set(String, String),
}

impl Cmd {
    fn parse(input: &str) -> Result<Self, String> {
        let mut words = input.split_whitespace();
        match words.next() {
            Some("get") => {
                let k = words.next().ok_or("get requires string")?.to_owned();
                Ok(Self::Get(k))
            }
            Some("set") => {
                let mut kv = words
                    .next()
                    .map(|kv| kv.split('='))
                    .ok_or("set argument needs to be in format key=value")?;
                let k = kv
                    .next()
                    .ok_or("missing key in set command, example: set key=value")?
                    .to_owned();
                let v = kv
                    .next()
                    .ok_or("missing value in set command, example: set key=value")?
                    .to_owned();
                Ok(Self::Set(k, v))
            }
            Some(_) | None => Err("unknown command".to_owned()),
        }
    }
}

use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

type Store = HashMap<String, String>;
type SyncStore = Arc<Mutex<Store>>;

fn handle_client(store: SyncStore, file: &PathBuf, mut stream: TcpStream) {
    let mut data = Vec::with_capacity(256);
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    while match reader.read_until(b'\n', &mut data) {
        Ok(_size) => {
            let input = std::str::from_utf8(&data).unwrap();
            let input = input.trim();
            let cmd = Cmd::parse(&input);
            println!("input: {input}");
            println!("parsed: {cmd:?}");
            let out = match cmd {
                Ok(Cmd::Get(k)) => {
                    let mut store = store.lock().unwrap();
                    let contents = fs::read_to_string(&file).unwrap();
                    let map2: Store = ron::from_str(&contents).unwrap();
                    *store = map2;
                    let v = format!("{:?}", store.get(&k));
                    drop(store);
                    v
                }
                Ok(Cmd::Set(k, v)) => {
                    let mut store = store.lock().unwrap();
                    let insert = store.insert(k, v).is_none();

                    fs::write(&file, &ron::to_string(&*store).unwrap()).unwrap();
                    drop(store);
                    if insert {
                        format!("inserted key")
                    } else {
                        format!("updated key")
                    }
                }
                Err(err) => {
                    format!("{err}")
                }
            };
            stream.write_all(&out.as_bytes()).unwrap();
            stream.write_all(b"\n").unwrap();
            println!("output: {out}");
            data.clear();
            true
        }
        Err(_) => {
            println!(
                "an error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

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
    let map = if size == 0 {
        fs::write(&file, "{}")?;
        HashMap::new()
    } else {
        let contents = fs::read_to_string(&file)?;
        ron::from_str(&contents).unwrap()
    };
    let store = Arc::new(Mutex::new(map));
    let file = Arc::new(file);

    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
    println!("server listening on port 6969");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new connection: {}", stream.peer_addr().unwrap());
                let s = store.clone();
                let f = file.clone();
                thread::spawn(move || {
                    handle_client(s, &f, stream);
                });
            }
            Err(e) => {
                println!("conn error: {}", e);
            }
        }
    }

    Ok(())
}
