use std::process::exit;
use std::sync::mpsc::{channel, Sender, RecvError};
use std::thread;

use crate::timetable::{Timetable, TimetableId, TimetableDescriptor};
use std::sync::Arc;

pub mod inmemory;

#[derive(Clone)]
pub struct ShareableTimetableProvider {
    actual: Arc<dyn TimetableProvider + Send + Sync>,
}

impl ShareableTimetableProvider {
    pub fn new<T>(actual: T) -> ShareableTimetableProvider
        where T: TimetableProvider + Send + Sync,
              T: 'static,
    {
        ShareableTimetableProvider { actual: Arc::new(actual) }
    }
}

impl TimetableProvider for ShareableTimetableProvider {
    fn get(&self, id: TimetableId) -> Option<Timetable> {
        self.actual.get(id)
    }

    fn namespaces(&self) -> Vec<String> {
        self.actual.namespaces()
    }

    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableDescriptor>> {
        self.actual.available_timetables(namespace)
    }
}

pub trait TimetableConsumer {
    fn consume(&mut self, timetable: Timetable);
}

pub trait TimetableProvider {
    fn get(&self, id: TimetableId) -> Option<Timetable>;
    fn namespaces(&self) -> Vec<String>;
    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableDescriptor>>;
}

pub fn listen_for_timetables(publisher: Box<dyn TimetableConsumer + Send>, exit_on_failure: bool) -> Sender<Timetable> {
    debug!("Initializing timetable listener.");
    let (tx, rx) = channel::<Timetable>();

    thread::spawn(move || {
        info!("Listening for timetable updates, exit_on_failure: [{}]", exit_on_failure);
        let mut consumer = publisher;
        loop {
            let recv = rx.recv();
            consumer = receive_timetable(recv, consumer, exit_on_failure)
        }
    });

    debug!("Timetable initialization complete.");
    tx
}

fn receive_timetable(recv: Result<Timetable, RecvError>,
                     consumer: Box<dyn TimetableConsumer + Send>,
                     exit_on_failure: bool,
) -> Box<dyn TimetableConsumer + Send> {
    match recv {
        Ok(timetable) => {
            let mut consumer = consumer;
            trace!("Received timetable with id [{}]", timetable.descriptor.id);
            consumer.consume(timetable);
            consumer
        }
        Err(_) => {
            error!("Critical error during timetable listening - MPSC channel dropped.");
            if exit_on_failure {
                exit(255);
            }
            consumer
        }
    }
}