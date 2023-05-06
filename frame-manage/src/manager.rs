use core::marker::PhantomData;
use alloc::collections::BTreeMap;
use kernel_vm::{PageManager, AddressSpace, VmMeta, VAddr, VmFlags};
use crate::plugins::Manage;
use crate::config::GLOBAL_ID;

pub struct PageFaultHandler<Meta: VmMeta, PM: PageManager<Meta> + 'static, FM: Manage<Meta, PM>, Func: Fn(usize) -> &'static mut AddressSpace<Meta, PM>> {
    enable_pagefault: bool,
    managers: BTreeMap<usize, FM>,
    func_get_memory_set: Option<Func>,

    dummy1: PhantomData<Meta>,
    dummy2: PhantomData<PM>
}

impl<Meta: VmMeta, PM: PageManager<Meta> + 'static, FM: Manage<Meta, PM> + Clone, Func: Fn(usize) -> &'static mut AddressSpace<Meta, PM>> 
    PageFaultHandler<Meta, PM, FM, Func> {
    pub const fn new() -> Self {
        if cfg!(feature = "none") {
            Self { enable_pagefault: false, managers: BTreeMap::new(), func_get_memory_set: None, dummy1: PhantomData::<Meta>, dummy2: PhantomData::<PM> }
        } else {
            Self { enable_pagefault: true, managers: BTreeMap::new(), func_get_memory_set: None, dummy1: PhantomData::<Meta>, dummy2: PhantomData::<PM> }
        }
    }

    pub fn set_func(&mut self, func: Func) {
        self.func_get_memory_set = Some(func);
    }

    pub fn handle_pagefault(&mut self, addr: usize, flag: usize, task_id: usize) {
        let get_memory_set = self.func_get_memory_set.as_ref().unwrap();
        if self.enable_pagefault {
            // check if the addr is already mapped to memory set
            let vaddr: VAddr<Meta> = VAddr::from(addr);
            if let Some(pte) = get_memory_set(task_id).translate_to_pte(vaddr) {
                if pte.is_valid() && !pte.flags().contains(unsafe { VmFlags::from_raw(flag) }) {
                    panic!("[PAGE FAULT]: unsupported flags, require={}, orig={}", flag, pte.flags().val());
                }
            }

            // handle page fault
            let _id = if cfg!(feature = "pff") || cfg!(feature = "work-set") { GLOBAL_ID } else { task_id };
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
        if (cfg!(feature = "pff") || cfg!(feature = "work-set") )&& task_id != GLOBAL_ID{
            return; 
        }

        self.managers.insert(task_id, FM::new());
    }

    pub fn clone_memory_set(&mut self, task_id: usize, parent_id: usize) {
        if cfg!(feature = "pff") || cfg!(feature = "work-set"){
            return;
        }
        
        let pmanager = self.managers.get(&parent_id).unwrap();
        self.managers.insert(task_id, pmanager.clone());
    }

    pub fn del_memory_set(&mut self, task_id: usize) {
        if cfg!(feature = "pff") || cfg!(feature = "work-set") {
            let manager = self.managers.get_mut(&GLOBAL_ID).unwrap();
            manager.clear_frames(task_id);
        } else {
            self.managers.remove(&task_id);
        }
    }

    pub fn time_interrupt_hook(&mut self) {
        if cfg!(feature = "pff") || cfg!(feature = "work-set") {
            let manager = self.managers.get_mut(&GLOBAL_ID).unwrap();
            manager.handle_time_interrupt(self.func_get_memory_set.as_ref().unwrap());
        }
    }
}