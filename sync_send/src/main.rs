use std::{cell::RefCell, fmt::Display, thread};

use std::any::type_name;

fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

fn main() {
    static D1: i32 = 32;
    static D2: u8 = 8;
    static D3: f64 = 64.0;

    test_send(D1);
    test_sync(&D2);
    test_sync_send(&D3);
}

fn test_send<T: Send + Display + 'static>(t: T) {
    thread::spawn(move || {
        println!("test_send: type={} value={}", type_of(&t), t);
    }).join();
}

fn test_sync<T: Sync + Display>(t: &'static T) {
    thread::spawn(move || {
        println!("test_sync: type={} value={}", type_of(&t), t);
    }).join();
}

fn test_sync_send<T: Sync + Display>(t: &'static T) {
    print!("test_sync_send: ");
    test_send(t);
}

/*
fn test_sync_scoped<T: Sync + Display>(t: T) {
    let th = thread::scoped(|| {
        println!("{}", t);
        let xt = t;
    });
    th.join().unwrap();
    println!("{}", t);
}
*/
