mod thread_pool;
use thread_pool::ThreadPool;

use std::{fs::File, io::{BufRead, BufReader, Error, ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, path::Path, sync::mpsc::channel, thread, time::Duration};

fn main() -> Result<(), Error> {
    println!("http-server starting");

    let (tx, rx) = channel::<TcpData>();

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:83").unwrap();
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        tx.send(TcpData::Connected(stream)).unwrap_or_default();
                    }
                    Err(_) => { /* connection failed */ }
                }
            }
        });
    }

    let mut tp = ThreadPool::new(2);

    for ss in rx.iter() {
        match ss {
            TcpData::Connected(stream) => {
                let tx = tx.clone();
                let handler = move || match handle_connection(stream) {
                    Ok(RequestResult::Quit) => {
                        tx.send(TcpData::Quit).unwrap_or_default();
                    }
                    Err(err) => {
                        println!("error: {}", err);
                    }
                    _ => {}
                };
                tp.execute(handler);
                //thread::spawn(handler);
            }
            TcpData::Quit => {
                break;
            }
        }
    }

    println!("http-server quitting");
    Ok(())
}

enum TcpData {
    Connected(TcpStream),
    Quit,
}

enum RequestResult {
    Ok,
    Quit,
}

fn handle_connection(mut stream: TcpStream) -> Result<RequestResult, Error> {
    let mut ins = BufReader::new(stream);
    let mut str = String::new();
    ins.read_line(&mut str)?;

    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        return Err(Error::new(ErrorKind::InvalidInput, "http request format error"));
    }
    let method = strsubs[0];
    let uri = strsubs[1];

    println!("method: {} uri: {}", method, uri);

    let (path, query) = match uri.split_once("?") {
        Some(a) => a,
        None => (uri, ""),
    };

    if query == "sleep" {
        let now = std::time::Instant::now();
        thread::sleep(Duration::new(20, 0));
        println!("sleep for {} seconds", now.elapsed().as_secs());
    }

    stream = ins.into_inner();

    if path == "/" {
        write!(stream, r#"HTTP/1.1 200 OK

<html>
<body>
<h1>Welcom to {}</h1>
</body>
</html>"#, uri)?;
    } else {
        let root = Path::new("E:\\");
        let fullpath = match path.strip_prefix("/") {
            Some(p) => root.join(p),
            None => root.join(path),
        };
        match File::open(&fullpath) {
            Ok(mut f) => {
                println!("File {:?} opened", fullpath);
                write!(stream, "HTTP/1.1 200 OK\n\n")?;
                let mut buf = [0 as u8; 1024];
                loop {
                    let len = f.read(&mut buf)?;
                    if len == 0 {
                        break;
                    }
                    stream.write(&buf[..len])?;
                }
            }
            Err(err) => {
                write!(stream, r#"HTTP/1.1 404 NOT FOUND

<html>
<body>
<h1>not found: {}</h1>
<span>{}</span>
</body>
</html>"#, uri, err)?;
            }
        }
    }
    stream.flush()?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}
