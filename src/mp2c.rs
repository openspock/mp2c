use std::iter::Iterator;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

/// `Event` is an enum that offers various type of events that will be 
/// handled by an mp2c carousel.
#[derive(Debug, Clone)]
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
  fn consume(&self, data: Vec<u8>);
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
  fn start(&mut self) {
    //@TODO: implement me.

    loop {
      let event = self.rx.recv().unwrap();

      match event {
        Event::Message(data) => {
          self.consumer.consume(data);
          self.count += 1;
        },
        _ => (),
      }
    }
  }
}

/// `Carousel` is the core data structure that represents a multi-producer multi-polling consumer
/// async `Event` delivery system.
pub struct Carousel<T: Consumer> {
  store: Store,
  pollers: Vec<Poller<T>>,
  tx: Sender<Event>,
  rx: Receiver<Event>,
  consumer_txs: Vec<Sender<Event>>
}

impl<T: Consumer + Send + 'static> Carousel<T>{

  pub fn new(size: usize, consumers: Vec<T>) -> Self {
    let mut consumer_txs = Vec::<Sender<Event>>::new();

    let pollers: Vec<Poller<T>> = consumers.into_iter().
        map(|consumer| {
          let (tx, rx) = channel::<Event>();

          consumer_txs.push(tx);

          let mut p = Poller{count: 0, consumer, rx};

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
      rx,
      consumer_txs,
    };

    c.start();

    c
  }

  fn start(&mut self) {
    thread::spawn(|| {
      loop {
        let e = self.rx.recv().unwrap();

        self.add(e);

        self.consumer_txs.into_iter().for_each(|tx| {
          let ce = e.clone();
          tx.send(ce).unwrap();
        });
      }
    });
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
  use crate::mp2c::{Consumer, Carousel, Event};

  struct TestConsumer {

  }

  impl Consumer for TestConsumer {
    fn consume(&self, data: Vec<u8>) {
      println!("{}",String::from_utf8(data).unwrap());
    }
  }

  #[test]
  fn basic() {
    let c = Carousel::new(5, vec![TestConsumer{}]);

    c.queue(Event::Message(String::from("test").into_bytes())).unwrap();
  }
}