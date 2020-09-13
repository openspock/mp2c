use std::sync;
use std::sync::mpsc;
use std::thread;

type Data = Box<Vec<u8>>;

/// `Event` is an enum that offers various type of events that will be 
/// handled by an mp2c carousel.
#[derive(Debug, Clone)]
enum Event {
  Message(Data),
  Terminate,
}

/// `Consumer` enables to implement handling logic for a vector of bytes.
/// 
/// Each consumer which would like to receive a message should implement
/// this trait. 
pub trait Consumer {
  fn consume(&self, data: Vec<u8>);
}

/// `Poller` is a simple struct that encapsulates a polling thread that calls
/// the encapsulating `Consumer` for each `Event`.
struct Poller {
  thread: Option<thread::JoinHandle<()>>,
}

impl Poller {

  fn new<T: ?Sized>(consumer: Box<T>,rx: sync::Arc<sync::Mutex<mpsc::Receiver<Event>>>) -> Poller 
    where
      T: Consumer + Send + 'static  
  {
    let thread = thread::spawn(move || loop {
      match rx.lock().unwrap().recv() {
        Ok(event) => {
          match event {
            Event::Message(data) => {
              let data = data.clone();
              consumer.consume(*data);
            }
            Event::Terminate => {
              break;
            }
          }     
        },
        Err(e) => println!("Poller error receiving an event: {}", e),
      }
    });

    Poller {
      thread: Some(thread),
    }
  }
}

struct Multipier {
  pollers: Vec<Poller>,
  thread: Option<thread::JoinHandle<()>>,
}

impl Multipier {
  fn new<T: ?Sized>(consumers: Vec<Box<T>>,rx: sync::Arc<sync::Mutex<mpsc::Receiver<Event>>>) -> Multipier 
    where
      T: Consumer + Send + 'static  
  {
    let mut multiplier_txs: Vec<mpsc::Sender<Event>> = Vec::with_capacity(consumers.len());

    let pollers: Vec<Poller> = consumers.into_iter().map(|c| {
      let (ctx, crx) = mpsc::channel::<Event>();

      let crx = sync::Arc::new(sync::Mutex::new(crx));

      multiplier_txs.push(ctx);

      Poller::new(c, sync::Arc::clone(&crx))
    }).collect();

    let thread = thread::spawn(move || {    
      loop {
        let cloned = multiplier_txs.clone();
        match rx.lock().unwrap().recv() {
          Ok(event) => {              
            cloned.into_iter().for_each(|tx| {
              tx.send(event.clone()).unwrap();
            });
            if let Event::Terminate = event {
              break;
            }              
          },
          Err(e) => println!("Multiplier error receiving an event: {}", e),
        }
      }
    });

    Multipier {
      pollers,
      thread: Some(thread),
    }
  }
}

/// `Carousel` represents a multi producer multi polling consumer data carousel. It enables
/// message passing from multiple producers to multiple consumers asynchronously.
/// 
/// It accepts a vector of bytes as a message/ event.
/// 
/// A mp2c `Carousel` can be created for a list of consumers. However, each consumer
/// is expected to implement the `Consumer` trait. 
/// 
/// A multiplier thread is started which receives one end of an async channel. 
/// Each message `put` on the `Carousel` is sent to this multiplier thread. The job
/// of the `Multiplier` is to clone each incoming event/ message and send it to each 
/// polling consumer.
/// 
/// For each consumer, a poller thread is started which receives one end of an async
/// channel. The transmitting end of the channel is with the `Multiplier` thread. The 
/// poller calls `Consumer::consume` on it's registered consumer.
/// 
/// # Example
/// ```
/// use mp2c::asynch::{Carousel, Consumer};
/// 
/// struct TestConsumer1;
///
/// impl Consumer for TestConsumer1 {
///   fn consume(&self, data: Vec<u8>) {
///     let msg = String::from_utf8(data).unwrap();
///     // do something with msg
///   }
/// }
///
/// struct TestConsumer2;
///
/// impl Consumer for TestConsumer2 {
///  fn consume(&self, data: Vec<u8>) {
///    let msg = String::from_utf8(data).unwrap();
///    // do something with msg   
///  }
/// }
///
/// let mut v: Vec<Box<dyn Consumer + Send + 'static>> = Vec::new();
/// v.push(Box::new(TestConsumer1));
/// v.push(Box::new(TestConsumer2));
///
/// let c = Carousel::new(v);
///
/// c.put(String::from("test").into_bytes());
/// 
/// ```

pub struct Carousel {
  tx: mpsc::Sender<Event>,  
  multiplier: Option<Multipier>,
}

impl Carousel {

  /// Creates a new `Carousel` for a vector of consumers.
  pub fn new<T: ?Sized>(consumers: Vec<Box<T>>) -> Carousel
    where 
      T: Consumer + Send + 'static 
  {
    assert!(consumers.len() > 0);

    let (tx, rx) = mpsc::channel::<Event>();

    let rx = sync::Arc::new(sync::Mutex::new(rx));

    let multiplier = Multipier::new(consumers, rx);
    
    Carousel {
      tx,
      multiplier: Some(multiplier),
    }
  }

  /// Puts a message on the `Carousel` which will be asynchronously
  /// sent to all it's consumers.
  pub fn put(&self, data: Vec<u8>) {
    let data = Box::new(data);
    let event = Event::Message(data);
    self.tx.send(event).unwrap();
  }
}

impl Clone for Carousel {
  fn clone(&self) -> Self {
    Carousel {
      tx: self.tx.clone(),
      multiplier: Option::None,
    }
  }
}

impl Drop for Carousel {
  fn drop(&mut self) {
      if let Some(multiplier) = &mut self.multiplier {
        println!("Sending terminate message to all pollers.");

        self.tx.send(Event::Terminate).unwrap();

        if let Some(multiplier_thread) = multiplier.thread.take() {
          multiplier_thread.join().unwrap();
        }

        println!("Shutting down all pollers.");
    
        for poller in &mut multiplier.pollers {
            if let Some(thread) = poller.thread.take() {
                thread.join().unwrap();
            }
        }
      }
  }
}

#[cfg(test)]
mod tests {
  use crate::asynch::{Consumer, Carousel};

  #[test]
  fn basic() {
    struct TestConsumer1;

    impl Consumer for TestConsumer1 {
      fn consume(&self, data: Vec<u8>) {
        assert_eq!(String::from_utf8(data).unwrap(), String::from("test"));
      }
    }
  
    struct TestConsumer2;
  
    impl Consumer for TestConsumer2 {
      fn consume(&self, data: Vec<u8>) {
        assert_eq!(String::from_utf8(data).unwrap(), String::from("test"));
      }
    }

    let mut v: Vec<Box<dyn Consumer + Send + 'static>> = Vec::new();
    v.push(Box::new(TestConsumer1));
    v.push(Box::new(TestConsumer2));
    let c = Carousel::new(v);

    c.put(String::from("test").into_bytes());
    c.put(String::from("test").into_bytes());
    c.put(String::from("test").into_bytes());
    c.put(String::from("test").into_bytes());

    std::thread::sleep(std::time::Duration::from_secs(2));
  }

  #[test]
  fn multi_producer() {
    struct TestConsumer1;

    impl Consumer for TestConsumer1 {
      fn consume(&self, data: Vec<u8>) {
        assert_eq!(String::from_utf8(data).unwrap(), String::from("test"));
      }
    }
  
    struct TestConsumer2;
  
    impl Consumer for TestConsumer2 {
      fn consume(&self, data: Vec<u8>) {
        assert_eq!(String::from_utf8(data).unwrap(), String::from("test"));
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
  }  
}