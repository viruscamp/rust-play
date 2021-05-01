use std::thread::spawn;
use std::thread::sleep;
use std::io::{Result, Error, ErrorKind, BufReader, Write, BufRead, copy};
use std::time::Duration;
use std::sync::mpsc::channel;
use std::fs::File;
use std::net::{TcpListener, TcpStream};


fn main() -> Result<()> {
    let (dispatch_sender, dispatch_receiver) = channel::<DispatchMessage>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port))?;
    let dispatch_sender1 = dispatch_sender.clone();
    let accept_loop = spawn(move || {
        while let Ok((stream, addr)) = listener.accept() {
            dispatch_sender1.send(DispatchMessage::Connected(stream)).unwrap();
        }
    });
    println!("server started at http://{}:{}/ serving files in {:?}", local_host, port, std::env::current_dir().unwrap_or_default());

    while let Ok(dispatch_message) = dispatch_receiver.recv() {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let dispatch_sender = dispatch_sender.clone();
                spawn(move || {
                    if let Ok(RequestResult::Quit) = handle_connection(stream) {
                        dispatch_sender.send(DispatchMessage::Quit).unwrap();
                    }
                });
            }
            DispatchMessage::Quit => { break; }
        }
    }

    //accept_loop.join();
    Ok(())
}

#[derive(Debug)]
enum DispatchMessage {
    Connected(TcpStream),
    Quit,
}

enum RequestResult {
    Ok,
    Quit,
}

fn handle_connection(mut stream: TcpStream) -> Result<RequestResult> {
    let mut str = String::new();
    BufReader::new(&stream).read_line(&mut str)?;

    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        return Err(Error::from(ErrorKind::InvalidInput));
    }
    let method = strsubs[0];
    let path = strsubs[1];

    let (path, query) = match path.find("?") {
        Some(pos) => (&path[..pos], &path[(pos+1)..]),
        None => (path, ""),
    };

    if query == "sleep" {
        sleep(Duration::new(4, 0));
    }

    if path == "/" {
        write!(stream, "HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome</body></html>")?;
    } else {
        let relative_path = match path.strip_prefix("/") {
            Some(p) => p,
            None => path,
        };
        match File::open(relative_path) {
            Ok(mut f) => {
                write!(stream, "HTTP/1.1 200 OK\r\n\r\n")?;
                copy(&mut f, &mut stream)?;
            }
            Err(err) => {
                write!(stream, "HTTP/1.1 404 NOT FOUND\r\n\r\n<html><body>Not Found {}</body></html>", path)?;
            }
        }
    }
    stream.flush()?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}
