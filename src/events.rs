#![allow(clippy::uninit_vec)]

#[derive(Default)]
pub struct Events {
    /// The number of file descriptor that have events available.
    /// This value changes with every `poll` call
    fds: u64,
    /// The actual events buffer passed to the kernel.
    pub events: Vec<libc::epoll_event>,
}

impl Events {
    /// Set the number of max events a single `poll` call can produce.
    ///
    /// This is `maxevents` for `epoll_wait(2)`
    pub fn with_capacity(size: usize) -> Self {
        assert!(size != 0, "Size can't be zero");
        Events {
            fds: 0,
            events: Vec::with_capacity(size),
        }
    }

    pub fn capacity(&self) -> i32 {
        self.events.capacity() as i32
    }

    pub fn set_len(&mut self, len: i32) {
        unsafe {
            self.events.set_len(len as usize);
        };
    }
}
