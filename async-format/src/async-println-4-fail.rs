use async_std::task::spawn;
use async_std::io::prelude::*;

#[async_std::main]
async fn main() {
    spawn(async {
        let mut stdout = async_std::io::stdout();
        let mut string = String::new();

        {
            let fmt = format_args!("spawn {}", 4);
            std::fmt::write(&mut string, fmt).unwrap();
        }

        stdout.write_all(string.as_bytes()).await;
    });
}