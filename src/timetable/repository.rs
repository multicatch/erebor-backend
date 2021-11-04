use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::timetable::Timetable;
use std::process::exit;

#[derive(Eq, PartialEq, Hash)]
pub struct TimetableId(pub String);

pub struct TimetablePacket(pub TimetableId, pub Timetable);

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
}

pub fn listen_for_timetables(repository: TimetableRepository) -> (Arc<Mutex<TimetableRepository>>, Sender<TimetablePacket>) {
    let (tx, rx) = channel::<TimetablePacket>();
    let repo_arc = Arc::new(Mutex::new(repository));
    let result = repo_arc.clone();
    thread::spawn(move || {
        loop {
            let recv = rx.recv();
            match recv {
                Ok(timetable) => {
                    repo_arc.lock().unwrap().insert(timetable.0, timetable.1);
                },
                Err(e) => {
                    println!("MPSC channel dropped.");
                    exit(255);
                }
            }
        }
    });

    (result, tx)
}