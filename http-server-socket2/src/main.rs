mod thread_pool;
use thread_pool::ThreadPool;
use std::thread;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::path::Path;
use std::sync::mpsc;
use std::fs::File;
use std::sync::Arc;
use std::net::{IpAddr, SocketAddr, TcpStream, Shutdown};
use socket2::{Socket, Domain, Type};
use std::io::{BufReader, Read, Write, BufRead};

use thread::spawn;
use thread::sleep;

fn main() -> Result<(), Error> {
    println!("http-server using socket2 and threadpool starting");

    let (tx, rx) = mpsc::channel::<DispatchMessage>();

    let local_host = "127.0.0.1";
    let port = 20086;
    let tx1 = tx.clone();

    let (socket, accept_loop) = match create_socket2(local_host, port) {
        Ok(socket) => {
            let socket = Arc::new(socket);
            let socket1 = socket.clone();
            println!("server started at http://{}:{}/", local_host, port);
            let accept_loop = spawn(move || {
                loop {
                    let connected = socket.accept();
                    match connected {
                        Ok((socket, _)) => {
                            tx1.send(DispatchMessage::Connected(socket.into())).unwrap_or_default();
                        }
                        Err(_) => {
                             /* connection failed */ 
                             break;
                        }
                    }
                }
                println!("accept_loop ends");
            });
            (socket1, accept_loop)
        }
        Err(err) => {
            println!("http-server starting failed");
            //tx1.send(DispatchMessage::Quit).unwrap(); // useless
            return Err(err);
        }
    };

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

    println!("try shutdown");
    socket.shutdown(Shutdown::Both);
    println!("try join");
    accept_loop.join();

    println!("http-server quitting");
    Ok(())
}

fn create_socket2(ip: &str, port: u16) -> Result<Socket, Error> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    let ip: IpAddr = ip.parse().unwrap();
    let address = SocketAddr::new(ip, port);
    let address = address.into();
    socket.bind(&address)?;
    socket.listen(128)?;

    Ok(socket)
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
        //if len == 0 { break Ok(sz) } // ???????????????????????? 0
        w.write(&buf[..len])?;
        sz += len;
        if len < buf.len() { break Ok(sz) } // ??????????????? ok , ???????????? = buflen*n ??? ?????????????????? 0
    }
}
