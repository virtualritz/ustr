use bumpalo::Bump;

// Wrapper around bumpalo::Bump to provide the same interface as the original
// LeakyBumpAlloc
pub(crate) struct LeakyBumpAlloc {
    bump: Bump,
    capacity: usize,
    allocated: usize,
}

impl LeakyBumpAlloc {
    pub fn new(capacity: usize, _alignment: usize) -> LeakyBumpAlloc {
        // Create bump allocator. We don't limit bumpalo's capacity since it
        // grows automatically, but we track our logical capacity for
        // compatibility with StringCache logic
        let bump = Bump::new();

        LeakyBumpAlloc {
            bump,
            capacity,
            allocated: 0,
        }
    }

    #[doc(hidden)]
    // used for resetting the cache between benchmark runs. DO NOT CALL THIS.
    pub unsafe fn clear(&mut self) {
        // Reset the bump allocator by replacing it with a new one
        self.bump = Bump::new();

        // Reset tracking
        self.allocated = 0;
    }

    // Allocates a new chunk. Aborts if out of memory.
    pub unsafe fn allocate(&mut self, num_bytes: usize) -> *mut u8 {
        // Use bumpalo's allocation with proper alignment for StringCacheEntry
        let layout = std::alloc::Layout::from_size_align(
            num_bytes,
            std::mem::align_of::<crate::StringCacheEntry>(),
        )
        .unwrap();

        // Try to allocate. bumpalo will handle growing automatically
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.bump.alloc_layout(layout)
            }));

        match result {
            Ok(ptr) => {
                let result = ptr.as_ptr();

                // Update our tracking
                self.allocated += num_bytes;
                result
            }
            Err(_) => {
                eprintln!("Allocator failed to allocate {num_bytes} bytes");
                // We have to abort here rather than panic or the mutex may
                // deadlock.
                std::process::abort();
            }
        }
    }

    pub fn allocated(&self) -> usize {
        // Return our manual tracking rather than bumpalo's since we want to
        // preserve the original behavior for StringCache capacity
        // checks
        self.allocated
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
