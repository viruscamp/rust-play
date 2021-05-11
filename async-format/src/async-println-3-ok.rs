use async_std::task::spawn;
use async_std::io::prelude::*;

#[async_std::main]
async fn main() {
    spawn(async {
        let mut stdout = async_std::io::stdout();
        let mut string = String::new();

        std::fmt::write(&mut string, format_args!("spawn {}", 3)).unwrap();

        stdout.write_all(string.as_bytes()).await;
    });
}