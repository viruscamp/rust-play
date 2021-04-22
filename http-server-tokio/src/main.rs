use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::path::Path;
use tokio::sync::mpsc;
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufReader, AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt, AsyncBufRead, AsyncBufReadExt};

use tokio::spawn;
use tokio::time::sleep;
use tokio::io::copy as stream_copy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("http-server using tokio starting");

    let (tx, mut rx) = mpsc::unbounded_channel::<DispatchMessage>();

    let port = 20084;
    let tx1 = tx.clone();
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener)  => {
            spawn(async move {
                // listen loop
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            tx1.send(DispatchMessage::Connected(stream)).unwrap_or_default();
                        },
                        Err(_) => { /* connection failed */ }
                    }
                }
            });
        }
        Err(err) => {
            println!("http-server starting failed");
            //tx1.send(DispatchMessage::Quit).unwrap();
            //return Err(Box::new(err)); // fail
            //return Err(Box::<dyn std::error::Error>::new(err)); // fail too
            let err: Box<dyn std::error::Error> = Box::new(err);
            return Err(err);
        }
    }

    // dispatch loop
    while let Some(dispatch_message) = rx.recv().await {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let tx = tx.clone();
                let job = async move {
                    match handle_connection(stream).await {
                        Ok(RequestResult::Quit) => {
                            tx.send(DispatchMessage::Quit).unwrap_or_default();
                        }
                        Err(err) => {
                            println!("error: {}", err);
                        }
                        _ => {}
                    }
                };
                spawn(job);
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

async fn handle_connection(mut stream: TcpStream) -> Result<RequestResult, Error> {
    let mut ins = BufReader::new(stream);
    let mut str = String::new();
    ins.read_line(&mut str).await?;

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
        sleep(Duration::new(20, 0)).await;
        println!("sleep for {} seconds", now.elapsed().as_secs());
    }

    stream = ins.into_inner();

    if path == "/" {
        let str = format!(r#"HTTP/1.1 200 OK

<html>
<body>
<h1>Welcom to {}</h1>
</body>
</html>"#, uri);
        stream.write_all(str.as_bytes()).await?;
    } else {
        let root = Path::new("E:\\");
        let fullpath = match path.strip_prefix("/") {
            Some(p) => root.join(p),
            None => root.join(path),
        };
        match File::open(&fullpath).await {
            Ok(mut f) => {
                println!("File {:?} opened", fullpath);
                let str = "HTTP/1.1 200 OK\n\n";
                stream.write_all(str.as_bytes()).await?;
                stream_copy(&mut f, &mut stream).await?;
            }
            Err(err) => {
                let str = format!(r#"HTTP/1.1 404 NOT FOUND

<html>
<body>
<h1>not found: {}</h1>
<span>{}</span>
</body>
</html>"#, uri, err);
                stream.write_all(str.as_bytes()).await?;
            }
        }
    }
    stream.flush().await?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}