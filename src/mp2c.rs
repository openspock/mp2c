use std::iter::Iterator;
use std::error::Error;

/// `Event` is an enum that offers various type of events that will be 
/// handled by an mp2c carousel.
pub enum Event {
  Message(Vec<u8>),
  Terminate,
}

/// `Store` is a basic circular buffer/ queue implementation backed by
/// a data vector and a vector of iterable readers. 
///
/// An iterable reader is a type that implements `Iterator` and
/// `Counter`.
struct Store<T: Counter + Iterator> {
  size: usize,
  data: Vec<Event>,
  tail: usize,
  pollers: Vec<T>,
}

impl<T: Counter + Iterator> Store<T>{

  fn new(size: usize, pollers: Vec<T>) -> Self {
    Store {
      size,
      data: Vec::with_capacity(size),
      tail: 0,
      pollers
    }
  }

  /// Returns the head of this ring buffer.
  fn head(&self) -> usize {
    self.pollers.iter().map(|x| x.count()).min().unwrap_or(0)
  }

  /// Returns if the ring buffer is empty or not.
  fn is_empty(&self) -> bool {
    self.tail == self.head()
  }

  /// Returns if the ring buffer is full or not.
  fn is_full(&self) -> bool {
    (self.tail % self.size) + 1 == self.head()
  }

  fn add(&mut self, event: Event) -> Result<(), String> {
    if self.is_full() {
      return Err(String::from("waiting for readers to finish reading"));
    }
    self.data[self.tail] = event;
    self.tail = (self.tail + 1) % self.size;

    Ok(())
  }
}

/// `Counter` indicates the index till where a consumer has polled
/// the Store
trait Counter {
  fn count(&self) -> usize;
}

/// `Consumer` enables to implement handling logic for a vector of bytes.
pub trait Consumer {
  fn consume(&self, Event);
}

/// `Poller` is a simple struct that encapsulates a polling thread that calls
/// the encapsulating `Consumer` for each `Event`.
///
/// counts the number of `Event`s processed by each poller.
struct Poller<T: Consumer> {
  count: usize,
  consumer: T,
}

impl<T: Consumer> Counter for Poller<T> {
  fn count(&self) -> usize {
    self.count
  }
}

impl<T: Consumer> Poller<T> {
  fn start(&self) {

  }
}

pub struct Carousel {
}

impl Carousel {
  pub fn new() -> Self {
    Carousel {
    }
  }
}

#[cfg(test)]
mod tests {

  struct TestConsumer {

  }

  impl Iterator for TestPoller {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
      unimplemented!()
    }
  }

  #[test]
  fn basic() {
    let 
  }
}