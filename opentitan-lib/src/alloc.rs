//! Adds support for alloc by providing an allocator

use core::{
    alloc::GlobalAlloc,
    cell::RefCell,
    ptr::{self, NonNull},
};

use linked_list_allocator::Heap;

/// CustomHeap implementation handling the allocations on the heap
#[global_allocator]
pub(crate) static ALLOCATOR: CustomHeap = CustomHeap::empty();

/// Since the architecture is assumed to be on a single core and without atomic instructions
/// the GlobalAlloc Trait has to be manually implemented for Heap, therefore we define this
/// Wrapper type
pub(crate) struct CustomHeap(RefCell<Heap>);

impl CustomHeap {
    const fn empty() -> CustomHeap {
        CustomHeap(RefCell::new(Heap::empty()))
    }

    pub(crate) unsafe fn init(&self, heap_bottom: *mut u8, heap_size: usize) {
        self.0.borrow_mut().init(heap_bottom, heap_size)
    }
}

unsafe impl Sync for CustomHeap {}

unsafe impl GlobalAlloc for CustomHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        riscv::interrupt::disable();
        let x = self
            .0
            .borrow_mut()
            .allocate_first_fit(layout)
            .ok()
            .map_or(ptr::null_mut::<u8>(), |addr| addr.as_ptr());
        riscv::interrupt::disable();
        x
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        riscv::interrupt::disable();
        let x = self
            .0
            .borrow_mut()
            .deallocate(NonNull::new_unchecked(ptr), layout);
        riscv::interrupt::disable();
        x
    }
}
