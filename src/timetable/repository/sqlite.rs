mod load;
mod persist;

use crate::timetable::repository::TimetableConsumer;
use crate::timetable::{Timetable, TimetableVariant, TimetableId};
use crate::timetable::repository::sqlite::load::load_from_db;
use rusqlite::Connection;
use std::sync::mpsc;
use crate::timetable::repository::inmemory::{in_memory_repo, InMemoryRepo};
use std::sync::mpsc::Sender;
use crate::timetable::repository::sqlite::persist::listen_for_db_updates;

pub fn sqlite() -> (SqliteConsumer, InMemoryRepo) {
    let (consumer, mut provider) = in_memory_repo();
    let (sender, receiver) = mpsc::channel();
    let connection = Connection::open("test.db").unwrap();
    load_from_db(&connection, &mut provider).unwrap();
    listen_for_db_updates(connection, receiver);

    (SqliteConsumer::new(consumer, sender), provider)
}

pub struct SqliteConsumer {
    consumer: InMemoryRepo,
    sender: Sender<Timetable>,
}

impl SqliteConsumer {
    pub fn new(consumer: InMemoryRepo, sender: Sender<Timetable>) -> SqliteConsumer {
        SqliteConsumer {
            consumer,
            sender,
        }
    }
}

impl TimetableConsumer for SqliteConsumer {
    fn consume(&mut self, timetable: Timetable) {
        self.consumer.consume(timetable.clone());
        if let Err(e) = self.sender.send(timetable) {
            error!("Could not persist [{}] - the SQLite did not receive the timetable. The channel was probably dropped.", e.0.descriptor.id)
        };
    }
}

fn as_db_id(timetable: &TimetableId) -> String {
    format!("{}_{}", timetable.namespace, timetable.id)
}

fn db_to_variant(variant: String, variant_value: Option<u32>) -> TimetableVariant {
    if variant == *"semester" {
        TimetableVariant::Semester(variant_value.unwrap())
    } else if variant == *"year" {
        TimetableVariant::Year(variant_value.unwrap())
    } else {
        TimetableVariant::Unique
    }
}