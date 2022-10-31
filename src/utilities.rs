use alloc_cortex_m::CortexMHeap;

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    rtt_target::rprintln!("Allocation error");
    loop {}
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

/// Initialize the allocator BEFORE you use it.
pub fn init_mem_allocator() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
}
