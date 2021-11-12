use std::collections::HashMap;
use std::process::exit;
use std::sync::mpsc::{channel, Sender};
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

pub fn listen_for_timetables(publisher: Box<dyn TimetableConsumer + Send>) -> Sender<TimetablePacket> {
    let (tx, rx) = channel::<TimetablePacket>();

    thread::spawn(move || {
        let mut publisher = publisher;
        loop {
            let recv = rx.recv();
            match recv {
                Ok(timetable) => {
                    publisher.consume(timetable.0, timetable.1);
                },
                Err(_) => {
                    println!("MPSC channel dropped.");
                    exit(255);
                }
            }
        }
    });

    tx
}