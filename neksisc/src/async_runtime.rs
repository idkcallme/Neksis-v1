use std::collections::VecDeque;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use crate::vm::VMValue;

pub type TaskId = usize;

#[derive(Debug)]
pub enum AsyncMessage {
    SpawnTask(Task),
    CompleteTask(TaskId, VMValue),
    FailTask(TaskId, String),
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub bytecode: Vec<crate::vm::BytecodeInstruction>,
    pub priority: TaskPriority,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

pub struct AsyncRuntime {
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    completed_tasks: Arc<Mutex<std::collections::HashMap<TaskId, VMValue>>>,
    failed_tasks: Arc<Mutex<std::collections::HashMap<TaskId, String>>>,
    worker_handles: Vec<thread::JoinHandle<()>>,
    #[allow(dead_code)]
    message_sender: mpsc::Sender<AsyncMessage>,
    #[allow(dead_code)]
    message_receiver: Arc<Mutex<mpsc::Receiver<AsyncMessage>>>,
    next_task_id: Arc<Mutex<TaskId>>,
    is_running: Arc<Mutex<bool>>,
    worker_count: usize,
}

impl AsyncRuntime {
    pub fn new(worker_count: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            completed_tasks: Arc::new(Mutex::new(std::collections::HashMap::new())),
            failed_tasks: Arc::new(Mutex::new(std::collections::HashMap::new())),
            worker_handles: Vec::new(),
            message_sender: sender,
            message_receiver: Arc::new(Mutex::new(receiver)),
            next_task_id: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
            worker_count,
        }
    }
    
    pub fn start(&mut self) {
        *self.is_running.lock().unwrap() = true;
        
        // Start worker threads
        for worker_id in 0..self.worker_count {
            let task_queue = Arc::clone(&self.task_queue);
            let completed_tasks = Arc::clone(&self.completed_tasks);
            let failed_tasks = Arc::clone(&self.failed_tasks);
            let is_running = Arc::clone(&self.is_running);
            
            let handle = thread::spawn(move || {
                Self::worker_loop(worker_id, task_queue, completed_tasks, failed_tasks, is_running);
            });
            
            self.worker_handles.push(handle);
        }
        
        println!("AsyncRuntime: Started {} worker threads", self.worker_count);
    }
    
    fn worker_loop(
        worker_id: usize,
        task_queue: Arc<Mutex<VecDeque<Task>>>,
        completed_tasks: Arc<Mutex<std::collections::HashMap<TaskId, VMValue>>>,
        failed_tasks: Arc<Mutex<std::collections::HashMap<TaskId, String>>>,
        is_running: Arc<Mutex<bool>>,
    ) {
        println!("Worker {} started", worker_id);
        
        while *is_running.lock().unwrap() {
            let task = {
                let mut queue = task_queue.lock().unwrap();
                queue.pop_front()
            };
            
            if let Some(task) = task {
                println!("Worker {} executing task {} ({})", worker_id, task.id, task.name);
                
                // Execute the task (simplified for now)
                let result = Self::execute_task(&task);
                
                match result {
                    Ok(value) => {
                        completed_tasks.lock().unwrap().insert(task.id, value);
                        println!("Worker {} completed task {}", worker_id, task.id);
                    }
                    Err(error) => {
                        failed_tasks.lock().unwrap().insert(task.id, error);
                        println!("Worker {} failed task {}: error", worker_id, task.id);
                    }
                }
            } else {
                // No tasks available, sleep briefly
                thread::sleep(Duration::from_millis(10));
            }
        }
        
        println!("Worker {} shutting down", worker_id);
    }
    
