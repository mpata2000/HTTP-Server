use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Option<thread::JoinHandle<()>>>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    /// The size is the number of threads in the pool with a minimum of 1.
    pub fn new(size: usize) -> ThreadPool {
        // make sure size is at least 1
        let size = size.max(1);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let receiver: Arc<Mutex<mpsc::Receiver<Job>>> = Arc::clone(&receiver);

            let worker = thread::spawn(move || loop {
                let message = match receiver.lock() {
                    Ok(receiver) => receiver.recv(),
                    Err(_) => {
                        // Mutex was poisoned, so we should exit the thread
                        break;
                    }
                };

                match message {
                    Ok(job) => {
                        job();
                    }
                    Err(_) => {
                        // Sender was dropped, so we should exit the thread
                        break;
                    }
                }
            });

            workers.push(Some(worker));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .as_ref()
            .unwrap()
            .send(Box::new(f))
            .expect("Error sending job")
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for thread in &mut self.workers {
            println!("Shutting down worker");
            if let Some(thread) = thread.take() {
                thread.join().expect("Error joining worker thread");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);

        for i in 0..20 {
            pool.execute(move || {
                println!("Job {} started", i);
                thread::sleep(std::time::Duration::from_secs(1));
                println!("Job {} finished", i);
            });
        }

        thread::sleep(std::time::Duration::from_secs(5));
    }
}
