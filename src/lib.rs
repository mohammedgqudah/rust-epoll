#![allow(clippy::all)]
pub mod events;

use events::Events;

pub struct Poll {
    fd: i32,
}

impl Poll {
    /// Register a file descriptor in the interest list.
    pub fn register(&self, fd: u64) -> Result<(), ()> {
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | libc::EPOLLHUP | libc::EPOLLRDHUP) as u32,
            u64: fd,
        };

        let result = unsafe {
            libc::epoll_ctl(
                self.fd,
                libc::EPOLL_CTL_ADD,
                i32::try_from(fd).unwrap(),
                std::ptr::from_mut(&mut event),
            )
        };
        if result == -1 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Remove a file descriptor from the interest list.
    pub fn remove(&self, event: &mut libc::epoll_event) -> Result<(), ()> {
        let result = unsafe {
            libc::epoll_ctl(
                self.fd,
                libc::EPOLL_CTL_DEL,
                i32::try_from(event.u64).unwrap(),
                std::ptr::from_mut(event),
            )
        };
        // TODO: Build a macro for all libc functions that could return -1, and wrap them in a
        // Result
        if result == -1 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn poll(&self, events: &mut Events) {
        let fds =
            unsafe { libc::epoll_wait(self.fd, events.events.as_mut_ptr(), events.capacity(), -1) };
        events.set_len(fds);
    }
}

impl Default for Poll {
    fn default() -> Self {
        let fd = unsafe { libc::epoll_create1(0) };
        Poll { fd }
    }
}

pub trait PollEvent {
    fn is_readable(&self) -> bool;
    fn is_closed(&self) -> bool;
    fn as_raw_fd(&self) -> i32;
}

impl PollEvent for libc::epoll_event {
    fn as_raw_fd(&self) -> i32 {
        self.u64 as i32
    }
    fn is_readable(&self) -> bool {
        self.events as i32 & libc::EPOLLIN != 0
    }
    fn is_closed(&self) -> bool {
        (self.events as i32) & (libc::EPOLLHUP | libc::EPOLLRDHUP) != 0
    }
}
