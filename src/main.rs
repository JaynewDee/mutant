#[allow(dead_code)]

fn main() {
    //? Experimental ?//

    //
    //
    //

    pub mod mutant {
        use crossbeam::channel;
        use std::{collections::BTreeMap, thread};

        pub struct Student {
            id: u32,
            name: String,
            pub avg: f32,
            group: u16,
            email: String,
        }

        type Job = Box<
            dyn FnOnce(Vec<Student>, u16) -> (f32, BTreeMap<u16, Vec<Student>>) + Send + 'static,
        >;
        type Args = (Vec<Student>, u16);
        enum Message {
            NewJob(Job, Args),
            Terminate,
        }

        pub struct ThreadPool {
            workers: Vec<Worker>,
            sender: channel::Sender<Message>,
        }

        impl ThreadPool {
            pub fn new(size: usize) -> ThreadPool {
                assert!(size > 0);

                let (sender, receiver) = channel::unbounded();

                let mut workers = Vec::with_capacity(size);

                for id in 0..size {
                    workers.push(Worker::new(id, receiver.clone()));
                }

                ThreadPool { workers, sender }
            }

            pub fn execute<F, A>(&self, f: F, args: Args)
            where
                F: FnOnce(Vec<Student>, u16) -> (f32, BTreeMap<u16, Vec<Student>>) + Send + 'static,
            {
                let job = Box::new(f);
                self.sender.send(Message::NewJob(job, args)).unwrap();
            }
        }

        impl Drop for ThreadPool {
            fn drop(&mut self) {
                for _ in &self.workers {
                    self.sender.send(Message::Terminate).unwrap();
                }

                for worker in &mut self.workers {
                    if let Some(thread) = worker.thread.take() {
                        thread.join().unwrap();
                    }
                }
            }
        }

        struct Worker {
            id: usize,
            thread: Option<thread::JoinHandle<()>>,
        }

        impl Worker {
            fn new(id: usize, receiver: channel::Receiver<Message>) -> Worker {
                let thread = thread::spawn(move || loop {
                    match receiver.recv().unwrap() {
                        Message::NewJob(job, args) => {
                            job(args.0, args.1);
                        }
                        Message::Terminate => {
                            break;
                        }
                    }
                });

                Worker {
                    id,
                    thread: Some(thread),
                }
            }
        }
    }
}
