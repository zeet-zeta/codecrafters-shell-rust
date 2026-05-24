use std::{
    collections::HashMap,
    fmt,
    sync::{LazyLock, Mutex, RwLock},
};

pub struct CompletionSpec {
    pub completer_path: String,
}

pub struct IDAllocator {
    pub current_id: usize,
    pub recycle_stack: Vec<usize>,
}

impl IDAllocator {
    pub fn new() -> Self {
        Self {
            current_id: 1,
            recycle_stack: Vec::new(),
        }
    }

    pub fn alloc_id(&mut self) -> usize {
        if let Some(id) = self.recycle_stack.pop() {
            return id;
        }
        self.current_id += 1;
        self.current_id - 1
    }

    pub fn dealloc_id(&mut self, id: usize) {
        assert!(id < self.current_id);
        self.recycle_stack.push(id);
    }
}

pub static COMPLETION_REGISTRY: LazyLock<RwLock<HashMap<String, CompletionSpec>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub static ID_ALLOCATOR: LazyLock<Mutex<IDAllocator>> =
    LazyLock::new(|| Mutex::new(IDAllocator::new()));

pub static JOB_TABLE: LazyLock<Mutex<JobTable>> = LazyLock::new(|| Mutex::new(JobTable::new()));

pub fn register_completion(cmd: String, path: String) {
    if let Ok(mut guard) = COMPLETION_REGISTRY.write() {
        guard.insert(
            cmd,
            CompletionSpec {
                completer_path: path,
            },
        );
    }
}

pub fn unregister_completion(cmd: &str) {
    if let Ok(mut guard) = COMPLETION_REGISTRY.write() {
        guard.remove(cmd);
    }
}

pub fn find_completion(cmd: &str) -> Option<String> {
    if let Ok(guard) = COMPLETION_REGISTRY.read() {
        guard.get(cmd).map(|spec| spec.completer_path.clone())
    } else {
        None
    }
}

pub fn alloc_id() -> usize {
    let mut data = ID_ALLOCATOR.lock().unwrap();
    data.alloc_id()
}

pub fn dealloc_id(id: usize) {
    let mut data = ID_ALLOCATOR.lock().unwrap();
    data.dealloc_id(id);
}

#[derive(PartialEq)]
pub enum JobState {
    Running,
    Done,
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match self {
            JobState::Done => "Done",
            JobState::Running => "Running",
        };
        write!(f, "{:<24}", state_str)
    }
}

pub struct JobDetail {
    pub id: usize,
    pub handler: std::process::Child,
    pub command_string: String,
    pub state: JobState,
}

pub struct JobTable {
    pub current: usize,
    pub previous: usize,
    pub jobs: Vec<JobDetail>,
}

impl JobDetail {
    pub fn new(id: usize, handler: std::process::Child, command_string: String) -> Self {
        Self {
            id,
            handler,
            command_string,
            state: JobState::Running,
        }
    }
}

impl JobTable {
    pub fn new() -> Self {
        Self {
            current: 0,
            previous: 0,
            jobs: Vec::new(),
        }
    }

    pub fn add(&mut self, detail: JobDetail) {
        if self.current != 0 {
            self.previous = self.current;
        }
        self.current = detail.id;
        self.jobs.push(detail);
    }

    pub fn print(&self) {
        for job in &self.jobs {
            let marker = if job.id == self.current {
                "+"
            } else if job.id == self.previous {
                "-"
            } else {
                " "
            };
            println!(
                "[{}]{}  {}{}",
                job.id, marker, job.state, job.command_string
            );
        }
    }

    pub fn reap(&mut self) {
        for job in &mut self.jobs {
            if job.state == JobState::Running {
                if let Ok(Some(_)) = job.handler.try_wait() {
                    job.state = JobState::Done;
                    job.command_string.truncate(job.command_string.len() - 2);
                }
            }
        }
    }

    pub fn remove_done(&mut self) {
        self.jobs.retain(|x| x.state == JobState::Running);
    }
}

pub fn with_global_jobs<F, R>(f: F) -> R
where
    F: FnOnce(&mut JobTable) -> R,
{
    let mut guard = JOB_TABLE.lock().unwrap();
    f(&mut *guard)
}
