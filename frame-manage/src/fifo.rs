
use core::marker::PhantomData;

use crate::{plugins::{handle_local_pagefault, Manage}, frame_allocator::FrameTracker};
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN, PPN};
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::VecDeque;


pub struct FIFOManager<Meta: VmMeta, M: PageManager<Meta>> {
    queue: VecDeque<(VPN<Meta>, FrameTracker)>,
    manager: PhantomData<M>
}

impl<Meta: VmMeta, M: PageManager<Meta>> Clone for FIFOManager<Meta, M> {
    fn clone(&self) -> Self {
        Self { queue: self.queue.clone(), manager: PhantomData }
    }
}

impl<Meta: VmMeta, M: PageManager<Meta> + 'static> Manage<Meta, M> for FIFOManager<Meta, M> {
    fn new() -> Self {
        Self { queue: VecDeque::new(), manager: PhantomData }
    }

    fn handle_pagefault<F>(&mut self, get_memory_set: &F, vpn: VPN<Meta>, task_id: usize)
            where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        handle_local_pagefault(get_memory_set, vpn, task_id, self)
    } 

    fn insert_frame(&mut self, vpn: VPN<Meta>, ppn: PPN<Meta>, task_id: usize, frame: FrameTracker) {
        self.queue.push_back((vpn, frame));
    }

    fn work<F>(&mut self, get_memory_set: &F) -> Vec<(PPN<Meta>, VPN<Meta>, usize)>  
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, M> {
        vec![(PPN::new(0), self.queue.pop_front().unwrap().0, 0)]
    }

    fn clear_frames(&mut self, task_id: usize) {}
}
