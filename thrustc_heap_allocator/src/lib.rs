use std::alloc::{GlobalAlloc, Layout, System};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static HEAP_LIMIT_BYTES: AtomicUsize = AtomicUsize::new(16usize * 1024 * 1024 * 1024); // 16 GB

pub struct ThrustCompilerHeapAllocator;

static SYSTEM: System = System;

fn abort(msg: &[u8]) -> ! {
    let stderr: std::io::Stderr = std::io::stderr();
    let mut handle: std::io::StderrLock<'_> = stderr.lock();

    let _ = handle.write_all(msg);
    let _ = handle.write_all(b"\n");
    let _ = handle.flush();

    std::process::exit(1);
}

unsafe impl GlobalAlloc for ThrustCompilerHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let alloc_size: usize = layout.size();
        let current_usage: usize = ALLOCATED_BYTES.load(Ordering::Relaxed);
        let limit: usize = HEAP_LIMIT_BYTES.load(Ordering::Relaxed);

        if limit > 0 && current_usage.saturating_add(alloc_size) > limit {
            abort("Compiler out of heap space! Compilation failed.".as_bytes());
        }

        let ptr: *mut u8 = unsafe { SYSTEM.alloc(layout) };

        if ptr.is_null() {
            abort("Heap memory allocation failed! Aborting. Compilation failed.".as_bytes());
        }

        ALLOCATED_BYTES.fetch_add(alloc_size, Ordering::Relaxed);

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let dealloc_size: usize = layout.size();

        unsafe { SYSTEM.dealloc(ptr, layout) };

        ALLOCATED_BYTES.fetch_sub(dealloc_size, Ordering::Relaxed);
    }
}
