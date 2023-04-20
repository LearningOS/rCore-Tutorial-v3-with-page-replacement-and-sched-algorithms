/**
 * 这里使用内存的部分空间来存放应当被置换出的页面
 * 实际情况下，被置换出的页面会被放于磁盘中
 */

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::config::{MAX_PAGES, IDE_SIZE, PAGE_SIZE};

struct IDE {
    pub data: [u8; IDE_SIZE],
}

impl IDE {
    pub fn ide_valid(idx: usize) -> bool {
        idx < MAX_PAGES
    }

    pub fn ide_read(&mut self, idx: usize, dst: &mut [u8]) -> usize {
        if !Self::ide_valid(idx) {
            return 1;
        }
        let base = idx * PAGE_SIZE;
        let ide_ptr = &self.data[base..(base+PAGE_SIZE)];
        dst.copy_from_slice(ide_ptr);
        0
    }

    pub fn ide_write(&mut self, idx: usize, src: &[u8]) -> usize {
        if !Self::ide_valid(idx) {
            return 1;
        }
        let base = idx * PAGE_SIZE;
        let ide_ptr = &mut self.data[base..(base+PAGE_SIZE)];
        ide_ptr.copy_from_slice(src);
        0
    }
}

static mut IDE: IDE = IDE {
    data: [0; IDE_SIZE],
};


pub struct IdeManager {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
    map: BTreeMap<(usize, usize), usize>,
}

impl IdeManager {
    pub const fn new() -> Self {
        Self {
            current: 0,
            end: MAX_PAGES - 1,
            recycled: Vec::new(),
            map: BTreeMap::new(),
        }
    }
    pub fn swap_in(&mut self, token: usize, vpn: usize, src: &mut [u8]) {
        if let Some(idx) = self.recycled.pop() {
            unsafe { IDE.ide_write(idx, src); }
            self.map.insert((token, vpn), idx);
        } else if self.current == self.end {
            panic!("[kernel] Virtual disk space is not enough for handling page fault.");
        } else {
            self.current += 1;
            unsafe { IDE.ide_write(self.current - 1, src); }
            self.map.insert((token, vpn), self.current - 1);
        }
    }
    pub fn swap_out(&mut self, token: usize, vpn: usize, dst: &mut [u8]) {
        let idx = self.map.get(&(token, vpn)).unwrap();
        unsafe { IDE.ide_read(*idx, dst); }
        self.recycled.push(*idx);
        self.map.remove(&(token, vpn));
    }
    pub fn check(&mut self, token: usize, vpn: usize) -> bool {
        self.map.get(&(token, vpn)) != None
    }
}

pub static mut IDE_MANAGER: IdeManager = IdeManager::new();