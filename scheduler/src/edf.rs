use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use alloc::collections::{BTreeMap, BinaryHeap};
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;
use rcore_utils::get_time_ms;

#[derive(Clone, Copy)]
struct EDFBlock<I: Copy + Ord> {
    task_id: I,
    period: isize,
    init_ddl: isize,
    next_ddl: isize
}

impl<I: Copy + Ord> PartialOrd for EDFBlock<I>  {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.next_ddl.partial_cmp(&self.next_ddl)
    }
}

impl<I: Copy + Ord> PartialEq for EDFBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl<I: Copy + Ord> Ord for EDFBlock<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<I: Copy + Ord> Eq for EDFBlock<I> {}

pub struct EDFManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    info_map: BTreeMap<I, EDFBlock<I>>,
    heap: BinaryHeap<EDFBlock<I>>
}

impl<T, I: Copy + Ord> EDFManager<T, I> {
    pub fn new() -> Self {
        Self { 
            tasks: BTreeMap::new(), 
            info_map: BTreeMap::new(), 
            heap: BinaryHeap::new()
        }
    }
}


impl<T, I: Copy + Ord> Manage<T, I> for EDFManager<T, I> {
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
        self.info_map.remove(&id);
    }
}

impl<T, I: Copy + Ord> Schedule<I> for EDFManager<T, I> {
    fn add(&mut self, id: I) {
        let block = match self.info_map.get(&id) {
            None => {
                let tmp = EDFBlock { 
                    task_id: id.clone(), 
                    init_ddl: isize::MAX,
                    period: -1,
                    next_ddl: isize::MAX
                };
                self.info_map.insert(id, tmp.clone());
                tmp
            },
            Some(&t) => t
        };
        self.heap.push(block);
    }

    fn fetch(&mut self) -> Option<I> {
        let t = get_time_ms() as isize;
        loop {
            if let Some(mut tb) = self.heap.pop() {
                if tb.next_ddl <= t{
                    if tb.period > 0 {
                        let diff = t - tb.next_ddl;
                        let times = diff / tb.period + 1;
                        tb.next_ddl += tb.period * times;
                        self.info_map.get_mut(&tb.task_id).unwrap().next_ddl = tb.next_ddl;
                        self.heap.push(tb);
                    } else {
                        return Some(tb.task_id);
                    }
                } else {
                    return Some(tb.task_id);
                }
            } else {
                return None;
            }
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        let block = self.info_map.get_mut(&id).unwrap();
        block.period = args.period;
        block.init_ddl = args.init_ddl;
        block.next_ddl = args.init_ddl;
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let block = self.info_map.get(&parent_id).unwrap();
        self.info_map.insert(child_id, EDFBlock { task_id: child_id, ..block.clone() });
    }

    fn update_sched_to(&mut self, id: I, time: usize) {}

    fn update_suspend(&mut self, id: I, time: usize) {}

    fn update_sleep(&mut self, id: I) {}
}