use std::{borrow::Borrow, cell::RefCell, fmt::Display, rc::Rc, sync::{Arc, Mutex}, thread};

use std::any::type_name;

extern crate crossbeam;

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

    ok_send_arc();
    ok_sync_mutex();
}

fn test_send<T: Send + Display + 'static>(t: T) {
    thread::spawn(move || {
        println!("test_send: type={} value={}", type_of(&t), t);
    }).join().unwrap();
}

fn test_sync<T: Sync + Display>(t: &'static T) {
    thread::spawn(move || {
        println!("test_sync: type={} value={}", type_of(&t), t);
    }).join().unwrap();
}

fn test_sync_send<T: Sync + Display>(t: &'static T) {
    print!("test_sync_send: ");
    test_send(t);
}

fn ok_send_arc() {
    let arc = Arc::new(33); // Send
    crossbeam::scope(move |s| {
        s.spawn(move |_| {
            println!("send_arc: type={} value={}", type_of(&arc), *arc);
            drop(arc);
        }).join().unwrap();
    }).unwrap();
}

fn ok_sync_mutex() {
    let mutex = Mutex::new(34); // Sync
    crossbeam::scope(|s| {
        s.spawn(|_| {
            println!("sync_mutex: type={} value={}", type_of(&mutex), *Mutex::lock(&mutex).unwrap());
        }).join().unwrap();
    }).unwrap();
}

/*
fn fail_send_rc() {
    let rc = Rc::new(31); // 未实现 Send
    crossbeam::scope(move |s| {
        s.spawn(move |_| {
            println!("test_sync: type={} value={}", type_of(&rc), *rc);
            drop(rc);
        }).join().unwrap();
    }).unwrap();
}

fn fail_sync_ref_cell() {
    let ref_cell = RefCell::new(32); // 未实现 Sync
    crossbeam::scope(|s| {
        s.spawn(|_| {
            println!("test_sync: type={} value={}", type_of(&ref_cell), *ref_cell.borrow());
        }).join().unwrap();
    }).unwrap();
}
*/