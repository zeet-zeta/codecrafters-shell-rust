use std::{
    collections::HashMap,
    fmt,
    sync::{LazyLock, Mutex, RwLock},
};

use crate::state;

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
    pub pid: u32,
    pub command_string: String,
    pub state: JobState,
}

pub struct JobTable {
    pub recent: usize,
    pub jobs: Vec<JobDetail>,
}

impl JobDetail {
    pub fn new(id: usize, pid: u32, command_string: String) -> Self {
        Self {
            id,
            pid,
            command_string,
            state: JobState::Running,
        }
    }
}

impl JobTable {
    pub fn new() -> Self {
        Self {
            recent: 0,
            jobs: Vec::new(),
        }
    }

    pub fn add(&mut self, detail: JobDetail) {
        self.recent = detail.id;
        self.jobs.push(detail);
    }

    pub fn print(&self) {
        for job in &self.jobs {
            let marker = if job.id == self.recent { "+" } else { " " };
            println!(
                "[{}]{}  {}{}",
                job.id, marker, job.state, job.command_string
            );
        }
    }
}

pub fn add_to_job_table(detail: JobDetail) {
    let mut data = JOB_TABLE.lock().unwrap();
    data.add(detail);
}

pub fn print_job_table() {
    let data = JOB_TABLE.lock().unwrap();
    data.print();
}
