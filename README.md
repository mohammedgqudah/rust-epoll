# A safe high-level API for epoll(2)
This is one of my learning projects.
Once the API is stable I will use it in my [rust-http-server](https://github.com/mohammedgqudah/rust-http-server) project.

# Example

```rust
use epoll::events::Events;
use epoll::{Poll, PollEvent};


let mut events = Events::with_capacity(15); // epoll maxsize
let poll = Poll::default();

let listener = ...

poll.register(listener.as_raw_fd() as u64)?;

loop {
    println!("polling..");
    poll.poll(&mut events);

    for event in &mut events.events {
        if event.as_raw_fd() == listener.as_raw_fd() {
            // Connections are available.
            let fd = accept(listener.as_raw_fd());
            poll.register(fd);
        } else if event.is_closed() {
            // socket has closed
            println!("Connection closed..");
            poll.remove(event);
        } else if event.is_readable() {
            // socket is readable
            process_socket(event.as_raw_fd())
        } else {
            // Something else
        }
    }
}
```
