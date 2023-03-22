use alloc::collections::{BinaryHeap, BTreeMap, VecDeque};
use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use core::option::Option;
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;

struct STCFTaskBlock<I: Copy + Ord> {
    task_id: I,
    time_total: isize,
    time_left: isize
}

impl<I: Copy + Ord> PartialOrd for STCFTaskBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.time_left != other.time_left {
            other.time_left.partial_cmp(&self.time_left)
        } else {
            self.task_id.partial_cmp(&other.task_id)
        }
    }
}

impl<I: Copy + Ord> PartialEq for STCFTaskBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl<I: Copy + Ord> Ord for STCFTaskBlock<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.time_left != other.time_left {
            other.time_left.cmp(&self.time_left)
        } else {
            self.task_id.cmp(&other.task_id)
        }
    }
}

impl<I: Copy + Ord> Eq for STCFTaskBlock<I> {}

pub struct STCFManager<T, I: Copy + Ord> {
    tasks: BTreeMap<I, T>,
    time_map: BTreeMap<I, (isize, isize)>, // (time_total, time_left)
    heap: BinaryHeap<STCFTaskBlock<I>>,
    current: Option<(I, usize)>  // (id, st_time)
}

impl<T, I: Copy + Ord> STCFManager<T, I> {
    pub fn new() -> Self {
        Self { 
            tasks: BTreeMap::new(), 
            time_map: BTreeMap::new(), 
            heap: BinaryHeap::new(),
            current: None
        }
    }

    // fn update_total_time(&mut self, id: &I, new_time: isize) {
    //     if new_time <= 0 {
    //         panic!("total time can't be negative or zero! ");
    //     }

    //     if let Some((total_, left_)) = self.time_map.get(id) {
    //         let done = total_ - left_;
    //         let mut new_left = new_time - done;
    //         new_left = if new_left < 0 { 0 } else { new_left };
    //         self.time_map.insert(id.clone(), (new_time, new_left));
    //     } else {
    //         // not registered
    //         self.time_map.insert(id.clone(), (new_time, new_time));
    //     }
    // }

    fn update_left_time(&mut self, id:I, time_pass: isize) {
        match self.time_map.get(&id) {
            None => { panic!("try to update left time for non-exist!"); },
            Some(&(total_, old_time)) => {
                let new_time = old_time - time_pass;
                self.time_map.insert(id, (total_, new_time));
            }
        }  
    }
}

impl<T, I: Copy + Ord> STCFManager<T, I> {
    pub fn get_list(&self) -> VecDeque<(I, isize, isize)> {
        let ret: VecDeque<(I, isize, isize)> = self.heap.iter().map(|block| (block.task_id, block.time_total, block.time_left)).collect();
        ret
    }
}

impl<T, I: Copy + Ord> Manage<T, I> for STCFManager<T, I>{
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
        if let Some((cur_id, _)) = self.current {
            if cur_id == id {
                self.current = None
            }
        }
    }
}

impl<T, I: Copy + Ord> Schedule<I> for STCFManager<T, I> {
    fn add(&mut self, id: I) {
        let (total_, left_) = match self.time_map.get(&id) {
            None => (isize::MAX, isize::MAX),
            Some(t) => *t
        };
        self.heap.push(STCFTaskBlock { 
            task_id: id, 
            time_total: total_, 
            time_left: left_ 
        });
        self.time_map.insert(id, (total_, left_));
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(tb) => Some(tb.task_id)
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        self.time_map.insert(id, (args.total_time, args.total_time));
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let time_pair = match self.time_map.get(&parent_id) {
            None => (isize::MAX, isize::MAX),
            Some(&t) => t
        };
        self.time_map.insert(child_id, time_pair);
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        if let None = self.current {
            self.current = Some((id, time));
        } else {
            panic!("call sched while current is not suspended!")
        }
    }

    fn update_suspend(&mut self, id: I, time: usize) {
        if let Some((cur_id, st_time)) = self.current {
            if cur_id != id {
                panic!("suspend wrong id? ");
            }

            self.current = None;
            let time_pass = time - st_time;
            self.update_left_time(id, time_pass as isize)
        } else {
            panic!("call suspend but current is none! ")
        }
    }

    fn update_sleep(&mut self, id: I) {}
}