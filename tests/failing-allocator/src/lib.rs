// SPDX-License-Identifier: AGPL-3.0-or-later
//! Thread-local real allocation denial for the external-check test binary.
//! Safe code cannot implement `GlobalAlloc`; all unsafe operations below only
//! forward the allocator contract unchanged to `System`.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr;

thread_local! {
    static REMAINING: Cell<Option<usize>> = const { Cell::new(None) };
    static FAILED_BYTES: Cell<Option<usize>> = const { Cell::new(None) };
}

/// System allocator with a thread-local, one-shot denial.
pub struct FailingAllocator;

fn deny(size: usize) -> bool {
    REMAINING.with(|remaining| match remaining.get() {
        Some(0) => {
            remaining.set(None);
            FAILED_BYTES.with(|failed| failed.set(Some(size)));
            true
        }
        Some(count) => {
            remaining.set(Some(count - 1));
            false
        }
        None => false,
    })
}

// SAFETY: Successful allocations and every deallocation are delegated to System
// with unchanged pointers/layouts. Returning null is an allowed allocation
// failure. Thread-local Cell state neither allocates nor recursively calls us.
unsafe impl GlobalAlloc for FailingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if deny(layout.size()) {
            ptr::null_mut()
        } else {
            // SAFETY: The caller supplied the GlobalAlloc layout contract.
            unsafe { System.alloc(layout) }
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if deny(layout.size()) {
            ptr::null_mut()
        } else {
            // SAFETY: The caller supplied the GlobalAlloc layout contract.
            unsafe { System.alloc_zeroed(layout) }
        }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if deny(size) {
            ptr::null_mut()
        } else {
            // SAFETY: The caller supplied a live System allocation and valid
            // new size; denial leaves that allocation untouched.
            unsafe { System.realloc(pointer, layout, size) }
        }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: All successful allocations came from System, and the caller
        // supplies the matching live pointer and original layout.
        unsafe { System.dealloc(pointer, layout) }
    }
}

/// Deny the allocation after `successful` successful requests on this thread.
pub fn arm(successful: usize) {
    FAILED_BYTES.with(|failed| failed.set(None));
    REMAINING.with(|remaining| remaining.set(Some(successful)));
}

/// Restore allocation and return the real denied layout size, if reached.
pub fn disarm() -> Option<usize> {
    REMAINING.with(|remaining| remaining.set(None));
    FAILED_BYTES.with(Cell::take)
}
