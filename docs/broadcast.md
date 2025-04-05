# Broadcast channels  In Async Chat Server 

## Introduction

  Broadcasting is a means of communication whereby information is transferred from a single producer to multiple consumers or receivers. Here , Broadcast channels are the channels through which broadcasting occurs.


## Why broadcasting 

- ### Point to Point

   Although point-to-point communication allows multiple users to send messages, it only supports a single receiver. On the other hand, broadcasting works with SPMC (single producer, multiple consumers) channels, allowing multiple consumers to receive messages simultaneously

- ### Unicast 

  Here, the messages are sent from one person to another. This is not very efficient because if someone wants to send the same message to 10 people, they would need to send the message one by one, which is not ideal. Broadcasting would be preferred here, as it would make message sending easier and more efficient.

- ### Multicast
   Multicasting is very difficult and complex to built than broadcasting .. Even though both send messages from one producer to multiple consumers, there are many aspects to handle when using multicasting that introduce complexity, while broadcasting is simpler and less complex.

## Advantages 

 Some of the advantages of broadcasting are 

 - ### Efficiency 
   A broadcast channel is not only simple to implement but also highly efficient. It enables the transfer of messages to multiple receivers simultaneously, making it ideal for chat servers with a large number of users.

- ### Data Isolation
  When using a broadcast channel, each user receives their own copy of the data, preventing shared access that could lead to data races.

- ### Concurrency Managemen
  Even though broadcast channels work with MPSC (Multiple Producer, Single Consumer), they allow multiple users to send messages by cloning the user. This reduces potential issues that may arise if two users attempt to send messages simultaneously.

- ### Offline Message Retrieval
  Using broadcasting allows the chat server to store built-in messages, enabling users to retrieve them if they were offline.


## Conclusion 
In conclusion, using a broadcasting channel is certainly the best choice for this chat server, as it provides efficiency, fast message retrieval, and safety.