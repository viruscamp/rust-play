use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::path::Path;
use async_std::fs::File;
use async_std::net::{TcpListener, TcpStream};
use async_std::io::BufReader;

use async_std::task::spawn;
use async_std::task::sleep;
use async_std::io::copy as stream_copy;

use async_std::io::prelude::*;

#[async_std::main]
async fn main() -> Result<(), Error> {
    async_std::println!("main 1").await;

    test(1).await;

    //spawn(test(2));

    spawn(async {
        let mut stdout = async_std::io::stdout();
        let mut string = String::new();

        std::fmt::write(&mut string, format_args!("{}", 1)).unwrap();

        stdout.write_all(string.as_bytes()).await;
    });
    /*
    spawn(async {
        let mut stdout = async_std::io::stdout();
        let mut string = String::new();

        {
            let fmt = format_args!("{}", 1);
            std::fmt::write(&mut string, fmt).unwrap();
        }

        stdout.write_all(string.as_bytes()).await;
    });

    spawn(async {
        let mut stdout = async_std::io::stdout();
        let mut string = String::new();

        let fmt = format_args!("{}", 1);
        std::fmt::write(&mut string, fmt).unwrap();

        stdout.write_all(string.as_bytes()).await;
    });

    spawn(async {
        let fmt = format_args!("{}", 1);
        let x = async_std::io::stdout().write_fmt(fmt);
        x.await;
    });

    spawn(async {
        let fmt = format_args!("{}", 1);
        let x = async {
            if let Err(e) = async_std::io::stdout().write_fmt(fmt).await {
                panic!("failed printing to stdout: {}", e);
            }
        };
        x.await;
    });

    spawn(async {
        let fmt = format_args!("{}", 1);
        let x = async_std::io::_print(fmt);
        x.await;
    });

    spawn(async {
        async_std::io::_print(format_args!("{}", 1)).await;
    });

    spawn(async {
        let x = async_std::io::_print(format_args!("{}", 1));
        x.await;
    });

    */
    Ok(())
}

async fn test(n: i32) {
    async_std::println!("test {}", n).await;
}