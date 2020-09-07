use std::iter::Iterator;
use std::error::Error;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

/// `Event` is an enum that offers various type of events that will be 
/// handled by an mp2c carousel.
#[derive(Debug, Copy, Clone)]
pub enum Event {
  Message(Vec<u8>),
  Terminate,
}

/// `Store` is a basic circular buffer/ queue implementation backed by
/// a data vector and a vector of iterable readers. 
///
/// An iterable reader is a type that implements `Iterator` and
/// `Counter`.
struct Store {
  size: usize,
  data: Vec<Event>,
  tail: usize,
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
  rx: Receiver<Event>,
}

impl<T: Consumer> Counter for Poller<T> {
  fn count(&self) -> usize {
    self.count
  }
}

impl<T: Consumer> Poller<T> {
  fn start(&self) {
    //@TODO: implement me.
  }
}

/// `Carousel` is the core data structure that represents a multi-producer multi-polling consumer
/// async `Event` delivery system.
pub struct Carousel<T: Consumer> {
  store: Store,
  pollers: Vec<Poller<T>>,
  tx: Sender<Event>,
}

impl<T: Consumer> Carousel<T>{

  pub fn new(size: usize, consumers: Vec<T>) -> Self {
    let mut txs = Vec::<Sender<Event>>::new();
    let pollers: Vec<Poller<T>> = consumers.
        map(|consumer| {
          let (tx, rx) = channel::<Event>();

          txs.push(tx);

          let p = Poller{count: 0, consumer, rx};

          p.start();

          p
        }).
        collect();

    let store = Store {
      size,
      data: Vec::<Event>::with_capacity(size),
      tail: 0,
    };

    let (tx, rx) = channel::<Event>();

    let mut c = Carousel {
      store,
      pollers,
      tx,
    };

    thread::spawn(move|| {
      let e = rx.recv().unwrap();

      txs.for_each(|tx| {
        let ce = e;
        tx.send(ce).unwrap();
      });

      c.add(e);
    });

    c
  }

  /// Returns the head of this ring buffer.
  fn head(&self) -> usize {
    self.pollers.iter().map(|x| x.count()).min().unwrap_or(0)
  }

  /// Returns if the ring buffer is empty or not.
  fn is_empty(&self) -> bool {
    self.store.tail == self.head()
  }

  /// Returns if the ring buffer is full or not.
  fn is_full(&self) -> bool {
    (self.store.tail % self.store.size) + 1 == self.head()
  }

  fn add(&mut self, event: Event) -> Result<(), String> {
    if self.is_full() {
      return Err(String::from("waiting for readers to finish reading"));
    }
    self.store.data[self.store.tail] = event;
    self.store.tail = (self.store.tail + 1) % self.store.size;

    Ok(())
  }

  /// Queues an item on the carousel to be polled by the consumers.
  pub fn queue(&self, event: Event) -> Result<(), String> {
    return match self.tx.send(event) {
      Err(e) => Err(String::from(e.to_string())),
      _ => Ok(()),
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