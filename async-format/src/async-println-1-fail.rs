use async_std::task::spawn;
use async_std::io::prelude::*;

#[async_std::main]
async fn main() {
    async_std::println!("main {}", 1).await;

    async {
        async_std::println!("spawn {}", 1).await;
    }.await;

    // compile error
    spawn(async {
        async_std::println!("spawn {}", 1).await;
    });
}