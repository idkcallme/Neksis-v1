// Modern async runtime for Neksis 2025
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crate::modern_stdlib::{NeksisError, NeksisResult};

/// Simple Task that can be executed by the runtime
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Task {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            future: Box::pin(future),
        }
    }
}

/// Simple executor for async tasks
pub struct Executor {
    tasks: Arc<Mutex<VecDeque<Task>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
            waker: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Task::new(future);
        self.tasks.lock().unwrap().push_back(task);
        
        // Wake up the executor if it's sleeping
        if let Some(waker) = self.waker.lock().unwrap().as_ref() {
            waker.wake_by_ref();
        }
    }
    
    pub fn run(&self) {
        loop {
            let mut tasks = self.tasks.lock().unwrap();
            
            if tasks.is_empty() {
                break;
            }
            
            let mut task = tasks.pop_front().unwrap();
            drop(tasks);
            
            // Create a simple waker
            let waker = futures::task::noop_waker();
            let mut context = Context::from_waker(&waker);
            
            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {
                    // Task completed
                }
                Poll::Pending => {
                    // Task is not ready, put it back
                    self.tasks.lock().unwrap().push_back(task);
                }
            }
        }
    }
    
    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        // Simple block_on implementation using polling
        let mut future = Box::pin(future);
        let waker = futures::task::noop_waker();
        let mut context = Context::from_waker(&waker);
        
        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(result) => return result,
                Poll::Pending => thread::yield_now(),
            }
        }
    }
}

/// Global executor instance
static mut GLOBAL_EXECUTOR: Option<Executor> = None;
static INIT: std::sync::Once = std::sync::Once::new();

#[allow(static_mut_refs)]
fn get_global_executor() -> &'static Executor {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_EXECUTOR = Some(Executor::new());
        });
        GLOBAL_EXECUTOR.as_ref().unwrap()
    }
}

/// Spawn a task on the global executor
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    get_global_executor().spawn(future);
}

/// Block on a future using the global executor
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    get_global_executor().block_on(future)
}

/// Run the global executor
pub fn run() {
    get_global_executor().run();
}

/// Sleep for a duration (async)
pub async fn sleep(duration: Duration) {
    let start = Instant::now();
    loop {
        if start.elapsed() >= duration {
            break;
        }
        // Yield control
        yield_now().await;
    }
}

/// Yield control to other tasks
pub async fn yield_now() {
    struct YieldNow {
        yielded: bool,
    }
    
    impl Future for YieldNow {
        type Output = ();
        
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
    
    YieldNow { yielded: false }.await
}

/// Timeout wrapper for futures
pub async fn timeout<F, T>(duration: Duration, future: F) -> NeksisResult<T>
where
    F: Future<Output = T>,
{
    struct Timeout<F> {
        future: Pin<Box<F>>,
        deadline: Instant,
    }
    
    impl<F: Future> Future for Timeout<F> {
        type Output = NeksisResult<F::Output>;
        
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if Instant::now() >= self.deadline {
                return Poll::Ready(Err(NeksisError::Timeout("Operation timed out".to_string())));
            }
            
            match self.future.as_mut().poll(cx) {
                Poll::Ready(output) => Poll::Ready(Ok(output)),
                Poll::Pending => Poll::Pending,
            }
        }
    }
    
    Timeout {
        future: Box::pin(future),
        deadline: Instant::now() + duration,
    }.await
}

/// Simple channel for async communication
pub struct Channel<T> {
    sender: std::sync::mpsc::Sender<T>,
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<T>>>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
    
    pub fn send(&self, value: T) -> NeksisResult<()> {
        self.sender.send(value)
            .map_err(|_| NeksisError::ChannelError("Failed to send message".to_string()))
    }
    
    pub async fn receive(&self) -> NeksisResult<T> {
        // For now, use blocking receive in a loop with yields
        loop {
            {
                let receiver = self.receiver.lock().unwrap();
                match receiver.try_recv() {
                    Ok(value) => return Ok(value),
                    Err(std::sync::mpsc::TryRecvError::Empty) => {},
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        return Err(NeksisError::ChannelError("Channel closed".to_string()));
                    }
                }
            }
            yield_now().await;
        }
    }
    
    pub fn try_receive(&self) -> NeksisResult<T> {
        let receiver = self.receiver.lock().unwrap();
        receiver.try_recv()
            .map_err(|e| match e {
                std::sync::mpsc::TryRecvError::Empty => 
                    NeksisError::ChannelError("No message available".to_string()),
                std::sync::mpsc::TryRecvError::Disconnected => 
                    NeksisError::ChannelError("Channel closed".to_string()),
            })
    }
}

/// Bounded channel
pub fn bounded_channel<T>(capacity: usize) -> (BoundedSender<T>, BoundedReceiver<T>) {
    let (sender, receiver) = std::sync::mpsc::sync_channel(capacity);
    (
        BoundedSender { sender },
        BoundedReceiver { receiver: Arc::new(Mutex::new(receiver)) }
    )
}

pub struct BoundedSender<T> {
    sender: std::sync::mpsc::SyncSender<T>,
}

