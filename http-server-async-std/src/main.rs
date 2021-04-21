use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::path::Path;
use std::sync::mpsc;
use async_std::io::prelude::*;
use async_std::fs::File;
use async_std::net::{TcpListener, TcpStream};
use async_std::io::BufReader;

fn main() -> Result<(), Error> {
    println!("http-server using async-std starting");

    let (tx, rx) = mpsc::channel::<QuitMessage>();

    async_std::task::spawn(async move {
        if let Ok(listener) = TcpListener::bind("127.0.0.1:20085").await {
            loop {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        let tx = tx.clone();
                        let job = async move {
                            match handle_connection(socket).await {
                                Ok(RequestResult::Quit) => {
                                    tx.send(QuitMessage).unwrap_or_default();
                                }
                                Err(err) => {
                                    println!("error: {}", err);
                                }
                                _ => {}
                            }
                        };
                        async_std::task::spawn(job);
                    },
                    Err(_) => { /* connection failed */ }
                }
            }
        } else {
            println!("http-server starting failed");
            tx.send(QuitMessage).unwrap();
        }
    });

    rx.recv().unwrap_or(QuitMessage);
    println!("http-server quitting");
    Ok(())
}

struct QuitMessage;

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
        async_std::task::sleep(Duration::new(20, 0)).await;
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
                async_std::io::copy(&mut f, &mut stream).await?;
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