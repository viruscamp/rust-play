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
        self.tx.send(JobMessage::Job(f));
    }
    pub fn execute<F: FnOnce() + Send + 'static>(&mut self, f: F) {
        self.execute_box(Box::new(f));
    }
    fn terminate(&mut self) {
        println!("ThreadPool try drop jobs");
        let rx = self.rx.lock().unwrap();
        self.tx.send(JobMessage::Terminate);
        let mut dropped_count = 0;
        while let Ok(JobMessage::Job(_)) = rx.recv() {
            dropped_count += 1;
        }
        println!("ThreadPool dropped {} jobs", dropped_count);
        drop(rx); // rx unlock here

        for _ in 0..self.pool.len() {
            self.tx.send(JobMessage::Terminate);
        }
        println!("ThreadPool sent {} terminate messages", self.pool.len());

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
            thread.join();
            println!("Worker {} terminated", self.id);
        }
    }
}
