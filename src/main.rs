#![feature(box_as_ptr)]

use epoll::events::Events;
use epoll::{Poll, PollEvent};
use std::mem;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

struct FileDescState {
    fd: i32,
    count: usize,
}

/// Accept a connection to a socket.
///
/// I'm doing syscalls directly instead of a TcpStream as a workaround, because using listener.accept
/// will give me ownership of TcpStream which gets dropped when the current iteration ends.
/// I will implement my own TcpStream for polling to avoid doing syscalls directly.
fn accept(fd: i32) -> Result<i32, ()> {
    let mut addr: libc::sockaddr_in = unsafe { mem::zeroed() };
    let mut addr_len: libc::socklen_t = mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
    let fd = unsafe {
        libc::accept4(
            fd,
            &mut addr as *mut libc::sockaddr_in as *mut libc::sockaddr,
            &mut addr_len,
            libc::SOCK_NONBLOCK,
        )
    };
    if fd == -1 {
        Err(())
    } else {
        Ok(fd)
    }
}

fn main() -> Result<(), ()> {
    let mut events = Events::with_capacity(50000);
    let poll = Poll::default();

    let listener = TcpListener::bind("0.0.0.0:5002").unwrap();
    listener.set_nonblocking(true).map_err(|_| ())?;

    let listener_state = Box::new(FileDescState {
        fd: listener.as_raw_fd(),
        count: 0,
    });
    let _ = poll.register(
        listener.as_raw_fd() as u64,
        Box::as_ptr(&listener_state) as u64,
    );

    loop {
        //println!("polling..");
        poll.poll(&mut events);
        for event in &mut events.events {
            let state = event.u64 as *mut FileDescState;
            let state = unsafe { state.as_mut().expect("State should be allocated already.") };
            println!("fd({}) = {}", state.fd, state.count);
            if state.fd == listener.as_raw_fd() {
                // Connections are available.
                println!("New connection.");
                while let Ok(fd) = accept(listener.as_raw_fd()) {
                    let fd_state = Box::new(FileDescState { fd, count: 0 });
                    // The state should live until the socket is closed.
                    let fd_state = Box::leak(fd_state);
                    let _ = poll.register(fd as u64, fd_state as *const FileDescState as u64);
                }
            } else if event.is_closed() {
                // Socket has closed
                println!("Connection closed..");
                let _ = poll.remove(state.fd as u64, event);
                // free the allocated state since the socket has closed.
                unsafe {
                    drop(Box::from_raw(state));
                }
            } else if event.is_readable() {
                // Socket is readable
                process_fd(state.fd).expect("Should be available");
                state.count += 1;
            } else {
                // Something else
                println!("Unexpected");
            }
        }
        //std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Read from a connection.
///
/// Note: This is just a quick implementation.
fn process_fd(fd: i32) -> Result<(), ()> {
    let data: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n\r\n";
    let buf: [u8; 1024] = [0; 1024];
    let size = unsafe {
        let s = libc::recv(fd, buf.as_ptr() as *mut libc::c_void, buf.len(), 0);
        libc::send(fd, data.as_ptr() as *const libc::c_void, data.len(), 0);
        s
    };
    unsafe {
        libc::close(fd);
    }
    if size == 0 {
        Err(())
    } else {
        Ok(())
    }
}
