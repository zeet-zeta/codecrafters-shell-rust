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

pub struct History {
    pub lines: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn push(&mut self, s: &str) {
        self.lines.push(s.to_string())
    }

    pub fn print(&self, n: Option<usize>) {
        let total_len = self.lines.len();
        let count = match n {
            Some(val) => val.min(total_len),
            None => total_len,
        };
        let skip_count = total_len - count;

        for (i, command) in self.lines.iter().enumerate().skip(skip_count) {
            let line_number = i + 1;
            println!("{:>5} {}", line_number, command);
        }
    }
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

pub static HISTORY: LazyLock<Mutex<History>> = LazyLock::new(|| Mutex::new(History::new()));

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

    pub fn to_string(&self, marker: &str) -> String {
        format!(
            "[{}]{}  {}{}",
            self.id, marker, self.state, self.command_string
        )
    }

    pub fn reap(&mut self) -> bool {
        if self.state == JobState::Running {
            if let Ok(Some(_)) = self.handler.try_wait() {
                self.state = JobState::Done;
                self.command_string.truncate(self.command_string.len() - 2);
                dealloc_id(self.id);
                return true;
            }
        }
        false
    }
}

impl JobTable {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn add(&mut self, detail: JobDetail) {
        self.jobs.push(detail);
    }

    pub fn reap_and_print(&mut self) {
        let (plus_id, minus_id) = self.get_plus_and_minus_id();
        for job in &mut self.jobs {
            job.reap();
            let marker = Self::get_marker(job.id, plus_id, minus_id);
            println!("{}", job.to_string(marker));
        }
        self.jobs.retain(|x| x.state == JobState::Running);
    }

    pub fn reap(&mut self) -> Vec<String> {
        let mut result = Vec::new();
        let (plus_id, minus_id) = self.get_plus_and_minus_id();
        for job in &mut self.jobs {
            if job.reap() {
                let marker = Self::get_marker(job.id, plus_id, minus_id);
                result.push(job.to_string(marker));
            }
        }
        self.jobs.retain(|x| x.state == JobState::Running);
        result
    }

    fn get_plus_and_minus_id(&self) -> (Option<usize>, Option<usize>) {
        let mut rev_iter = self.jobs.iter().rev();
        let plus_id = rev_iter.next().map(|x| x.id);
        let minus_id = rev_iter.next().map(|x| x.id);
        (plus_id, minus_id)
    }

    fn get_marker(id: usize, plus_id: Option<usize>, minus_id: Option<usize>) -> &'static str {
        let marker = if Some(id) == plus_id {
            "+"
        } else if Some(id) == minus_id {
            "-"
        } else {
            " "
        };
        marker
    }
}

pub fn with_global_jobs<F, R>(f: F) -> R
where
    F: FnOnce(&mut JobTable) -> R,
{
    let mut guard = JOB_TABLE.lock().unwrap();
    f(&mut *guard)
}

pub fn with_global_history<F, R>(f: F) -> R
where
    F: FnOnce(&mut History) -> R,
{
    let mut guard = HISTORY.lock().unwrap();
    f(&mut *guard)
}
