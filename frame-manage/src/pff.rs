use core::marker::PhantomData;
use core::option::Option;

use crate::{ACCESS_FLAG, PFF_T};
use crate::plugins::{Manage, handle_pagefault};
use crate::frame_allocator::{FrameTracker, frame_check};
use alloc::vec::Vec;
use alloc::vec;
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, Pte, PPN};
use alloc::collections::{VecDeque, BTreeMap};

use rcore_utils::get_time;

pub struct PffManager<Meta: VmMeta, M: PageManager<Meta> + 'static> {
    queue: BTreeMap<usize, VecDeque<(PPN<Meta>, VPN<Meta>, FrameTracker)>>, // key = task_id
    last_pgfault: usize, // timestamp
    dummy: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> PffManager<Meta, M> {
    fn get_pte(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> Option<Pte<Meta>> {
        memory_set.translate_to_pte(vpn.base())
    }

    fn has_accessed(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) -> bool {
        let pte = Self::get_pte(memory_set, vpn).unwrap();
        let flags = pte.flags();
        (flags.val() & ACCESS_FLAG) != 0 
    }

    fn clear_accessed(memory_set: &mut AddressSpace<Meta, M>, vpn: &VPN<Meta>) {
        memory_set.clear_accessed(*vpn);
    }

    fn pop_unaccessed_frames<F>(&mut self, get_memory_set: &F) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        let mut ret: Vec<(PPN<Meta>, VPN<Meta>, usize)> = vec![];
        let topop: Vec<usize> = self.queue.iter_mut()
            .filter_map(|(task_id, list)| {
                let mem_set = get_memory_set(*task_id);
                (0..list.len()).rev()
                    .for_each(|i| {
                        let _vpn = list[i].1;
                        if !Self::has_accessed(mem_set, &_vpn) {
                            let (ppn, vpn, _) = list.remove(i).unwrap();
                            ret.push((ppn, vpn, *task_id));
                        } else {
                            Self::clear_accessed(mem_set, &_vpn);
                        }
                    });
                
                if list.is_empty() {
                    Some(*task_id)
                } else {
                    None
                }
            }).collect();
        topop.iter().for_each(|id| { self.queue.remove(id); });
        ret
    }

    fn pop_first(&mut self, task_id: usize) -> (PPN<Meta>, VPN<Meta>, usize) {
        let list = self.queue.get_mut(&task_id).unwrap();
        let entry = list.pop_front().unwrap();
        if list.len() == 0 {
            self.queue.pop_first();
        }
        (entry.0, entry.1, task_id)
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for PffManager<Meta, M> {
    fn new() -> Self {
        Self { queue: BTreeMap::new(), last_pgfault: usize::MAX, dummy: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_pagefault(get_memory_set, vpn, task_id, self);
    }

    fn insert_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
        if let Some(vec) = self.queue.get_mut(&task_id) {
            vec.push_back((ppn, vpn, frame))
        } else {
            let mut tmp = VecDeque::new();
            tmp.push_back((ppn, vpn, frame));
            self.queue.insert(task_id, tmp);
        }
    }

    fn work<F>(&mut self, get_memory_set: &F, task_id: usize) -> Vec<(PPN<Meta>, VPN<Meta>, usize)> 
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
            if self.last_pgfault == usize::MAX {
                self.last_pgfault = get_time();
            }
            let cur_time = get_time();
            if cur_time - self.last_pgfault > PFF_T {
                let mut ret = self.pop_unaccessed_frames(get_memory_set);
                if !frame_check() && ret.len() == 0 {
                    ret.push(self.pop_first(task_id));
                }
                ret
            } else {
                if !frame_check() {
                    vec![self.pop_first(task_id)]
                } else {    
                    Vec::new()
                }
            }
    }

    fn clear_frames(&mut self, task_id: usize) {
        self.queue.remove(&task_id);
    }

    fn handle_time_interrupt<F>(&mut self, get_memory_set: &F) 
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {}
}