use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    num::NonZero,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{self, Context, Wake, Waker},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(NonZero<u64>);

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskId({})", self.0)
    }
}

impl TaskId {
    pub(crate) fn next() -> Self {
        static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id.try_into().unwrap())
    }
}

struct TaskWaker {
    schedule_fn: Arc<dyn Fn(TaskId) + Send + Sync>,
    task_id: TaskId,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        (self.schedule_fn)(self.task_id);
    }
}

struct Task {
    fut: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

pub struct VelonaTasksExecutor {
    tasks: HashMap<TaskId, Task>,
    schedule_fn: Arc<dyn Fn(TaskId) + Send + Sync>,
}

impl Debug for VelonaTasksExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VelonaTasksExecutor")
            .field("tasks", &self.tasks.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PollAllResult {
    pub pending: usize,
    pub ready: usize,
}

impl VelonaTasksExecutor {
    pub fn new<S>(schedule: S) -> Self
    where
        S: Fn(TaskId) + Send + Sync + 'static,
    {
        Self {
            tasks: Default::default(),
            schedule_fn: Arc::new(schedule),
        }
    }
    pub fn spawn<F>(&mut self, task: F) -> Option<TaskId>
    where
        F: Future<Output = ()> + 'static,
    {
        let task_id = TaskId::next();
        let mut task = Box::pin(task);
        let waker = self.get_task_waker(task_id);
        let res = task.as_mut().poll(&mut Context::from_waker(&waker));
        match res {
            std::task::Poll::Ready(_) => {
                log::trace!("Task already done, no need for inserting it in the executor");
                None
            }
            std::task::Poll::Pending => {
                self.tasks.insert(task_id, Task { fut: task, waker });
                Some(task_id)
            }
        }
    }
    pub fn poll_task(&mut self, task_id: TaskId) {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            match task
                .fut
                .as_mut()
                .poll(&mut Context::from_waker(&task.waker))
            {
                std::task::Poll::Ready(_) => {
                    self.tasks.remove(&task_id);
                }
                std::task::Poll::Pending => {
                    log::trace!("{task_id} is still pending");
                }
            }
        }
    }
    fn get_task_waker(&self, task_id: TaskId) -> task::Waker {
        unsafe {
            Waker::from_raw(
                Arc::new(TaskWaker {
                    schedule_fn: self.schedule_fn.clone(),
                    task_id,
                })
                .into(),
            )
        }
    }
    pub fn poll_all(&mut self) -> PollAllResult {
        let mut to_remove_stack = Vec::new();
        let mut res = PollAllResult {
            ready: 0,
            pending: 0,
        };
        for (task_id, task) in self.tasks.iter_mut() {
            match task
                .fut
                .as_mut()
                .poll(&mut Context::from_waker(&task.waker))
            {
                task::Poll::Ready(_) => {
                    to_remove_stack.push(*task_id);
                    res.ready += 1
                }
                task::Poll::Pending => {
                    log::trace!("{task_id} is still pending");
                    res.pending += 1;
                }
            }
        }
        for id in to_remove_stack {
            self.tasks.remove(&id);
        }
        res
    }
    pub fn shrink_to_fit(&mut self) {
        self.tasks.shrink_to_fit();
    }
    pub fn tasks_count(&self) -> usize {
        self.tasks.len()
    }
}

#[cfg(test)]
mod tests {
    use std::{future::pending, sync::mpsc};

    use super::*;

    #[test]
    fn test_run() {
        let (sender, receiver) = mpsc::channel::<TaskId>();

        let mut executor = VelonaTasksExecutor::new(move |task_id| {
            let _ = sender.send(task_id);
        });

        assert!(
            executor
                .spawn(async {
                    println!("done");
                })
                .is_none()
        );

        executor.spawn(async { pending().await });
        assert_eq!(executor.tasks_count(), 1);
        assert_eq!(receiver.try_iter().count(), 0);
    }
}
