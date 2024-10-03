use epoll::events::Events;
use epoll::{Poll, PollEvent};
use std::mem;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

/// Accept a connection to a socket.
///
/// I'm doing syscalls directly instead of a TcpStream as a workaround, because using listener.accept
/// will give me ownership of TcpStream which gets dropped when the current iteration ends.
/// I will implement my own TcpStream for polling to avoid doing syscalls directly.
fn accept(fd: i32) -> i32 {
    let mut addr: libc::sockaddr_in = unsafe { mem::zeroed() };
    let mut addr_len: libc::socklen_t = mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
    unsafe {
        libc::accept(
            fd,
            &mut addr as *mut libc::sockaddr_in as *mut libc::sockaddr,
            &mut addr_len,
        )
    }
}
fn main() -> Result<(), ()> {
    let mut events = Events::with_capacity(15);
    let poll = Poll::default();

    let listener = TcpListener::bind("0.0.0.0:5002").unwrap();
    listener.set_nonblocking(true).map_err(|_| ())?;

    let _ = poll.register(listener.as_raw_fd() as u64);

    loop {
        println!("polling..");
        poll.poll(&mut events);
        for event in &mut events.events {
            if event.as_raw_fd() == listener.as_raw_fd() {
                // Connections are available.
                // TODO: should register all available connections, not just the first one
                println!("New connection.");
                let fd = accept(listener.as_raw_fd());
                let _ = poll.register(fd as u64);
            } else if event.is_closed() {
                // Socket has closed
                println!("Connection closed..");
                let _ = poll.remove(event);
            } else if event.is_readable() {
                // Socket is readable
                process_fd(event.u64 as i32).expect("Should be available");
            } else {
                // Something else
                println!("Unexpected");
            }
        }
    }
}

/// Read from a connection.
///
/// Note: This is just a quick implementation.
fn process_fd(fd: i32) -> Result<(), ()> {
    println!("recieved bytes!");
    let data: &[u8] = b"hello";
    let buf: [u8; 1024] = [0; 1024];
    let size = unsafe {
        let s = libc::read(fd, buf.as_ptr() as *mut libc::c_void, buf.len());
        libc::write(fd, data.as_ptr() as *const libc::c_void, data.len());
        s
    };

    if size == 0 {
        Err(())
    } else {
        Ok(())
    }
}
