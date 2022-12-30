use std::{thread::JoinHandle, sync::atomic::AtomicBool};
use ::std::{
    sync::{mpsc, Arc},
    thread,
};

/// Thread worker utility class facilitating ergonomic communication with background thread workers
/// using thread channels. It can also be used with UI event loops such as [winit] (see example below).
/// 
/// ## Usage
/// **Example #1**:
/// ```ignore
/// let mut bg_worker = ThreadWorker::new(|| { do_some_initializing_work(); });
/// let channel_rcv = bg_worker.run(|| { compute_stuff(); });
/// ```
/// since channel_rcv is a [std::sync::mpsc::Receiver], it can be waited on with
/// ```ignore
/// let operation_result = channel_rcv.recv().unwrap(); // this will block
/// // for polling, use try_recv()
/// ```
/// 
/// **Example #2 with [winit] EventLoop**
/// ```ignore
/// fn main() {
///     use std::time;
///     use winit::{EventLoopBuilder, WindowBuilder};
///     let mut bg_worker = ThreadWorker::new(|| { do_some_initializing_work(); });
///     let event_loop = EventLoopBuilder::with_user_event().build();
///     let window = WindowBuilder::new()
///         .with_title("My Fancy App")
///         .build(&event_loop);
/// 
///     bg_worker.run_async(
///         || { String::from("Hello from the worker!"); },
///         | incoming | {
///             if let Ok(result_msg) = incoming.downcast_ref() {
///                 println!("{}", result_msg); 
///                 // will print "Hello from the worker!"
///             }
///         }
///     );
/// 
///     event_loop.run(|move |event: Event<UserEvent>, _, control_flow: &mut ControlFlow| {
///         *control_flow = ControlFlow::Poll; // using poll causes winit EventLoop to keep spinning
///         match event { ... };
///         
///         worker.tick(); // if tick() is not called, result callbacks passed to run_async won't get called
///         std::thread::sleep(Duration::from_millis(1));
///     })
/// }
/// ```
/// ## Under the hood
/// [ThreadWorker] is a simple wrapper using [std::sync::mpsc::channel] to raise and send work to an OS
/// thread. The run and run_async functions take closures that are sent to the thread to be executed.
/// The run function exposes the underlying [std::sync::mpsc::Receiver] and can be used to customize 
/// synchronization between the threads.
/// 
/// For event loops, the [`run_async`](ThreadWorker::run_async) and `tick` functions allow polling
/// behaviors. (See example #2). The run_async takes a task closure and a result closure. It sends
/// the task closure to the background thread but saves the task handle and result closure internally.
/// Every time `tick()` is called on the calling thread, we check if we have any finished tasks and
/// call the success callback for the finished tasks.
/// 
/// We have to leverage [std::any::Any] to be able to store result callbacks as Rust currently lacks
/// generic functions and thus we cannot define a type of function that is abstract over its parameters
/// <sup>[Relevant rust-lang discussion.](https://github.com/rust-lang/rust/issues/10124)</sup>.
/// 
/// Since we can't type it, we rely on [std::any::Any] as a type instead. This means the compiler cannot infer
/// what type we receive in the success closure so we have to manually know and `downcast_ref` the value
/// to extract the worker operation result.
/// 
pub struct ThreadWorker {
    task_sender: mpsc::Sender<Box<dyn FnOnce() + Send>>,
    pub join_handle: JoinHandle<()>,
    tasks_results: Vec<(ResultCallback, std::sync::mpsc::Receiver<AnythingBoxed>)>,
    stop: Arc<AtomicBool>,
    exited: Arc<AtomicBool>,
}

pub type AnythingBoxed = Box<dyn std::any::Any + Send>;
pub type ResultCallback = Box<dyn FnOnce(AnythingBoxed)>;

impl ThreadWorker {
    pub fn new<LoopState>(thread_prelude: impl 'static + Send + FnOnce() -> LoopState) -> Self {
        let (task_sender, to_be_received_tasks) = mpsc::channel();
        let _: mpsc::Sender<Box<dyn FnOnce() + Send>> = task_sender;
        let stop_flag = Arc::new(AtomicBool::new(false));
        let exited_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
        let exited_flag_clone = exited_flag.clone();
        let join_handle = ::std::thread::spawn(move || {
            let _state = thread_prelude();
            loop { // TODO: Condvar
                let task_opt = to_be_received_tasks.try_recv();
                if let Ok(task) = task_opt {
                    task();
                } else {
                    if stop_flag_clone.load(std::sync::atomic::Ordering::Relaxed) != true {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    } else {
                        break;
                    }
                    
                }
            }
            exited_flag_clone.store(true, std::sync::atomic::Ordering::Relaxed);
        });
        Self {
            task_sender,
            join_handle,
            tasks_results: Default::default(),
            stop: stop_flag,
            exited: exited_flag,
        }
    }

    /// Sends the given closure to the background thread and returns the [std::sync::mpsc::Receiver]
    pub fn run<R>(self: &'_ ThreadWorker, f: impl 'static + Send + FnOnce() -> R) -> std::sync::mpsc::Receiver<R>
    where
        R: 'static + Send,
    {
        let (sender, receiver) = mpsc::channel();
        let _ = self.task_sender.send(Box::new(move || {
            let _ = sender.send(f());
        }));
        receiver
    }

    /// Sends the `task` closure to the thread and saves the `result_cb` to be called when the `task` is complete.
    /// The `result_cb` is invoked on the calling thread **only if tick() is called** in a polling loop. See module examples.
    pub fn run_async<R: std::any::Any>(&mut self, task: impl 'static + Send + FnOnce() -> R, result_cb: impl 'static + Fn(AnythingBoxed))
    where
        R: 'static + Send
    {
        let (sender, receiver) = mpsc::channel::<AnythingBoxed>();
        let _ = self.task_sender.send(Box::new(move || {
            let result: AnythingBoxed = Box::new(task());
            let _ = sender.send(result);
        }));
        let rcb: ResultCallback = Box::new(result_cb);
        self.tasks_results.push((rcb, receiver));
    }

    /// Looks at the pending background tasks and calls the result closures for the completed ones. Returns true
    /// if there are any pending tasks, false when all tasks have completed.
    /// Example:
    /// ```ignore
    /// let mut bg_worker = ThreadWorker::new(|| {});
    /// while bg_worker.tick() {
    ///     std::thread::sleep(Duration::from_millis(1));
    /// }
    /// ```
    pub fn tick(&mut self) -> bool {
        for i in 0..self.tasks_results.len() {
            if let Some(task) = self.tasks_results.get(i) {
                let (_, receiver) = task;
                if let Ok(res) = receiver.try_recv() {
                    let (cb, _) = self.tasks_results.remove(i);
                    cb(res);
                }
            } else {
                // panic!("How do you loop through an array but get None?");
            }
        }

        self.tasks_results.len() != 0
    }

    pub fn is_finished(&self) -> bool {
        return self.exited.load(std::sync::atomic::Ordering::Relaxed);
    }

    pub fn stop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
