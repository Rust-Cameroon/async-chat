# Overview of block_on function

## What is block_on

The block_on function is a synchronous function that produces a final value of an asynchronous function, 
you can think of it as an adapter from the asynchronous world to the synchronous world. The block_on 
function is part of the tokio or async-std crates, not part of the standard library.

## Why block_on 

In a sense, asynchronous functions just pass the buck. This buck is simply due to the fact that when executing a synchronous function, the caller only resumes when the operation is completed. What if we want our thread to do something else while the operating system is doing its work? We will need to use a new I/O library that provides an asynchronous version of this function. Rust's approach to supporting asynchronous operations is by introducing a trait: std::future::Future.

A Future represents an operation you can test for completion. So with Future, you can always know the state of the current thread in order to do other jobs, but using futures seems challenging because you keep on polling other jobs while a future is still pending, keeping track of previous futures that are pending and what should be done once they are finished and poll it again, and this somehow ruins the simplicity of the function. Good news: asynchronous functions are there! This buck is solved using the .await expression which pauses the execution of this async function until the awaited value is ready before resuming its execution. It's true that it's easy to get the value of an async function: just await it. But async functions themselves return a future, so it's now the caller's job to do the polling somehow, thus someone has to wait for the value and in this case block_on is our waiter. 

Consider the example below:
```sh
use async_std::io::prelude::*;
use async_std::net;
async fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String> {
    let mut socket = net::TcpStream::connect((host, port)).await?;
    let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
    socket.write_all(request.as_bytes()).await?;
    socket.shutdown(net::Shutdown::Write)?;
    let mut response = String::new();
    socket.read_to_string(&mut response).await?;
    Ok(response)
}
```

```sh
fn main() -> std::io::Result<()> {
    use async_std::task;
        // `block_on` is used here to run an async function cheapo_request in a synchronous context.
    let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
    println!("{}", response);
    Ok(())
}
```

We can call the function cheapo_request from an ordinary, synchronous function (like main, for example), using async_std's task::block_on function, which takes a future and polls it until it returns a value as seen above.

So in summary, the block_on function is used to execute asynchronous blocks synchronously in Rust.
