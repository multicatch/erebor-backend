use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc::{channel, Sender, RecvError};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::timetable::Timetable;

pub mod inmemory;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct TimetableId(pub String);

pub struct TimetablePacket(pub TimetableId, pub Timetable);

pub trait CloneableTimetableProvider: TimetableProvider + Clone {}

pub trait TimetableConsumer {
    fn consume(&mut self, id: TimetableId, timetable: Timetable);
}

pub trait TimetableProvider {
    fn get(&self, id: TimetableId) -> Option<Timetable>;
    fn available(&self) -> Vec<TimetableId>;
}

#[derive(Clone)]
pub struct TimetableRepository {
    timetables: HashMap<TimetableId, Timetable>,
}

impl TimetableRepository {
    pub fn new() -> TimetableRepository {
        TimetableRepository {
            timetables: HashMap::new()
        }
    }

    pub fn insert(&mut self, id: TimetableId, timetable: Timetable) {
        self.timetables.insert(id, timetable);
    }

    pub fn get(&self, id: TimetableId) -> Option<&Timetable> {
        self.timetables.get(&id)
    }

    pub fn available(&self) -> Vec<TimetableId> {
        self.timetables.iter().map(|(key, _)| {
            key.clone()
        }).collect()
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
                     exit_on_failure: bool
) -> Box<dyn TimetableConsumer + Send> {
    match recv {
        Ok(timetable) => {
            let mut publisher = publisher;
            trace!("Received timetable with id [{}]", timetable.0.0);
            publisher.consume(timetable.0, timetable.1);
            publisher
        },
        Err(_) => {
            error!("Critical error during timetable listening - MPSC channel dropped.");
            if exit_on_failure {
                exit(255);
            }
            publisher
        }
    }
}