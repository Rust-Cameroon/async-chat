# Overview of block_on function
## What's block_on
The block_on function is an synchronous function that produces a final value of an asynchronous function, you can think of it as an adapter from the asynchronous world to the synchronous world. The block_on function is part of the tokio or async-std crates, not part of the standard library.

## Why block_on 
In a sense, asynchronous function just pass the buck, this buck is simply due to the fact that, when executing a synchronous function: the caller only resumes when the operation is completed, what if we want our thread to do something else while the operating system does it work, we will need to use new I/O library that provides an asynchronous version of this function, Rust approach of supporting asynchronous operation is by introducing a trait, std::future::Future. A Future represent an operation you can test for completion. So with Future, you can use the current thread and do some other job, but using futures seems challenging to use because, you keep on polling other jobs when a future is still pending, keeping track of previous futures who's pending and what should be done once its finish, somehow ruin the simplicity of the asynchronous function. Well this buck is solve using .await expression.  it's true that it easy to get the value of an async function: just await it. But async function itself return a future, so it's now the caller's job to do the polling somehow, thus someone has to wait for value. 
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

We can call the function cheapo_request from an ordinary, synchronous function(like the main, for example), using the async_std's tasks::block_on function, which takes a future and poll it until it return a value as seen above.

So in summary, the block_on function in used to execute asynchronous block synchronousely in rust.