    fn execute_task(task: &Task) -> Result<VMValue, String> {
        // For now, simulate different types of async work
        match task.name.as_str() {
            "fibonacci" => {
                // CPU-intensive task
                thread::sleep(Duration::from_millis(50)); // Simulate work
                Ok(VMValue::Int(Self::fibonacci(20)))
            }
            "io_operation" => {
                // I/O intensive task
                thread::sleep(Duration::from_millis(100)); // Simulate I/O
                Ok(VMValue::String("I/O completed".to_string()))
            }
            "network_request" => {
                // Network operation
                thread::sleep(Duration::from_millis(200)); // Simulate network delay
                Ok(VMValue::String("Network response".to_string()))
            }
            _ => {
                // Generic async task
                thread::sleep(Duration::from_millis(25));
                Ok(VMValue::String(format!("Task {} completed", task.name)))
            }
        }
    }
    
    pub fn spawn_task(&mut self, name: String, bytecode: Vec<crate::vm::BytecodeInstruction>, priority: TaskPriority) -> TaskId {
        let task_id = {
            let mut next_id = self.next_task_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let priority_clone = priority.clone();
        
        let task = Task {
            id: task_id,
            name,
            bytecode,
            priority,
            created_at: Instant::now(),
        };
        
        // Insert task in priority order
        {
            let mut queue = self.task_queue.lock().unwrap();
            
            // Find correct position based on priority
            let insert_pos = queue
                .iter()
                .position(|existing_task| existing_task.priority < task.priority)
                .unwrap_or(queue.len());
            
            queue.insert(insert_pos, task);
        }
        
        println!("Spawned task {} with priority {:?}", task_id, priority_clone);
        task_id
    }
    
    pub fn await_task(&self, task_id: TaskId) -> Result<VMValue, String> {
        // Poll for task completion
        loop {
            // Check if task completed successfully
            if let Some(result) = self.completed_tasks.lock().unwrap().remove(&task_id) {
                return Ok(result);
            }
            
            // Check if task failed
            if let Some(error) = self.failed_tasks.lock().unwrap().remove(&task_id) {
                return Err(error);
            }
            
            // Task still running, sleep briefly
            thread::sleep(Duration::from_millis(5));
        }
    }
    
    pub fn try_get_result(&self, task_id: TaskId) -> Option<Result<VMValue, String>> {
        if let Some(result) = self.completed_tasks.lock().unwrap().remove(&task_id) {
            Some(Ok(result))
        } else if let Some(error) = self.failed_tasks.lock().unwrap().remove(&task_id) {
            Some(Err(error))
        } else {
            None
        }
    }
    
    pub fn shutdown(&mut self) {
        println!("AsyncRuntime: Shutting down...");
        
        *self.is_running.lock().unwrap() = false;
        
        // Wait for all workers to finish
        while let Some(handle) = self.worker_handles.pop() {
            handle.join().unwrap();
        }
        
        println!("AsyncRuntime: All workers shut down");
    }
    
    pub fn get_stats(&self) -> String {
        let queue_size = self.task_queue.lock().unwrap().len();
        let completed_count = self.completed_tasks.lock().unwrap().len();
        let failed_count = self.failed_tasks.lock().unwrap().len();
        
        format!(
            "Async Stats: {} queued, {} completed, {} failed, {} workers",
            queue_size, completed_count, failed_count, self.worker_count
        )
    }
    
    // Helper functions
    fn fibonacci(n: i64) -> i64 {
        if n <= 1 {
            return n;
        }
        
        let mut a = 0;
        let mut b = 1;
        
        for _ in 2..=n {
            let temp = a + b;
            a = b;
            b = temp;
        }
        
        b
    }
}

// Channel-based communication (Actor model)
pub struct Channel<T> {
    sender: mpsc::Sender<T>,
    receiver: Arc<Mutex<mpsc::Receiver<T>>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
    
    pub fn send(&self, message: T) -> Result<(), mpsc::SendError<T>> {
        self.sender.send(message)
    }
    
    pub fn try_receive(&self) -> Result<T, mpsc::TryRecvError> {
        self.receiver.lock().unwrap().try_recv()
    }
    
    pub fn receive(&self) -> Result<T, mpsc::RecvError> {
        self.receiver.lock().unwrap().recv()
    }
}

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
        }
    }
}