impl<T> BoundedSender<T> {
    pub async fn send(&self, mut value: T) -> NeksisResult<()> {
        // For now, use blocking send with yield
        loop {
            match self.sender.try_send(value) {
                Ok(()) => return Ok(()),
                Err(std::sync::mpsc::TrySendError::Full(v)) => {
                    // Put the value back and yield
                    value = v;
                    yield_now().await;
                    continue;
                }
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
                    return Err(NeksisError::ChannelError("Channel closed".to_string()));
                }
            }
        }
    }
    
    pub fn try_send(&self, value: T) -> NeksisResult<()> {
        self.sender.try_send(value)
            .map_err(|e| match e {
                std::sync::mpsc::TrySendError::Full(_) => 
                    NeksisError::ChannelError("Channel is full".to_string()),
                std::sync::mpsc::TrySendError::Disconnected(_) => 
                    NeksisError::ChannelError("Channel closed".to_string()),
            })
    }
}

pub struct BoundedReceiver<T> {
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<T>>>,
}

impl<T> BoundedReceiver<T> {
    pub async fn receive(&self) -> NeksisResult<T> {
        loop {
            {
                let receiver = self.receiver.lock().unwrap();
                match receiver.try_recv() {
                    Ok(value) => return Ok(value),
                    Err(std::sync::mpsc::TryRecvError::Empty) => {},
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        return Err(NeksisError::ChannelError("Channel closed".to_string()));
                    }
                }
            }
            yield_now().await;
        }
    }
    
    pub fn try_receive(&self) -> NeksisResult<T> {
        let receiver = self.receiver.lock().unwrap();
        receiver.try_recv()
            .map_err(|e| match e {
                std::sync::mpsc::TryRecvError::Empty => 
                    NeksisError::ChannelError("No message available".to_string()),
                std::sync::mpsc::TryRecvError::Disconnected => 
                    NeksisError::ChannelError("Channel closed".to_string()),
            })
    }
}

// Async Mutex - TODO: Fix lifetime issues  
/*
pub struct AsyncMutex<T> {
    inner: Arc<Mutex<T>>,
    waiters: Arc<Mutex<VecDeque<Waker>>>,
}

impl<T> AsyncMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
            waiters: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    pub async fn lock(&self) -> AsyncMutexGuard<T> {
        loop {
            if let Ok(guard) = self.inner.try_lock() {
                return AsyncMutexGuard { guard: Some(guard) };
            }
            
            // Add to waiters and yield
            yield_now().await;
        }
    }
    
    pub fn try_lock(&self) -> NeksisResult<AsyncMutexGuard<T>> {
        self.inner.try_lock()
            .map(|guard| AsyncMutexGuard { guard: Some(guard) })
            .map_err(|_| NeksisError::LockError("Failed to acquire lock".to_string()))
    }
}

pub struct AsyncMutexGuard<T: 'static> {
    guard: Option<std::sync::MutexGuard<'static, T>>,
}

impl<T: 'static> std::ops::Deref for AsyncMutexGuard<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

impl<T: 'static> std::ops::DerefMut for AsyncMutexGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().unwrap()
    }
}

/// Join multiple futures concurrently
pub async fn join_all<T>(futures: Vec<impl Future<Output = T>>) -> Vec<T> {
    let mut results = Vec::with_capacity(futures.len());
    
    // Simple sequential execution for now
    for future in futures {
        results.push(future.await);
    }
    
    results
}

/// Create a oneshot channel
pub fn oneshot<T>() -> (OneshotSender<T>, OneshotReceiver<T>) {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    (
        OneshotSender { sender: Some(sender) },
        OneshotReceiver { receiver }
    )
}

pub struct OneshotSender<T> {
    sender: Option<std::sync::mpsc::SyncSender<T>>,
}

impl<T> OneshotSender<T> {
    pub fn send(mut self, value: T) -> NeksisResult<()> {
        if let Some(sender) = self.sender.take() {
            sender.send(value)
                .map_err(|_| NeksisError::ChannelError("Receiver dropped".to_string()))
        } else {
            Err(NeksisError::ChannelError("Already sent".to_string()))
        }
    }
}

pub struct OneshotReceiver<T> {
    receiver: std::sync::mpsc::Receiver<T>,
}

impl<T> OneshotReceiver<T> {
    pub async fn receive(self) -> NeksisResult<T> {
        loop {
            match self.receiver.try_recv() {
                Ok(value) => return Ok(value),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    yield_now().await;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err(NeksisError::ChannelError("Sender dropped".to_string()));
                }
            }
        }
    }
}

/// Interval timer
pub async fn interval(duration: Duration) -> IntervalStream {
    IntervalStream {
        duration,
        next_tick: Instant::now() + duration,
    }
}

pub struct IntervalStream {
    duration: Duration,
    next_tick: Instant,
}

impl IntervalStream {
    pub async fn tick(&mut self) {
        let now = Instant::now();
        if now >= self.next_tick {
            self.next_tick = now + self.duration;
            return;
        }
        
        let remaining = self.next_tick - now;
        sleep(remaining).await;
        self.next_tick = Instant::now() + self.duration;
    }
}
*/
