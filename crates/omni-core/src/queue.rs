use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

use crate::engine::convert_file;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Done,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub input: String,
    pub output: String,
    pub state: JobState,
    pub progress: f32,
}

type Listener = tokio::sync::mpsc::UnboundedSender<Job>;

/// Parallel conversion queue with progress, cancel, retry.
#[derive(Debug, Default)]
pub struct JobQueue {
    inner: Arc<Mutex<HashMap<String, Job>>>,
    listeners: Arc<Mutex<Vec<Listener>>>,
    cancelled: Arc<Mutex<HashMap<String, bool>>>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> tokio::sync::mpsc::UnboundedReceiver<Job> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        // best-effort: blocking not allowed here; spawn
        let listeners = self.listeners.clone();
        tokio::spawn(async move {
            listeners.lock().await.push(tx);
        });
        rx
    }

    async fn emit(&self, job: &Job) {
        let mut dead = vec![];
        let mut ls = self.listeners.lock().await;
        for (i, l) in ls.iter().enumerate() {
            if l.send(job.clone()).is_err() {
                dead.push(i);
            }
        }
        for i in dead.into_iter().rev() {
            ls.remove(i);
        }
    }

    pub async fn submit(&self, input: String, output: String) -> String {
        let id = format!("job-{}", self.inner.lock().await.len() + 1);
        let job = Job { id: id.clone(), input, output, state: JobState::Queued, progress: 0.0 };
        self.inner.lock().await.insert(id.clone(), job.clone());
        self.emit(&job).await;
        id
    }

    pub async fn cancel(&self, id: &str) {
        self.cancelled.lock().await.insert(id.into(), true);
        if let Some(j) = self.inner.lock().await.get_mut(id) {
            j.state = JobState::Cancelled;
            let c = j.clone();
            self.emit(&c).await;
        }
    }

    pub async fn list(&self) -> Vec<Job> {
        self.inner.lock().await.values().cloned().collect()
    }

    /// Run all queued jobs with bounded parallelism.
    pub async fn run_all(&self, parallel: usize) {
        let sem = Arc::new(Semaphore::new(parallel.max(1)));
        let ids: Vec<String> = self.inner.lock().await.keys().cloned().collect();
        let mut handles = vec![];
        for id in ids {
            let sem = sem.clone();
            let this = self.clone_ref();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                this.run_one(&id).await;
            }));
        }
        for h in handles {
            let _ = h.await;
        }
    }

    fn clone_ref(&self) -> QueueRef {
        QueueRef {
            inner: self.inner.clone(),
            listeners: self.listeners.clone(),
            cancelled: self.cancelled.clone(),
        }
    }
}

struct QueueRef {
    inner: Arc<Mutex<HashMap<String, Job>>>,
    listeners: Arc<Mutex<Vec<Listener>>>,
    cancelled: Arc<Mutex<HashMap<String, bool>>>,
}

impl QueueRef {
    async fn emit(&self, job: &Job) {
        let mut ls = self.listeners.lock().await;
        ls.retain(|l| l.send(job.clone()).is_ok());
    }
    async fn run_one(&self, id: &str) {
        let (input, output) = {
            let mut map = self.inner.lock().await;
            let Some(j) = map.get_mut(id) else { return };
            j.state = JobState::Running;
            j.progress = 0.1;
            (j.input.clone(), j.output.clone())
        };
        if let Some(j) = self.inner.lock().await.get(id).cloned() {
            self.emit(&j).await;
        }
        if *self.cancelled.lock().await.get(id).unwrap_or(&false) {
            return;
        }
        let res = convert_file(std::path::Path::new(&input), std::path::Path::new(&output)).await;
        let mut map = self.inner.lock().await;
        if let Some(j) = map.get_mut(id) {
            match res {
                Ok(()) => {
                    j.state = JobState::Done;
                    j.progress = 1.0;
                }
                Err(e) => {
                    j.state = JobState::Failed(e.to_string());
                }
            }
            let c = j.clone();
            drop(map);
            self.emit(&c).await;
        }
    }
}
