use std::sync::{Arc, Mutex, Condvar, mpsc};
use std::thread;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::ast::*;
use crate::error::CompilerError;

#[derive(Debug)]
pub struct ThreadHandle {
    pub id: usize,
    pub join_handle: Option<thread::JoinHandle<()>>,
    pub result: Arc<Mutex<Option<Value>>>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

#[derive(Debug)]
pub struct Channel<T> {
    pub sender: mpsc::Sender<T>,
    pub receiver: mpsc::Receiver<T>,
    pub channel_type: ChannelType,
}

#[derive(Debug)]
pub struct ThreadManager {
    threads: HashMap<usize, ThreadHandle>,
    next_thread_id: AtomicUsize,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            next_thread_id: AtomicUsize::new(1),
        }
    }

    pub fn spawn<F>(&mut self, function: F) -> Result<usize, CompilerError>
    where
        F: FnOnce() -> Value + Send + 'static,
    {
        let thread_id = self.next_thread_id.fetch_add(1, Ordering::SeqCst);
        let result = Arc::new(Mutex::new(None));
        let result_clone = result.clone();

        let join_handle = thread::spawn(move || {
            let value = function();
            if let Ok(mut guard) = result_clone.lock() {
                *guard = Some(value);
            }
        });

        let thread_handle = ThreadHandle {
            id: thread_id,
            join_handle: Some(join_handle),
            result,
        };

        self.threads.insert(thread_id, thread_handle);
        Ok(thread_id)
    }

    pub fn join(&mut self, thread_id: usize) -> Result<Value, CompilerError> {
        if let Some(thread_handle) = self.threads.remove(&thread_id) {
            if let Some(join_handle) = thread_handle.join_handle {
                join_handle.join().map_err(|_| {
                    CompilerError::runtime_error("Thread join failed")
                })?;
            }

            if let Ok(guard) = thread_handle.result.lock() {
                if let Some(value) = guard.clone() {
                    return Ok(value);
                }
            }
        }

        Err(CompilerError::runtime_error(&format!("Thread {} not found", thread_id)))
    }

    pub fn is_finished(&self, thread_id: usize) -> bool {
        if let Some(thread_handle) = self.threads.get(&thread_id) {
            if let Some(join_handle) = &thread_handle.join_handle {
                join_handle.is_finished()
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl<T> Channel<T> {
    pub fn new(channel_type: ChannelType) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            channel_type,
        }
    }

    pub fn send(&self, value: T) -> Result<(), CompilerError> {
        self.sender.send(value).map_err(|_| {
            CompilerError::runtime_error("Channel send failed")
        })
    }

    pub fn recv(&self) -> Result<T, CompilerError> {
        self.receiver.recv().map_err(|_| {
            CompilerError::runtime_error("Channel receive failed")
        })
    }

    pub fn try_recv(&self) -> Result<T, CompilerError> {
        self.receiver.try_recv().map_err(|_| {
            CompilerError::runtime_error("Channel try_receive failed")
        })
    }
}

#[derive(Debug, Clone)]
pub struct MutexGuard<T> {
    pub data: Arc<Mutex<T>>,
}

impl<T> MutexGuard<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub fn lock(&self) -> Result<std::sync::MutexGuard<T>, CompilerError> {
        self.data.lock().map_err(|_| {
            CompilerError::runtime_error("Mutex lock failed")
        })
    }

    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<T>, CompilerError> {
        self.data.try_lock().map_err(|_| {
            CompilerError::runtime_error("Mutex try_lock failed")
        })
    }
}

#[derive(Debug, Clone)]
pub struct RwLock<T> {
    pub data: Arc<std::sync::RwLock<T>>,
}

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(std::sync::RwLock::new(data)),
        }
    }

    pub fn read(&self) -> Result<std::sync::RwLockReadGuard<T>, CompilerError> {
        self.data.read().map_err(|_| {
            CompilerError::runtime_error("RwLock read lock failed")
        })
    }

    pub fn write(&self) -> Result<std::sync::RwLockWriteGuard<T>, CompilerError> {
        self.data.write().map_err(|_| {
            CompilerError::runtime_error("RwLock write lock failed")
        })
    }
}

