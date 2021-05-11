use async_std::task::spawn;
use async_std::io::prelude::*;

#[async_std::main]
async fn main() {
    spawn(async {
        let mut stdout = async_std::io::stdout();
        { let x = stdout.write_fmt(format_args!("spawn {}", 4)); x }.await;
    }).await;
}