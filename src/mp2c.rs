use std::sync;
use std::sync::mpsc;
use std::thread;

type Data = Box<Vec<u8>>;

/// `Event` is an enum that offers various type of events that will be 
/// handled by an mp2c carousel.
#[derive(Debug, Clone)]
pub enum Event {
  Message(Data),
  Terminate,
}

/// `Consumer` enables to implement handling logic for a vector of bytes.
pub trait Consumer {
  fn consume(&self, data: Vec<u8>);
}

/// `Poller` is a simple struct that encapsulates a polling thread that calls
/// the encapsulating `Consumer` for each `Event`.
///
/// counts the number of `Event`s processed by each poller.
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

struct Carousel {
  pollers: Vec<Poller>,
  tx: mpsc::Sender<Event>,
}

impl Carousel {

  fn new<T: ?Sized>(consumers: Vec<Box<T>>) -> Carousel
    where 
      T: Consumer + Send + 'static 
  {
    assert!(consumers.len() > 0);

    let (tx, rx) = mpsc::channel::<Event>();

    let rx = sync::Arc::new(sync::Mutex::new(rx));

    let pollers: Vec<Poller> = consumers.into_iter().map(|c| Poller::new(c, sync::Arc::clone(&rx))).collect();
    
    Carousel {
      pollers,
      tx,
    }
  }

  fn put(&self, data: Vec<u8>) {
    let data = Box::new(data);
    let event = Event::Message(data);
    self.tx.send(event).unwrap();
  }
}

impl Drop for Carousel {
  fn drop(&mut self) {
      println!("Sending terminate message to all pollers.");

      for _ in &self.pollers {
          self.tx.send(Event::Terminate).unwrap();
      }

      println!("Shutting down all pollers.");

      for poller in &mut self.pollers {
          if let Some(thread) = poller.thread.take() {
              thread.join().unwrap();
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use crate::mp2c::{Consumer, Carousel};

  struct TestConsumer1;

  impl Consumer for TestConsumer1 {
    fn consume(&self, data: Vec<u8>) {
      println!("Test consumer1: {}",String::from_utf8(data).unwrap());
    }
  }

  struct TestConsumer2;

  impl Consumer for TestConsumer2 {
    fn consume(&self, data: Vec<u8>) {
      println!("Test consumer2: {}",String::from_utf8(data).unwrap());
    }
  }


  #[test]
  fn basic() {
    let mut v: Vec<Box<dyn Consumer + Send + 'static>> = Vec::new();
    v.push(Box::new(TestConsumer1));
    v.push(Box::new(TestConsumer2));
    let c = Carousel::new(v);

    c.put(String::from("test").into_bytes());

    std::thread::sleep(std::time::Duration::from_secs(2));
  }
}