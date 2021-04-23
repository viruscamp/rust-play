mod thread_pool;
use thread_pool::ThreadPool;
use std::thread;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::path::Path;
use std::sync::mpsc;
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, Read, Write, BufRead};

use thread::spawn;
use thread::sleep;

fn main() -> Result<(), Error> {
    println!("http-server using threadpool starting");

    let (tx, rx) = mpsc::channel::<DispatchMessage>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let tx1 = tx.clone();
    match TcpListener::bind((local_host, port)) {
        Ok(listener) => {
            println!("server started at http://{}:{}/", local_host, port);
            spawn(move || {
                // listen loop
                // TODO how to exit
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            tx1.send(DispatchMessage::Connected(stream)).unwrap_or_default();
                        }
                        Err(_) => { /* connection failed */ }
                    }
                }
            });
        }
        Err(err) => {
            println!("http-server starting failed");
            //tx1.send(DispatchMessage::Quit).unwrap(); // useless
            return Err(err);
        }
    }

    let mut tp = ThreadPool::new(2);
    // dispatch loop
    while let Ok(dispatch_message) = rx.recv() {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let tx = tx.clone();
                let job = move || {
                    match handle_connection(stream) {
                        Ok(RequestResult::Quit) => {
                            tx.send(DispatchMessage::Quit).unwrap_or_default();
                        }
                        Err(err) => {
                            println!("error: {}", err);
                        }
                        _ => {}
                    }
                };
                tp.execute(job);
                //spawn(job);
            }
            DispatchMessage::Start => {}
            DispatchMessage::Quit => {
                break;
            }
        }
    }

    println!("http-server quitting");
    Ok(())
}

#[derive(Debug)]
enum DispatchMessage {
    Connected(TcpStream),
    Quit,
    Start,
}

#[derive(Debug)]
enum RequestResult {
    Ok,
    Quit,
}

fn handle_connection(mut stream: TcpStream) -> Result<RequestResult, Error> {
    let mut ins = BufReader::new(&stream);
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
        sleep(Duration::new(20, 0));
        println!("sleep for {} seconds", now.elapsed().as_secs());
    }

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
                stream_copy(&mut f, &mut stream)?;
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

fn stream_copy(r: &mut impl Read, w: &mut impl Write) -> Result<usize, Error> {
    let mut buf = [0u8; 1024];
    let mut sz: usize = 0;
    return loop {
        let len = r.read(&mut buf)?;
        //if len == 0 { break Ok(sz) } // 每次都会多读一个 0
        w.write(&buf[..len])?;
        sz += len;
        if len < buf.len() { break Ok(sz) } // 大部分情况 ok , 输入长度 = buflen*n 时 多读多写一个 0
    }
}
