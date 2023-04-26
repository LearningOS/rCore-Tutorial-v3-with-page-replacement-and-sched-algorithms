use core::marker::PhantomData;
use alloc::collections::BTreeMap;
use kernel_vm::{PageManager, AddressSpace, VmMeta, VAddr, VmFlags};
use crate::plugins::Manage;
use crate::config::GLOBAL_ID;

pub struct PageFaultHandler<Meta: VmMeta, PM: PageManager<Meta> + 'static, FM: Manage<Meta, PM>> {
    enable_pagefault: bool,
    managers: BTreeMap<usize, FM>,

    dummy1: PhantomData<Meta>,
    dummy2: PhantomData<PM>
}

impl<Meta: VmMeta, PM: PageManager<Meta> + 'static, FM: Manage<Meta, PM> + Clone> PageFaultHandler<Meta, PM, FM> {
    pub const fn new() -> Self {
        if cfg!(feature = "none") {
            Self { enable_pagefault: false, managers: BTreeMap::new(), dummy1: PhantomData::<Meta>, dummy2: PhantomData::<PM> }
        } else {
            Self { enable_pagefault: true, managers: BTreeMap::new(), dummy1: PhantomData::<Meta>, dummy2: PhantomData::<PM> }
        }
    }

    pub fn handle_pagefault<F>(&mut self, addr: usize, flag: usize, get_memory_set: &F, task_id: usize) 
        where F: Fn(usize) -> &'static mut AddressSpace<Meta, PM>{
        if self.enable_pagefault {
            // check if the addr is already mapped to memory set
            let vaddr: VAddr<Meta> = VAddr::from(addr);
            if let Some(pte) = get_memory_set(task_id).translate_to_pte(vaddr) {
                if pte.is_valid() && !pte.flags().contains(unsafe { VmFlags::from_raw(flag) }) {
                    panic!("[PAGE FAULT]: unsupported flags, require={}, orig={}", flag, pte.flags().val());
                }
            }

            // handle page fault
            let _id = if cfg!(feature = "pff") { GLOBAL_ID } else { task_id };
            match self.managers.get_mut(&_id) {
                None => {
                    panic!("[PAGE FAULT]: frame manager not set");
                },
                Some(fm) => {
                    let vpn = vaddr.floor();
                    fm.handle_pagefault(get_memory_set, vpn, task_id);
                }
            };
        } else {
            panic!("Page fault but page fault handling is not enabled")
        }
    }

    pub fn new_memory_set(&mut self, task_id: usize) {
        if cfg!(feature = "pff") && task_id != GLOBAL_ID{
            return; 
        }

        self.managers.insert(task_id, FM::new());
    }

    pub fn clone_memory_set(&mut self, task_id: usize, parent_id: usize) {
        if cfg!(feature = "pff") {
            return;
        }
        
        let pmanager = self.managers.get(&parent_id).unwrap();
        self.managers.insert(task_id, pmanager.clone());
    }

    pub fn del_memory_set(&mut self, task_id: usize) {
        if cfg!(feature = "pff") {
            let manager = self.managers.get_mut(&GLOBAL_ID).unwrap();
            manager.clear_frames(task_id);
        } else {
            self.managers.remove(&task_id);
        }
    }
}