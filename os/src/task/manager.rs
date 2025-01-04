//!Implementation of [`TaskManager`]
use super::{TaskControlBlock, TaskStatus};
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    // pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
    //     self.ready_queue.pop_front()
    // }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_stride = i32::MAX;
        let mut min_id = None;

        for (id, task) in self.ready_queue.iter().enumerate() {
            let inner = task.inner_exclusive_access();
            let stride = inner.stride;
            if inner.task_status == TaskStatus::Ready && stride < min_stride {
                min_stride = stride;
                min_id = Some(id);
            }
        }

        if min_id.is_none() {
            return None;
        }

        let min_id = min_id.unwrap();

        if let Some(task) = self.ready_queue.get(min_id) {
            let mut inner = task.inner_exclusive_access();
            inner.stride +=  255/ inner.priority;
        }
        self.ready_queue.remove(min_id)
        // self.ready_queue.pop_front()
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
