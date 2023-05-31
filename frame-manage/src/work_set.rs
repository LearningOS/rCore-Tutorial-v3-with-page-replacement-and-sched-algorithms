use core::marker::PhantomData;
use core::option::Option;

use crate::{ACCESS_FLAG, WORKSET_NUM};
use crate::plugins::{Manage, handle_pagefault};
use crate::frame_allocator::{FrameTracker, frame_check};
use alloc::vec::Vec;
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, Pte, PPN};
use alloc::collections::{VecDeque, BTreeMap, BTreeSet};

pub struct WorkSetManager<Meta: VmMeta, M: PageManager<Meta> + 'static> {
    queue: BTreeMap<usize, VecDeque<(VPN<Meta>, PPN<Meta>, FrameTracker)>>, // key = task_id
    workset: VecDeque<BTreeSet<(usize, VPN<Meta>)>>,
    ptr: usize,

    dummy: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> WorkSetManager<Meta, M> {
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

    fn pop_first(&mut self) -> (PPN<Meta>, VPN<Meta>, usize) {
        let (&id, _) = self.queue.first_key_value().unwrap();
        let list = self.queue.get_mut(&id).unwrap();
        let entry = list.pop_front().unwrap();
        if list.len() == 0 {
            self.queue.pop_first();
        }
        (entry.1, entry.0, id)
    }

    fn compute_workset<F>(&mut self, get_memory_set: &F, ptr: usize, reset: bool) 
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        while self.workset.len() <= ptr {
            self.workset.push_back(BTreeSet::new());
        }
        let work_set = &mut self.workset[ptr];
        work_set.clear();
        self.queue.iter().for_each(|(task_id, list)| {
            let mem_set = get_memory_set(*task_id);
            list.iter().for_each(|(vpn, ppn, frame)| {
                if Self::has_accessed(mem_set, vpn) {
                    work_set.insert((*task_id, *vpn));

                    if reset {
                        Self::clear_accessed(mem_set, vpn);
                    }
                }
            });
        });
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for WorkSetManager<Meta, M> {
    fn new() -> Self {
        let mut workset = VecDeque::new();
        Self { queue: BTreeMap::new(), workset: workset, ptr: 0, dummy: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_pagefault(get_memory_set, vpn, task_id, self);
    }

    fn insert_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
        if let Some(list) = self.queue.get_mut(&task_id) {
            list.push_back((vpn,ppn, frame));
        } else {
            let mut tmp = VecDeque::new();
            tmp.push_back((vpn, ppn, frame));
            self.queue.insert(task_id, tmp);
        }
    }

    fn work<F>(&mut self, get_memory_set: &F, task_id: usize) -> Vec<(PPN<Meta>, VPN<Meta>, usize)> 
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> { 
        self.compute_workset(get_memory_set, WORKSET_NUM, false);
        let work_set = &self.workset;

        let mut ret = Vec::new();
        self.queue.iter_mut().for_each(|(task_id, list)| {
            let remove_list: Vec<usize> =  list.iter().enumerate().filter_map(|(i, (vpn, ppn, frame))| {
                let mut flag = false;
                for set_id in 0..(WORKSET_NUM+1) {
                    if work_set[set_id].contains(&(*task_id, *vpn)) {
                        flag = true;
                        break;
                    }
                }

                if !flag { Some(i) } else { None }
            }).collect();

            remove_list.iter().rev().for_each(|&i| { 
                let (vpn, ppn, _) = list.remove(i).unwrap();
                ret.push((ppn, vpn, *task_id));
            });
        });
        ret
    }

    fn clear_frames(&mut self, task_id: usize) {
        self.queue.remove(&task_id);
    }

    fn handle_time_interrupt<F>(&mut self, get_memory_set: &F) 
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        self.compute_workset(get_memory_set, self.ptr, true);
        self.ptr = (self.ptr + 1) % WORKSET_NUM;
    }
}