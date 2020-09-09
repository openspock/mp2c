# mp2c
Multi producer multi polling consumer

_Note: work in progress_

# What is mp2c?
MP2C is a data structure that enables multiple producers/publishers/writers/senders to send messages to multiple consumers/subscribers/readers/receivers.

# What do you mean by messages?
A `message` in `mp2c` context is a simple `base64` encoded string. It's upto the Rx/Tx to marshall and unmarshall these messages as they best see fit.

# Is mp2c thread safe?
Hekk, yeah! If you observe it to not be threadsafe please open a bug.
