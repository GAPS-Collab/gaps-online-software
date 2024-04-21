//! Thread control structures
//! FIXME - this should go to liftof-lib

use std::collections::HashMap;
use std::fmt;

/// Send runtime information 
/// to threads via shared memory
/// (Arc(Mutex)
#[derive(Default, Debug)]
pub struct ThreadControl {
  /// Stop ALL threads
  pub stop_flag                  : bool,
  /// Trigger calibration thread
  pub calibration_active         : bool,
  /// Keep track on how many calibration 
  /// packets we have received
  pub finished_calibrations      : HashMap<u8,bool>,
  /// alive indicator for cmd dispatch thread
  pub thread_cmd_dispatch_active : bool,
  /// alive indicator for data sink thread
  pub thread_data_sink_active    : bool,
  /// alive indicator for runner thread
  pub thread_runner_active       : bool,
  /// alive indicator for event builder thread
  pub thread_event_bldr_active   : bool,
  /// alive indicator for master trigger thread
  pub thread_master_trg_active   : bool,
}

impl ThreadControl {
  pub fn new() -> Self {
    Self {
      stop_flag                  : false,
      calibration_active         : false,
      finished_calibrations      : HashMap::<u8,bool>::new(),
      thread_cmd_dispatch_active : false,
      thread_data_sink_active    : false,
      thread_runner_active       : false,
      thread_event_bldr_active   : false,
      thread_master_trg_active   : false,
    }
  }
}

impl fmt::Display for ThreadControl {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut repr = String::from("<ThreadControl:");
    repr        += &(format!("\n  stop flag : {}", self.stop_flag));
    repr        += "    -- reported RB calibration activity:";
    repr        += &(format!("\n  RB cali active : {}", self.calibration_active));
    repr        += &(format!("\n  -- finished    : {:?}", self.finished_calibrations));       
    repr        += "    -- reported thread activity:";
    repr        += &(format!("\n  cmd dispatcher : {}", self.thread_cmd_dispatch_active));
    repr        += &(format!("\n  runner         : {}", self.thread_data_sink_active));
    repr        += &(format!("\n  data sink      : {}", self.thread_runner_active));
    repr        += &(format!("\n  master trig    : {}>", self.thread_master_trg_active));
    write!(f, "{}", repr)
  }
}


//enum Message {
//  NewJob(Job),
//  Terminate,
//}
//
//
///// Implements "standard" Threadpool. 
/////
///// Threadpool spawns unnamed threads 
///// for workers
//pub struct ThreadPool {
//  workers: Vec<Worker>,
//  sender: mpsc::Sender<Message>,
//}
//
//trait FnBox {
//  fn call_box(self: Box<Self>);
//}
//
//impl<F: FnOnce()> FnBox for F {
//  fn call_box(self: Box<F>) {
//    (*self)()
//  }
//}
//
//type Job = Box<dyn FnBox + Send + 'static>;
//
//impl ThreadPool {
//  /// Create a new ThreadPool.
//  ///
//  /// The size is the number of threads in the pool.
//  ///
//  /// # Panics
//  ///
//  /// The `new` function will panic if the size is zero.
//  pub fn new(size: usize) -> ThreadPool {
//    assert!(size > 0);
//
//    let (sender, receiver) = mpsc::channel();
//    let receiver = Arc::new(Mutex::new(receiver));
//    let mut workers = Vec::with_capacity(size);
//
//    for id in 0..size {
//      workers.push(Worker::new(id, Arc::clone(&receiver)));
//    }
//
//    ThreadPool {
//      workers,
//      sender,
//    }
//  }
//
//  pub fn execute<F>(&self, f: F)
//    where
//        F: FnOnce() + Send + 'static {
//    let job = Box::new(f);
//    self.sender.send(Message::NewJob(job)).unwrap();
//  }
//}
//
//impl Drop for ThreadPool {
//  fn drop(&mut self) {
//    info!("Sending terminate message to all workers.");
//
//    for _ in &mut self.workers {
//      self.sender.send(Message::Terminate).unwrap();
//    }
//
//    warn!("Shutting down all workers.");
//
//    for worker in &mut self.workers {
//      info!("Shutting down worker {}", worker.id);
//
//      if let Some(thread) = worker.thread.take() {
//          thread.join().unwrap();
//      }
//    }
//  }
//}
//
//struct Worker {
//  id: usize,
//  thread: Option<thread::JoinHandle<()>>,
//}
//
//impl Worker {
//  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) ->
//    Worker {
//      let thread = thread::spawn(move ||{
//      loop {
//        let message = receiver.lock().unwrap().recv().unwrap();
//        match message {
//          Message::NewJob(job) => {
//            trace!("Worker {} got a job; executing.", id);
//            job.call_box();
//          },
//          Message::Terminate => {
//            trace!("Worker {} was told to terminate.", id);
//            break;
//          },
//        }
//      }
//    });
//
//    Worker {
//      id,
//      thread: Some(thread),
//    }
//  }
//}
