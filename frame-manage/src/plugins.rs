use core::option::Option;
use kernel_vm::{PageManager, AddressSpace, VmMeta, VPN, PPN};
use crate::{frame_allocator::{frame_alloc, frame_check, FrameTracker}, config::PAGE_SIZE, virt_frame_swapper::IDE_MANAGER};

pub trait Manage<Meta: VmMeta, M: PageManager<Meta>> {
    fn new() -> Self;

    fn handle_pagefault(&mut self, memory_set: &mut AddressSpace<Meta, M>, vpn: VPN<Meta>, task_id: usize);

    fn insert_frame(&mut self, vpn: VPN<Meta>, frame: FrameTracker);

    fn get_next_frame(&mut self, memory_set: &mut AddressSpace<Meta, M>) -> Option<VPN<Meta>>;

    // fn handle_read(&mut self, token: usize, ppn: PPN<Meta>);

    // fn handle_write(&mut self, token: usize, ppn: PPN<Meta>);
}

pub fn ppn_base<Meta: VmMeta>(ppn: &PPN<Meta>) -> usize {
    ppn.val() << Meta::PAGE_BITS
}

pub fn handle_local_pagefault<Meta, M, T>(memory_set: &mut AddressSpace<Meta, M>, vpn: VPN<Meta>, task_id: usize, manager: &mut T)
where Meta: VmMeta, 
    M: PageManager<Meta>,
    T: Manage<Meta, M>
{
    // finding the area contains vpn
    let res = memory_set.areas.iter_mut().enumerate().find(|(_, _area)| vpn >= _area.range.start && vpn <= _area.range.end);
    match res {
        None => panic!("this vpn is not mapped in memeory set"),
        Some((id, _)) => {
            if !frame_check() { // no space left in frame allocator
                let vpn_swap = manager.get_next_frame(memory_set).unwrap();
                let ppn_swap = memory_set.translate_to_pte(vpn_swap.base()).unwrap().ppn();
                let old_data = unsafe { core::slice::from_raw_parts_mut(ppn_base(&ppn_swap) as *mut u8, PAGE_SIZE) };
                unsafe { IDE_MANAGER.swap_in(task_id, vpn_swap.val(), old_data) } // swap vpn to disk

                // set vpn_swap to invalid in the area 
                // todo: multiple vpn point to the same ppn
                memory_set.unmap_one_in_exist_range(vpn_swap);
            }

            let frame = frame_alloc().unwrap();
            let ppn = PPN::new(frame.ppn);
            // insert frame to area
            memory_set.map_to_exist_range(id, vpn.clone(), ppn);
            manager.insert_frame(vpn, frame);
            unsafe {  
                if IDE_MANAGER.check(task_id, vpn.val())  {
                    let dst = core::slice::from_raw_parts_mut(ppn_base(&ppn) as *mut u8, PAGE_SIZE);
                    IDE_MANAGER.swap_out(task_id, vpn.val(), dst); // swap orig data save in disk to frame
                }
            }
        }
    }
}