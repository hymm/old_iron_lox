use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    process::exit,
    ptr::null_mut,
};

// TODO: none of this code is dealing with padding and alignment correctly.

#[inline(always)]
pub const fn grow_capacity(capacity: usize) -> usize {
    if capacity < 8 {
        8
    } else {
        capacity * 2
    }
}

pub unsafe fn grow_array<T>(pointer: *mut T, old_count: usize, new_count: usize) -> *mut T {
    let size_of_t = size_of::<T>();
    reallocate(
        pointer as *mut u8,
        size_of_t * old_count,
        size_of_t * new_count,
    ) as *mut T
}

/// Safety:
/// - ptr is a block of memory currently allocated via this allocator.
/// - layout is the same layout that was used to allocate this block of memory.
unsafe fn reallocate(pointer: *mut u8, _old_size: usize, new_size: usize) -> *mut u8 {
    if new_size == 0 {
        // Safety:
        // - safety of dealloc is ensured by the caller
        unsafe { dealloc(pointer, Layout::new::<u8>()) };
        return null_mut();
    }

    let result = if pointer.is_null() {
        unsafe { alloc(Layout::array::<u8>(new_size).expect("array is too large")) }
    } else {
        // Safety
        // The caller must ensure that:
        // - Safety of allocator and layout ensured by the caller.
        // - new_size is greater than zero is checked above.
        // TODO: add a check for the next safety comment
        // - new_size, when rounded up to the nearest multiple of layout.align(), does not overflow isize (i.e., the rounded value must be less than or equal to isize::MAX).
        unsafe { realloc(pointer, Layout::new::<u8>(), new_size) }
    };

    if result.is_null() {
        dbg!("couldn't realloc");
        exit(1);
    }

    result
}

/// Safety:
/// - ptr is a block of memory currently allocated via this allocator.
/// - layout is the same layout that was used to allocate this block of memory.
pub unsafe fn free_array<T>(pointer: *mut T, old_count: usize) {
    reallocate(pointer as *mut u8, size_of::<T>() * old_count, 0);
}
