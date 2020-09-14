# mp2c
Multi producer multi polling consumer

# What is mp2c?
MP2C is a data structure that enables multiple producers/publishers to send messages to multiple consumers/subscribers.
The async mp2c data structure is called a `Carousel`.

# What do you mean by messages?
A `message` in `mp2c` context is a vector of `u8`. It's up to the producers/ consumers to marshall and unmarshall these messages as they best see fit.

# Is mp2c thread safe?
Yes.

# How can multiple producers(threads) send messages to a mp2c Carousel?
Cloning a `mp2c::asynch::Carousel` creates a clone of the underlying `std::sync::Sender` and every invocation of `Carousel::put` will send messages to the consumers. 

# Does mp2c support async message pub?
`mp2c::asynch::Carousel` supports full async behavior. All messages put on the `Carousel` are asynchronously sent to the consumers.

# Is there a memory overhead?
Yes. In the spirit of don't communicate by sharing memory, share memory by communicating, all messages are cloned as many times as the count of `mp2c::asynch::Consumer`s.

# Multi producer multi consumer example

```rust
 use mp2c::asynch::{Carousel, Consumer};

 struct TestConsumer1;

 impl Consumer for TestConsumer1 {
   fn consume(&self, data: Vec<u8>) {
     let msg = String::from_utf8(data).unwrap();
     // do something with msg
   }
 }

 struct TestConsumer2;

 impl Consumer for TestConsumer2 {
  fn consume(&self, data: Vec<u8>) {
    let msg = String::from_utf8(data).unwrap();
    // do something with msg   
  }
 }

 let mut v: Vec<Box<dyn Consumer + Send + 'static>> = Vec::new();
 v.push(Box::new(TestConsumer1));
 v.push(Box::new(TestConsumer2));

 let c = Carousel::new(v);

 for _ in 1..10 {
   let cloned_c = c.clone();
   let t = std::thread::spawn(move || {
     cloned_c.put(String::from("test").into_bytes());
   });
   t.join().unwrap();
 }
```

# What's next?

## Message id
Add a message id to each message being put on the data carousel.

# Release history

## v0.1.1
Updated README with example.

## v0.1.0
Initial release.