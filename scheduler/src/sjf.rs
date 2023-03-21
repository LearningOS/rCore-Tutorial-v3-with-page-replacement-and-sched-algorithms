use alloc::collections::{BinaryHeap, BTreeMap};
use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use core::option::Option;
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;


struct SJFTaskBlock<I: Copy + Ord> {
    task_id: I,
    time: usize
}

impl<I: Copy + Ord> PartialOrd for SJFTaskBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.time.partial_cmp(&self.time) //reverse order
    }
}

impl<I: Copy + Ord> PartialEq for SJFTaskBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<I: Copy + Ord> Ord for SJFTaskBlock<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

impl<I: Copy + Ord> Eq for SJFTaskBlock<I> {}


pub struct SJFManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    time_map: BTreeMap<I, usize>,
    heap: BinaryHeap<SJFTaskBlock<I>> // max-heap, reverse Ord to get a min-heap
}

impl<T, I: Copy + Ord> SJFManager<T, I> {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            time_map: BTreeMap::new(),
            heap: BinaryHeap::new()
        }
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for SJFManager<T, I>{
    /// 插入一个新任务
    #[inline]
    fn insert(&mut self, id: I, task: T) {
        self.tasks.insert(id, task);
    }
    /// 根据 id 获取对应的任务
    #[inline]
    fn get_mut(&mut self, id: I) -> Option<&mut T> {
        self.tasks.get_mut(&id)
    }
    /// 删除任务实体
    #[inline]
    fn delete(&mut self, id: I) {
        self.tasks.remove(&id);
        self.time_map.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for SJFManager<T, I> {
    fn add(&mut self, id: I) {
        let time_ = match self.time_map.get(&id) {
            None => isize::MAX as usize,
            Some(t) => *t
        };
        self.heap.push(SJFTaskBlock {
            task_id: id,
            time: time_
        });
        self.time_map.insert(id, time_);
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(tb) => Some(tb.task_id)
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        self.time_map.insert(id, args.time);
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let time_ = match self.time_map.get(&parent_id) {
            None => isize::MAX as usize,
            Some(t) => *t
        };
        self.time_map.insert(child_id, time_);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {}
    fn update_suspend(&mut self, id: I, time: usize) {}
    fn update_sleep(&mut self, id: I) {}
}