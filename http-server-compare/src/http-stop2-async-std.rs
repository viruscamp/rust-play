use async_std::task::spawn;
use async_std::task::sleep;
use async_std::io::{Result, Error, ErrorKind, BufReader, copy, prelude::*};
use std::time::Duration;
use async_std::channel::unbounded as channel;
use async_std::fs::File;
use async_std::net::{TcpListener, TcpStream};
use futures::{select, FutureExt};

#[async_std::main]
async fn main() -> Result<()> {
    let (kill_switch, mut kill_switch_receiver) = channel::<()>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port)).await?;
    let accept_loop = spawn(async move {
        select! {
            _ = async {
                while let Ok((stream, addr)) = listener.accept().await {
                    let kill_switch = kill_switch.clone();
                    spawn(async move {
                        if let Ok(RequestResult::Quit) = handle_connection(stream).await {
                            kill_switch.send(());
                        }
                    });
                }
            }.fuse() => {}
            _ = kill_switch_receiver.recv().fuse() => {}
        }
    });
    println!("server started at http://{}:{}/ serving files in {:?}", local_host, port, std::env::current_dir().unwrap_or_default());

    accept_loop.await;
    Ok(())
}

enum RequestResult {
    Ok,
    Quit,
}

async fn handle_connection(mut stream: TcpStream) -> Result<RequestResult> {
    let mut str = String::new();
    BufReader::new(&mut stream).read_line(&mut str).await?;

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
        sleep(Duration::new(4, 0)).await;
    }

    if path == "/" {
        stream.write("HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome</body></html>".as_bytes()).await?;
    } else {
        let relative_path = match path.strip_prefix("/") {
            Some(p) => p,
            None => path,
        };
        match File::open(relative_path).await {
            Ok(mut f) => {
                stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await?;
                copy(&mut f, &mut stream).await?;
            }
            Err(err) => {
                stream.write(format!("HTTP/1.1 404 NOT FOUND\r\n\r\n<html><body>Not Found {}</body></html>", path).as_bytes()).await?;
            }
        }
    }
    stream.flush().await?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}
