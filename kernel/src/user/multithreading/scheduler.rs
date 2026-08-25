use crate::user::multithreading::task::{Task, TaskId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use spin::{Mutex, Once};
use crate::{info, println};


const TARGET: &str = "SCHEDULER";

pub static SCHEDULER: Once<Mutex<Executor>> = Once::new();

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
    should_exit: bool,
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
            should_exit: false,
        }
    }

    fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }

        self.task_queue.push(task_id).expect("queue full");
    }

    fn run(&mut self) {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();

            if self.should_exit {
                break;
            }
        }
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
            ..
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let Some(task) = tasks.get_mut(&task_id) else {
                continue; // task no longer exists
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                },
                Poll::Pending => {
                    println!("PENDING")
                }
            }
        }
    }

    fn sleep_if_idle(&self) {
        println!("HALT");
        if self.task_queue.is_empty() {
            x86_64::instructions::hlt();
        }
    }
}

unsafe impl Sync for Executor {}
unsafe impl Send for Executor {}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    #[inline(always)]
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub fn init_scheduler() {
    info!("Initializing scheduler");
    let executor = Executor::new();

    SCHEDULER.call_once(|| Mutex::new(executor));
}

#[inline(always)]
pub fn spawn(future: impl Future<Output = ()> + 'static) {
    let task = Task::new(future);
    SCHEDULER
        .get()
        .unwrap()
        .lock()
        .spawn(task);
}

#[inline(always)]
pub fn run_scheduler() {
    SCHEDULER
        .get()
        .unwrap()
        .lock()
        .run();
}