#[derive(Debug, Clone)]
pub struct Barrier {
    pub barrier: Arc<std::sync::Barrier>,
}

impl Barrier {
    pub fn new(count: usize) -> Self {
        Self {
            barrier: Arc::new(std::sync::Barrier::new(count)),
        }
    }

    pub fn wait(&self) -> Result<std::sync::BarrierWaitResult, CompilerError> {
        Ok(self.barrier.wait())
    }
}

#[derive(Debug, Clone)]
pub struct Semaphore {
    pub semaphore: Arc<tokio::sync::Semaphore>,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(permits)),
        }
    }

    pub async fn acquire(&self) -> Result<tokio::sync::SemaphorePermit, CompilerError> {
        self.semaphore.acquire().await.map_err(|_| {
            CompilerError::runtime_error("Semaphore acquire failed")
        })
    }

    pub fn try_acquire(&self) -> Result<tokio::sync::SemaphorePermit, CompilerError> {
        self.semaphore.try_acquire().map_err(|_| {
            CompilerError::runtime_error("Semaphore try_acquire failed")
        })
    }
}

#[derive(Debug, Clone)]
pub struct Atomic<T> {
    pub value: Arc<std::sync::atomic::AtomicUsize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Atomic<T> {
    pub fn new(value: usize) -> Self {
        Self {
            value: Arc::new(std::sync::atomic::AtomicUsize::new(value)),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn load(&self, ordering: Ordering) -> usize {
        self.value.load(ordering)
    }

    pub fn store(&self, value: usize, ordering: Ordering) {
        self.value.store(value, ordering);
    }

    pub fn fetch_add(&self, value: usize, ordering: Ordering) -> usize {
        self.value.fetch_add(value, ordering)
    }

    pub fn fetch_sub(&self, value: usize, ordering: Ordering) -> usize {
        self.value.fetch_sub(value, ordering)
    }

    pub fn compare_exchange(
        &self,
        current: usize,
        new: usize,
        success: Ordering,
        failure: Ordering,
    ) -> Result<usize, usize> {
        self.value.compare_exchange(current, new, success, failure)
    }
}

// Async/await support
#[derive(Debug, Clone)]
pub struct Future<T> {
    pub inner: Arc<Mutex<Option<T>>>,
    pub waker: Arc<Condvar>,
}

impl<T> Future<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            waker: Arc::new(Condvar::new()),
        }
    }

    pub fn set_result(&self, value: T) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(value);
        }
        self.waker.notify_all();
    }

    pub fn get_result(&self) -> Result<T, CompilerError> {
        let guard = self.inner.lock().map_err(|_| {
            CompilerError::runtime_error("Future lock failed")
        })?;

        if let Some(value) = guard.clone() {
            Ok(value)
        } else {
            Err(CompilerError::runtime_error("Future not ready"))
        }
    }

    pub fn wait(&self) -> Result<T, CompilerError> {
        let mut guard = self.inner.lock().map_err(|_| {
            CompilerError::runtime_error("Future lock failed")
        })?;

        while guard.is_none() {
            guard = self.waker.wait(guard).map_err(|_| {
                CompilerError::runtime_error("Future wait failed")
            })?;
        }

        guard.take().ok_or_else(|| {
            CompilerError::runtime_error("Future result not available")
        })
    }
}

