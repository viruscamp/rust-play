use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::path::Path;
use async_std::fs::File;
use async_std::net::{TcpListener, TcpStream};
use async_std::io::{BufReader, prelude::*};
use futures::{select, FutureExt};

use async_std::task::spawn;
use async_std::task::sleep;
use async_std::io::copy as stream_copy;

#[async_std::main]
async fn main() -> Result<(), Error> {
    println!("http-server using async-std starting");

    let (tx, rx) = async_std::channel::unbounded::<DispatchMessage>();
    let (kill_switch, kill_switch_receiver) = async_std::channel::bounded::<()>(1);

    let local_host = "127.0.0.1";
    let port = 20085;
    let tx1 = tx.clone();

    let accept_loop_join = match TcpListener::bind((local_host, port)).await {
        Ok(listener)  => {
            println!("server started at http://{}:{}/", local_host, port);
            spawn(async move {
                // listen loop
                loop {
                    select! {
                        connected = listener.accept().fuse() => match connected {
                            Ok((stream,_)) => {
                                tx1.send(DispatchMessage::Connected(stream)).await.unwrap_or_default();
                            }
                            Err(_) => { /* connection failed */ }
                        },
                        _ = kill_switch_receiver.recv().fuse() => {
                            println!("terminating accept loop");
                            break;
                        },
                    }
                }
            })
        }
        Err(err) => {
            println!("http-server starting failed");
            tx1.send(DispatchMessage::Quit).await.unwrap_or_default();
            return Err(err);
        }
    };

    // dispatch loop
    while let Ok(dispatch_message) = rx.recv().await {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let tx = tx.clone();
                let job = async move {
                    match handle_connection(stream).await {
                        Ok(RequestResult::Quit) => {
                            tx.send(DispatchMessage::Quit).await.unwrap_or_default();
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

    kill_switch.send(()).await;
    accept_loop_join.await;

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