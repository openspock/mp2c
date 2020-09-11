# mp2c
Multi producer multi polling consumer

_Note: work in progress_

# What is mp2c?
MP2C is a data structure that enables multiple producers/publishers to send messages to multiple consumers/subscribers/readers/receivers.

# What do you mean by messages?
A `message` in `mp2c` context is a vector of `u8`. It's upto the producers/ consumers to marshall and unmarshall these messages as they best see fit.

# Is mp2c thread safe?
Yes.

# Does mp2c support async message pub?
`mp2c::asynch::Carousel` supports full async behavior. All messages put on the `Carousel` are asynchronously sent to the consumers.

# Is there a memory overhead?
Yes. In the spirit of don't communicate by sharing memory, share memory by communicating, all messages are cloned as many times as the count of `mp2c::asynch::Consumer`s.
