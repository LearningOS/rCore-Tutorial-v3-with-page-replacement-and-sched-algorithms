
use core::marker::PhantomData;
use core::option::Option;

use crate::{plugins::{handle_local_pagefault, Manage}, frame_allocator::FrameTracker};
use kernel_vm::{AddressSpace, VmMeta, PageManager, VPN};
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

impl<Meta: VmMeta, M: PageManager<Meta>> Manage<Meta, M> for FIFOManager<Meta, M> {
    fn new() -> Self {
        Self { queue: VecDeque::new(), manager: PhantomData }
    }

    fn handle_pagefault(&mut self, memory_set: &mut AddressSpace<Meta, M>, vpn: VPN<Meta>, token: usize) {
        handle_local_pagefault(memory_set, vpn, token, self)
    }

    fn insert_frame(&mut self, vpn: VPN<Meta>, frame: FrameTracker) {
        self.queue.push_back((vpn, frame));
    }

    fn get_next_frame(&mut self, memory_set: &mut AddressSpace<Meta, M>) -> Option<VPN<Meta>> {
        Some(self.queue.pop_front()?.0)
    }
}
