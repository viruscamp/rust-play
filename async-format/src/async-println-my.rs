use async_std::task::spawn;
use async_std::io::prelude::*;
use std::io::prelude::*;
use std::io::Write;

macro_rules! myprintln {
    () => (_myprint(format_args!("\n")));
    ($($arg:tt)*) => (async {
        _myprint(format_args!($($arg)*)).await;
        _myprint(format_args!("\n")).await;
    })
}

// 错的 要改 async_std::io::stdio::_print 和 async_std::io::stdio::_eprint
// 前提条件是 async_std::io::write::write_fmt::WriteFmtFuture 不再持有 Arguments<'_>
pub async fn _myprint(args: std::fmt::Arguments<'_>) {
    let mut stdout = async_std::io::stdout(); // make stdout live longer than .await
    if let Err(e) = {
        let x = stdout.write_fmt(args); // drop args:Arguments<'_> and [ArgumentV1<'a>] to make them live shorter than .await
        x
    }.await {
        panic!("failed printing to stdout: {}", e);
    }
}

// 最终解决方案 要改 async_std::println 等4个宏
// 前提条件是 async_std::io::write::write_fmt::WriteFmtFuture 不再持有 Arguments<'_>
macro_rules! myprintln2 {
    ($($arg:tt)*) => (async {
        let mut stdout = async_std::io::stdout(); // make stdout live longer than .await
        if let Err(e) = {
            let x = writeln!(stdout, $($arg)*);
            // drop Arguments<'_> and [ArgumentV1<'a>] to make them live shorter than .await
            x
        }.await {
            panic!("failed printing to stdout: {}", e);
        }
    });
}

#[async_std::main]
async fn main() {
    myprintln2!("main {}", 1).await;

    spawn(async {
        myprintln2!("spawn {}", 1).await;
        myprintln2!().await;
    }).await;
    
    let mut stdout = std::io::stdout();
    writeln!(&mut stdout, "std sync write {}", 1).unwrap();

    let mut async_stdout = async_std::io::stdout();
    async {
        writeln!(&mut async_stdout, "spawn write {}", 1).await.unwrap();
    }.await;

    /*
    async {
        myprintln2().await;
        myprintln2("hi").await;
        myprintln2("hi {}", 4.5).await;
    }.await;
    */

    println!("f {}", 4);
}