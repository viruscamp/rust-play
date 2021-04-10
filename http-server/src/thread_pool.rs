use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender, channel}}, thread};

pub struct ThreadPool {
    pool: Vec<Worker>,
    tx: Sender<JobMessage>,
    rx: Arc<Mutex<Receiver<JobMessage>>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut pool = Vec::with_capacity(size);
        let (tx, rx) = channel::<JobMessage>();
        let rx = Arc::new(Mutex::new(rx));

        for i in 0..size {
            pool.push(Worker::new(i,  rx.clone()));
        }

        return ThreadPool {
            pool,
            tx,
            rx,
        };
    }
    pub fn execute_box(&mut self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.tx.send(JobMessage::Job(f)).unwrap_or_default();
    }
    pub fn execute<F: FnOnce() + Send + 'static>(&mut self, f: F) {
        self.execute_box(Box::new(f));
    }
    fn terminate(&mut self) {
        for _ in 0..self.pool.len() {
            self.tx.send(JobMessage::Terminate).unwrap_or_default();
        }
        println!("ThreadPool sent {} terminate messages", self.pool.len());

        println!("ThreadPool try drop jobs");
        // 此处仅用于 所有 worker 都没在等消息, 都在处理 job 时, 消耗所有的 job
        // 没有消息时, 有一个 worker 会一直持锁, 这边就会死锁, 所以要在前面发 terminate 消息
        let rx = self.rx.lock().expect("ThreadPool got rx lock error");
        self.tx.send(JobMessage::Terminate).unwrap_or_default();
        let mut dropped_count = 0;
        while let Ok(JobMessage::Job(_)) = rx.recv() { // 最后会消耗一个 teminate, 所以之前要补一个
            dropped_count += 1;
        }
        println!("ThreadPool dropped {} jobs", dropped_count);
        drop(rx); // rx unlock here

        for worker in &mut self.pool {
            worker.join();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.terminate();
    }
}

enum JobMessage {
    Job(Box<dyn FnOnce() + Send + 'static>),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<JobMessage>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // 没有消息时会一直持锁
                let msg = rx.lock().unwrap().recv();
                match msg {
                    Ok(JobMessage::Job(job)) => {
                        println!("Worker {} got job", id);
                        job();
                    }
                    Ok(JobMessage::Terminate) => {
                        println!("Worker {} terminating", id);
                        break;
                    }
                    Err(_) => {
                        println!("Worker {} terminating unexpected", id);
                        break;
                    }
                }
            }
        });
        return Worker {
            id,
            thread: Some(thread),
        };
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take() {
            println!("Worker {} waiting for terminating", self.id);
            thread.join().unwrap_or_default();
            println!("Worker {} terminated", self.id);
        }
    }
}
