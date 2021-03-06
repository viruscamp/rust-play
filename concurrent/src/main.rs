#![feature(once_cell)]

use std::{borrow::{Borrow, BorrowMut}, cell::Cell, fmt::Display, rc::Rc, time::Duration};
use std::thread::{JoinHandle, ThreadId};
use std::sync::{Arc, Mutex, mpsc};
use std::ops::Deref;
use std::lazy::SyncLazy;
use std::cell::RefCell;
use std::thread;

fn main() {
    test4_arc_mutex();
}
// 多少线程不确定 线程运行时间不确定
// 无法确定什么时候该结束
// 0. 下面大部分方法都假定 12 个线程 或者 12 个返回数据
// 1. sleep 完全不可信
// 2. 收集所有 JoinHandle 然后 pop 再 join 似乎可行

fn test2_arc() {
    let a = Arc::new(20);
    let av = a.deref();
    // Arc 可以 clone 后 move 到其他线程
    // 使用时是只能用 deref 拿到不可变引用的 所以不能改
}

fn test3_arc_refcell() {
    let a = Arc::new(RefCell::new(30));
    println!("{}", a.take());
    //let am = a.get_mut();
    a.replace(a.take() + 1);
    println!("{}", a.take());
    // Arc 的 deref 拿到不可变引用的
    // 但 RefCell.get_mut 必须要 &mut self
    // 可用 take 取 replace 修改
    // 最后 RefCell 不能传出线程 `RefCell<i32>` cannot be shared between threads safely
}

fn get_tid() -> ThreadId {
    thread::current().id()
}

fn test4_arc_mutex() {
    // Mutex 可以获取 &mut 不需要 RefCell
    let data = Arc::new(Mutex::new(40));
    let join_handles = Arc::new(Mutex::new(Vec::<JoinHandle<()>>::new()));
    
    for _ in 0..3 {
        let join_handle = {
            let data = data.clone();
            let join_handles = join_handles.clone();

            thread::spawn(move || {
                let tid = get_tid();
                let mut number = data.lock().unwrap();

                println!("thread {:?} data={}", tid, number);
                *number += 1;
                println!("thread {:?} data++={}", tid, number);

                for _ in 0..3 {
                    let data = data.clone();

                    let join_handle = thread::spawn(move || {
                        let tid = get_tid();
                        let mut number = data.lock().unwrap();

                        thread::sleep(Duration::new(0,500));
            
                        println!("thread {:?} data={}", tid, number);
                        *number += 1;
                        println!("thread {:?} data++={}", tid, number);
                    });
                    join_handles.lock().unwrap().push(join_handle);
                }
            })
        };
        join_handles.lock().unwrap().push(join_handle);
    }

    let mut thread_count = 0;
    while let Some(join_handle) = {
        //join_handles.lock().unwrap().pop() // 1. 死锁
        let x = join_handles.lock().unwrap().pop(); x // 2. 未死锁 1和2有什么区别？
        //let mut join_handles = join_handles.lock().unwrap(); join_handles.pop() // 3. 未死锁
    } {
        join_handle.join().unwrap();
        thread_count += 1;
    }

    let a: i32 = *data.lock().unwrap();
    println!("end a={} thread_count={}", a, thread_count);
}

extern crate crossbeam;

/// ```
/// test5_scoped_mutex()
/// ```
fn test5_scoped_mutex() {
    // 理论上 可以通过传递 &Mutext<T> 来共享可变数据 (因为 Mutex: Sync)
    // 1. thread::spawn 生命期为 'static 要保证 data 生命期 需要 全局 静态 懒加载
    // 2. thread::scoped 应该可以保证子线程不长于父线程 好像已经被干掉了? 用 crossbeam::scope
    let a = Mutex::new(50);

    crossbeam::scope(|s| {
        for _ in 0..3 {
            let th = s.spawn(|_| {
                let id = get_tid();
                {
                    let mut number = a.lock().unwrap();
                    println!("thread {:?} a={}", id, number);
                    *number += 1;
                    println!("thread {:?} a++={}", id, number);
                }

                crossbeam::scope(|s| {
                    for _ in 0..3 {
                        let th = s.spawn(|_| {
                            let id = get_tid();
                            let mut number = a.lock().unwrap();

                            thread::sleep(Duration::new(0,2000));
                
                            println!("thread {:?} a={}", id, number);
                            *number += 1;
                            println!("thread {:?} a++={}", id, number);
                        });
                    }
                }).unwrap();
            });
        }
    }).unwrap();
    
    let number: i32 = *a.lock().unwrap();
    println!("end a={}", number);
}

fn test6_lazy_static_mutex() {
    static DATA: SyncLazy<Mutex<i32>> = SyncLazy::new(|| {
        Mutex::new(60)
    });

    for _ in 0..3 {
        let th = thread::spawn(|| {
            let id = get_tid();
            let mut number = DATA.lock().unwrap();

            println!("thread {:?} a={}", id, number);
            *number += 1;
            println!("thread {:?} a++={}", id, number);

            for _ in 0..3 {
                let th = thread::spawn(|| {
                    let id = get_tid();
                    let mut number = DATA.lock().unwrap();

                    thread::sleep(Duration::new(0,5));
        
                    println!("thread {:?} a={}", id, number);
                    *number += 1;
                    println!("thread {:?} a++={}", id, number);
                }); 
            }
        });
    }

    thread::sleep(Duration::new(3, 0));

    let a: i32 = *DATA.lock().unwrap();
    println!("end a={}", a);
}

fn test7_message_channel() {
    // 结果消息
    let (tx1, rx1) = mpsc::channel::<i32>();

    // 启动消息
    let (tx2, rx2) = mpsc::channel::<i32>();
    let arx2 = Arc::new(Mutex::new(rx2));

    for num in 70..82 {
        tx2.send(num);
    }

    for _ in 0..3 {
        let arx2 = arx2.clone();
        let tx1 = tx1.clone();
        let th = thread::spawn(move || {
            let id = get_tid();
            if let Ok(mut number) = arx2.lock().unwrap().recv() {
                println!("thread {:?} a={}", id, number);
                number += 1;
                println!("thread {:?} a++={}", id, number);
                tx1.send(number);
            }

            for _ in 0..3 {
                let arx2 = arx2.clone();
                let tx1 = tx1.clone();
                let th = thread::spawn(move || {
                    let id = get_tid();
                    if let Ok(mut number) = arx2.lock().unwrap().recv() {
                        println!("thread {:?} a={}", id, number);
                        number += 1;
                        println!("thread {:?} a++={}", id, number);
                        tx1.send(number);
                    }
                }); 
            }
        });
    }

    drop(tx1);
    let mut recv_count = 0;
    while let Ok(msg) = rx1.recv() {
        println!("Got: {}", msg);
        recv_count += 1;
    }

    println!("end recv_count={}", recv_count);
}