// Task scheduler for async operations
#[derive(Clone)]
pub struct TaskScheduler {
    pub tasks: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
    pub worker_threads: Arc<AtomicUsize>,
    pub max_workers: usize,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            worker_threads: Arc::new(AtomicUsize::new(0)),
            max_workers: 4, // Default to 4 workers
        }
    }

    pub fn spawn_task<F>(&self, task: F) -> Result<(), CompilerError>
    where
        F: FnOnce() + Send + 'static,
    {
        if let Ok(mut tasks) = self.tasks.lock() {
            tasks.push(Box::new(task));
        }

        // Spawn a worker thread if needed
        self.spawn_worker();
        Ok(())
    }

    fn spawn_worker(&self) {
        let tasks = self.tasks.clone();
        let worker_threads = self.worker_threads.clone();
        let max_workers = self.max_workers;

        let handle = thread::spawn(move || {
            loop {
                let task = {
                    if let Ok(mut tasks) = tasks.lock() {
                        tasks.pop()
                    } else {
                        None
                    }
                };

                if let Some(task) = task {
                    task();
                } else {
                    thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });

        if let Ok(mut current_workers) = worker_threads.lock() {
            *current_workers += 1;
            if *current_workers < max_workers {
                self.spawn_worker();
            }
        };
    }
}

// Memory ordering constants
pub use std::sync::atomic::Ordering;

// Concurrency utilities
pub fn yield_now() {
    thread::yield_now();
}

pub fn sleep(duration_ms: u64) {
    thread::sleep(std::time::Duration::from_millis(duration_ms));
}

pub fn current_thread_id() -> usize {
    // Use a simpler approach since as_u64() is unstable
    std::thread_local! {
        static THREAD_ID: std::cell::RefCell<Option<usize>> = std::cell::RefCell::new(None);
    }
    
    THREAD_ID.with(|id| {
        if let Some(id) = *id.borrow() {
            id
        } else {
            let new_id = std::sync::atomic::AtomicUsize::new(0).fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            *id.borrow_mut() = Some(new_id);
            new_id
        }
    })
}

// Thread-local storage
#[derive(Debug)]
pub struct ThreadLocal<T> {
    pub data: Arc<Mutex<HashMap<usize, T>>>,
}

impl<T: Clone> ThreadLocal<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_value<F, R>(&self, value: T, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let thread_id = self.current_thread_id();
        let mut data = self.data.lock().unwrap();
        
        data.insert(thread_id, value);
        let result = f(data.get(&thread_id).unwrap());
        
        result
    }

    pub fn get(&self) -> Option<T> {
        let thread_id = self.current_thread_id();
        let data = self.data.lock().unwrap();
        data.get(&thread_id).cloned()
    }

    pub fn set(&self, value: T) {
        let thread_id = self.current_thread_id();
        let mut data = self.data.lock().unwrap();
        data.insert(thread_id, value);
    }

    pub fn remove(&self) {
        let thread_id = self.current_thread_id();
        let mut data = self.data.lock().unwrap();
        data.remove(&thread_id);
    }

    fn current_thread_id(&self) -> usize {
        std::thread::current().id().as_u64().get() as usize
    }
}

// Concurrency primitives for the language
pub struct ConcurrencyPrimitives {
    pub thread_manager: ThreadManager,
    pub task_scheduler: TaskScheduler,
}

impl ConcurrencyPrimitives {
    pub fn new() -> Self {
        Self {
            thread_manager: ThreadManager::new(),
            task_scheduler: TaskScheduler::new(),
        }
    }

    pub fn spawn_thread<F>(&mut self, function: F) -> Result<usize, CompilerError>
    where
        F: FnOnce() -> Value + Send + 'static,
    {
        self.thread_manager.spawn(function)
    }

    pub fn join_thread(&mut self, thread_id: usize) -> Result<Value, CompilerError> {
        self.thread_manager.join(thread_id)
    }

    pub fn create_channel<T>(channel_type: ChannelType) -> Channel<T> {
        Channel::new(channel_type)
    }

    pub fn create_mutex<T>(data: T) -> MutexGuard<T> {
        MutexGuard::new(data)
    }

    pub fn create_rwlock<T>(data: T) -> RwLock<T> {
        RwLock::new(data)
    }

    pub fn create_barrier(count: usize) -> Barrier {
        Barrier::new(count)
    }

    pub fn create_semaphore(permits: usize) -> Semaphore {
        Semaphore::new(permits)
    }

    pub fn create_atomic<T>(value: usize) -> Atomic<T> {
        Atomic::new(value)
    }

    pub fn create_future<T>() -> Future<T> {
        Future::new()
    }

    pub fn create_thread_local<T>() -> ThreadLocal<T> {
        ThreadLocal::new()
    }
} 