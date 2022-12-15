# Note
## run
```shell
make docker
# in docker
make run
```

## task-manage
各个 trait 的接口可以参考 [task-manage readme](task-manage/README.md)

`schedule trait` 只使用任务`id`进行调度处理，`manage trait`是任务池的实现。

`Process`控制块的部分内容在[proc_rel.rs](task-manage/src/proc_rel.rs)(只有进程时)或者[proc_thread_rel.rs](task-manage/src/proc_thread_rel.rs)（有线程时）里。

通过修改[ch8/Cargo.toml](ch8/Cargo.toml)中的 feature 设置可以改变线程的启用：
```
rcore-task-manage = { path = "../task-manage", features = ["thread"]  }
```

[ch8/processor.rs](ch8/src/processor.rs)是带线程的调度器的实现。

给`manage trait`加回调 hook，复杂的调度数据结构维护在`schedule trait`中。可以在`feature`里尝试加入调度算法的 config 项。

## kernel-vm
页表相关处理内容，`PageManager trait` 在 [ch8/src/main.rs](ch8/src/main.rs) 中的`Sv39Manager`实现了。
`Sv39Manager`的`deallocate`没实现。

## fs相关
在`easy-fs/`和[ch8/src/fs.rs](ch8/src/fs.rs)中。
关于页面置换的部分在[easy-fs/src/block_cache.rs](easy-fs/src/block_cache.rs)中，其中的`BlockCacheManager`是当前适用的页面置换算法的示例。在`BlockCacheManager`的`get_block_cache`函数中实现了置换的控制。

目前的实现：
```rust
// substitute
if self.queue.len() == BLOCK_CACHE_SIZE {
    // from front to tail
    if let Some((idx, _)) = self
        .queue
        .iter()
        .enumerate()
        .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)  // drop 了没有 proc 引用的 block
    {
        self.queue.drain(idx..=idx);
    } else {
        panic!("Run out of BlockCache!");
    }
}
```

## syscall 相关
所有的`syscall`的具体实现在[ch8/src/main.rs](ch8/src/main.rs)中可以找到。
`yield`的 syscall 没有实现(line 502)。
`syscall id`匹配的部分在[syscall/src/kernel/mod.rs](syscall/src/kernel/mod.rs)的`handle`函数里。`id`常量写在了[syscall/src/syscall.h.in](syscall/src/syscall.h.in)中，编译时根据此文件生成[syscall/src/syscalls.rs](syscall/src/syscalls.rs)。

## update
### problems
1. 这个 `xtask/src/fs_pack.rs` 跑的好慢 ... 