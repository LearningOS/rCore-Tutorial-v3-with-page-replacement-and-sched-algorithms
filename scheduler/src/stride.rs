use core::marker::Copy;
use core::cmp::{Ord, Ordering};
use core::clone::Clone;
use alloc::collections::{BTreeMap, BinaryHeap};
use crate::Manage;
use crate::Schedule;
use crate::syscall_args::*;


const BIG_STRIDE: usize = usize::max_value();

#[derive(Clone, Copy)]
struct StrideBlock<I: Copy + Ord> {
    task_id: I,
    priority: usize,
    stride: usize
}

impl<I: Copy + Ord> PartialOrd for StrideBlock<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let diff = if self.stride > other.stride { self.stride - other.stride } else { other.stride - self.stride };
        if diff <= BIG_STRIDE / 2 {  // no overflow in stride
            other.stride.partial_cmp(&self.stride)
        } else {
            self.stride.partial_cmp(&other.stride)
        }
    }
}

impl<I: Copy + Ord> PartialEq for StrideBlock<I> {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

impl<I: Copy + Ord> Ord for StrideBlock<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<I: Copy + Ord> Eq for StrideBlock<I> {}

pub struct StrideManager<T, I: Copy + Ord>{
    tasks: BTreeMap<I, T>,
    info_map: BTreeMap<I, StrideBlock<I>>,
    heap: BinaryHeap<StrideBlock<I>>,
    current: Option<I>
}

impl<T, I: Copy + Ord> StrideManager<T, I> {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            info_map: BTreeMap::new(),
            heap: BinaryHeap::new(),
            current: None
        }
    } 
}

impl<T, I: Copy + Ord> Manage<T, I> for StrideManager<T, I> {
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
        if let Some(cur_id) = self.current {
            if cur_id == id {
                self.current = None;
            }
        }
    }
}

impl<T, I: Copy + Ord> Schedule<I> for StrideManager<T, I> {
    fn add(&mut self, id: I) {
        let block = match self.info_map.get(&id) {
            None => {
                let tmp = StrideBlock { task_id: id, priority: 16, stride: 0 };
                self.info_map.insert(id.clone(), tmp.clone());
                tmp
            },
            Some(&t) => t
        };
        self.heap.push(block);
    }

    fn fetch(&mut self) -> Option<I> {
        match self.heap.pop() {
            None => None,
            Some(tb) => Some(tb.task_id)
        }
    }

    fn update_exec(&mut self, id: I, args: &ExecArgs) {
        let block = self.info_map.get_mut(&id).unwrap();
        block.priority = args.priority
    }

    fn update_fork(&mut self, parent_id: I, child_id: I) {
        let block = self.info_map.get(&parent_id).unwrap();
        self.info_map.insert(child_id, StrideBlock { 
            task_id: child_id,
            ..block.clone() 
        });
    }

    fn update_sched_to(&mut self, id: I, time: usize) {
        if let None = self.current {
            self.current = Some(id);
        } else {
            panic!("call sched while current is not suspended!");
        }
    }

    fn update_suspend(&mut self, id: I, time: usize) {
        if let Some(cur_id) = self.current {
            if cur_id != id {
                panic!("suspend wrong id? ");
            }

            self.current = None;
            let block = self.info_map.get_mut(&id).unwrap();
            block.stride = block.stride.wrapping_add(BIG_STRIDE / block.priority);
        } else {
            panic!("call suspend but current is none! ");
        }
        
    }

    fn update_sleep(&mut self, id: I) {}
}