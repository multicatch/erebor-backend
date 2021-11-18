use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc::{channel, Sender, RecvError};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::timetable::Timetable;
use std::sync::Arc;

pub mod inmemory;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct TimetableId {
    pub namespace: String,
    pub id: String
}

impl TimetableId {
    pub fn new(namespace: String, id: String) -> TimetableId {
        TimetableId { namespace, id }
    }
}

pub struct TimetablePacket(pub TimetableId, pub Timetable);

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

    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableId>> {
        self.actual.available_timetables(namespace)
    }
}

pub trait TimetableConsumer {
    fn consume(&mut self, id: TimetableId, timetable: Timetable);
}

pub trait TimetableProvider {
    fn get(&self, id: TimetableId) -> Option<Timetable>;
    fn namespaces(&self) -> Vec<String>;
    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableId>>;
}

#[derive(Clone)]
pub struct TimetableRepository {
    timetables: HashMap<TimetableId, Timetable>,
    available: HashMap<String, Vec<TimetableId>>,
}

impl TimetableRepository {
    pub fn new() -> TimetableRepository {
        TimetableRepository {
            timetables: HashMap::new(),
            available: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: TimetableId, timetable: Timetable) {
        self.timetables.insert(id.clone(), timetable);
        let available = self.available.get(&id.namespace)
            .map(|v| v.clone())
            .unwrap_or_else(|| vec![id.clone()]);
        self.available.insert(id.namespace.clone(), available);
    }

    pub fn get(&self, id: TimetableId) -> Option<&Timetable> {
        self.timetables.get(&id)
    }

    pub fn namespaces(&self) -> Vec<String> {
        self.available.keys().cloned().into_iter().collect()
    }

    pub fn available_timetables(&self, namespace: &str) -> Option<&Vec<TimetableId>> {
        self.available.get(namespace)
    }
}

impl Default for TimetableRepository {
    fn default() -> Self {
        Self::new()
    }
}

pub fn listen_for_timetables(publisher: Box<dyn TimetableConsumer + Send>, exit_on_failure: bool) -> Sender<TimetablePacket> {
    debug!("Initializing timetable listener.");
    let (tx, rx) = channel::<TimetablePacket>();

    thread::spawn(move || {
        info!("Listening for timetable updates, exit_on_failure: [{}]", exit_on_failure);
        let mut publisher = publisher;
        loop {
            let recv = rx.recv();
            publisher = receive_timetable(recv, publisher, exit_on_failure)
        }
    });

    debug!("Timetable initialization complete.");
    tx
}

fn receive_timetable(recv: Result<TimetablePacket, RecvError>,
                     publisher: Box<dyn TimetableConsumer + Send>,
                     exit_on_failure: bool,
) -> Box<dyn TimetableConsumer + Send> {
    match recv {
        Ok(timetable) => {
            let mut publisher = publisher;
            trace!("Received timetable with id [{}:{}]", timetable.0.namespace, timetable.0.id);
            publisher.consume(timetable.0, timetable.1);
            publisher
        }
        Err(_) => {
            error!("Critical error during timetable listening - MPSC channel dropped.");
            if exit_on_failure {
                exit(255);
            }
            publisher
        }
    }
}