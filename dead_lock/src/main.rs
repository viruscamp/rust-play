use std::sync::Mutex;

fn main() {
    dead_lock_2();
}

fn no_lock() {
    let mut vec = vec![1,2,3];
    while let Some(num) = vec.pop() {
        if num == 2 {
            vec.push(4);
        }
        println!("got {}", num);
    }
}

fn dead_lock_1() {
    let vec = Mutex::new(vec![1,2,3]);
    // 会导致 lock (MutexGuard 的临时变量) 有效期包括整个 while 导致死锁
    while let Some(num) = vec.lock().unwrap().pop() {
        if num == 2 {
            vec.lock().unwrap().push(4);
        }
        println!("got {}", num);
    }
}

// https://rustcc.cn/article?id=3f446fab-1f4b-4d3f-9240-95b673bf5062
// https://doc.rust-lang.org/stable/reference/expressions.html#temporaries
// 居 7sDream 研究： 临时变量的 drop 时机，在所在的 statement 结束后
// 而 { expr } 这种形式是 block expression 并不是一个 statement
// while let Some(num) = { vec.lock().unwrap().pop() } 找到的 statement 就是 while

fn dead_lock_2() {
    let vec = Mutex::new(vec![1,2,3]);
    // 这也会 为什么?
    while let Some(num) = {
        vec.lock().unwrap().pop()
    } {
        if num == 2 {
            vec.lock().unwrap().push(4);
        }
        println!("got {}", num);
    }
}

fn dead_lock_3() {
    let vec = Mutex::new(vec![1,2,3]);
    while let Some(num) = {
        let x = vec.lock();
        x.unwrap().pop()
    } {
        if num == 2 {
            vec.lock().unwrap().push(4);
        }
        println!("got {}", num);
    }
}

fn no_dead_lock_1() {
    let vec = Mutex::new(vec![1,2,3]);
    // 终于不会死锁了，但跟 dead_lock_2 有什么区别
    while let Some(num) = {
        let mut vec = vec.lock().unwrap();
        vec.pop()
    } {
        if num == 2 {
            vec.lock().unwrap().push(4);
        }
        println!("got {}", num);
    }
}

fn no_dead_lock_2() {
    let vec = Mutex::new(vec![1,2,3]);
    // 这个跟 dead_lock_2 有什么区别
    while let Some(num) = {
        let num = vec.lock().unwrap().pop();
        num
    } {
        if num == 2 {
            vec.lock().unwrap().push(4);
        }
        println!("got {}", num);
    }
}