use async_std::task::spawn;
use async_std::io::prelude::*;

macro_rules! myprintln {
    () => (async_std::print!("\n"));
    ($($arg:tt)*) => (async {
        async_std::io::_print(format_args!($($arg)*)).await;
        async_std::io::_print(format_args!("\n")).await;
    })
}

// 最终解决方案 要改 async_std::io::stdio::_print 和 async_std::io::stdio::_eprint
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

#[async_std::main]
async fn main() {
    myprintln!("main {}", 1).await;

    async {
        myprintln!("spawn {}", 1).await;
    }.await;

    async {
        writeln!(async_std::io::stdout(), "spawn write {}", 1).await.unwrap();
    }.await;

    println!("f {}", 4);
}