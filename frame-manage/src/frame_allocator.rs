use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct FrameTracker {
    pub ppn: usize
}

impl FrameTracker {
    pub fn new(ppn: usize) -> Self {
        // // page cleaning
        // let bytes_array = ppn.get_bytes_array();
        // for i in bytes_array {
        //     *i = 0;
        // }
        Self { ppn: ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn alloc(&mut self) -> Option<usize>;
    fn dealloc(&mut self, ppn: usize);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>
}

impl StackFrameAllocator {
    pub const fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new()
        }
    }
    pub fn init(&mut self, l: usize, r: usize) {
        self.current = l;
        self.end = r;
    }
    fn check(&mut self) -> bool {
        if self.current < self.end {
            return true;
        }
        self.recycled.len() != 0
    }
}
impl FrameAllocator for StackFrameAllocator {
    fn alloc(&mut self) -> Option<usize> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn)
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some(self.current - 1)
        }
    }
    fn dealloc(&mut self, ppn: usize) {
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

pub static mut FRAME_ALLOCATOR: StackFrameAllocator = StackFrameAllocator::new();

pub fn frame_alloc() -> Option<FrameTracker> {
    unsafe { FRAME_ALLOCATOR.alloc().map(FrameTracker::new) }
}

pub fn frame_dealloc(ppn: usize) {
    unsafe { FRAME_ALLOCATOR.dealloc(ppn); }
}

pub fn frame_check() -> bool {
    unsafe { FRAME_ALLOCATOR.check() }
}
