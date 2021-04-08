mod thread_pool;
use thread_pool::ThreadPool;

use std::{io::{BufRead, BufReader, Error, Write}, net::{TcpListener, TcpStream}, sync::mpsc::channel, thread, time::Duration};

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
                        tx.send(TcpData::Connected(stream));
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
                tp.execute(move || {
                    if let RequestResult::Quit = handle_connection(stream) {
                        tx.send(TcpData::Quit);
                    }
                });
            }
            TcpData::Quit => { break; }
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
    Err,
    Quit,
}

fn handle_connection(mut stream: TcpStream) -> RequestResult {
    let mut ins = BufReader::new(stream);
    let mut str = String::new();
    ins.read_line(&mut str);

    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        println!("http request error");
        return RequestResult::Err;
    }
    let method = strsubs[0];
    let path = strsubs[1];

    println!("method: {} path: {}", method, path);

    if path == "/sleep" {
        thread::sleep(Duration::new(8, 0));
    }

    stream = ins.into_inner();
    write!(stream, r#"HTTP/1.1 200 OK

<html>
<body>
{}
</body>
</html>"#, path);

    if path == "/quit" {
        return RequestResult::Quit;
    }
    return RequestResult::Ok;
}
