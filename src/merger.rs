use std::sync::{Arc, Mutex};
use crate::image::Image;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::{thread, time};
use indicatif::ProgressBar;

#[derive(Copy, Clone)]
pub enum MergeAction {
    Add,
    Max
}

pub struct Merger {
    mode: MergeAction,
    done: Arc<AtomicBool>,
    queue: Arc<Mutex<Vec<Image>>>,
    threads: Vec<JoinHandle<()>>,
    progress_bar: Arc<Mutex<ProgressBar>>
}


impl Merger {
    pub fn new(action: MergeAction, progress_bar: ProgressBar) -> Self {
        Self {
            mode: action,
            done: Arc::new(AtomicBool::new(false)),
            queue: Arc::new(Mutex::new(vec![])),
            threads: vec![],
            progress_bar: Arc::new(Mutex::new(progress_bar))
        }
    }

    pub fn get_queue(&self) -> Arc<Mutex<Vec<Image>>> {
        Arc::clone(&self.queue)
    }

    pub fn spawn_workers(&mut self, num_threads: usize) {
        for _ in 0..num_threads {
            let q = Arc::clone(&self.queue);
            let d = Arc::clone(&self.done);
            let m = MergeAction::clone(&self.mode);
            let pb = Arc::clone(&self.progress_bar);

            self.threads.push(thread::spawn(move || {
                queue_worker(q, d, m, pb);
            }));
        }
    }

    pub fn finish(&self) {
        self.done.store(true, Ordering::Relaxed);
    }

    pub fn finish_and_join(self) -> Option<Image> {
        self.finish();

        for t in self.threads {
            t.join().unwrap_or(());
        }

        self.progress_bar.lock().unwrap().finish();
        self.queue.lock().unwrap().pop()
    }
}


fn queue_worker(queue: Arc<Mutex<Vec<Image>>>, done: Arc<AtomicBool>, mode: MergeAction, pb: Arc<Mutex<ProgressBar>>) {
    loop {
        let mut q = queue.lock().unwrap();
        if q.len() <= 1 {
            if done.load(Ordering::Relaxed) { return; } else {
                // Queue is empty but work is not done yet => Wait.
                drop(q);
                thread::sleep(time::Duration::from_millis(20));
                continue;
            }
        }

        let v1 = q.pop().unwrap();
        let v2 = q.pop().unwrap();
        drop(q);

        let res = match mode {
            MergeAction::Add => v1.merge_add(v2),
            MergeAction::Max => v1.merge_max(v2)
        };

        queue.lock().unwrap().push(res);
        pb.lock().unwrap().inc(1);
    }
}
