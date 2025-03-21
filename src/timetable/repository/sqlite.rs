mod load;
mod persist;

use crate::timetable::repository::TimetableConsumer;
use crate::timetable::{Timetable, TimetableVariant, TimetableId, ActivityOccurrence};
use crate::timetable::repository::sqlite::load::load_from_db;
use rusqlite::{Connection, Error};
use std::sync::mpsc;
use crate::timetable::repository::inmemory::{in_memory_repo, InMemoryRepo};
use std::sync::mpsc::Sender;
use crate::timetable::repository::sqlite::persist::listen_for_db_updates;

pub fn create_sqlite(connection: Connection) -> (SqliteConsumer, InMemoryRepo) {
    info!("Initializing SQLite tables...");
    init_tables(&connection).unwrap();

    let (consumer, mut provider) = in_memory_repo();
    let (sender, receiver) = mpsc::channel();
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

fn init_tables(connection: &Connection) -> Result<usize, Error> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS namespace(\
                id TEXT NOT NULL PRIMARY KEY\
            );",
        [],
    )?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS timetable(\
                id TEXT NOT NULL PRIMARY KEY,\
                timetable_id TEXT NOT NULL,\
                name TEXT NOT NULL,\
                variant TEXT NOT NULL,\
                variant_value INTEGER,\
                update_time INTEGER NOT NULL,\
                namespace_id TEXT NOT NULL,\
                FOREIGN KEY(namespace_id) REFERENCES namespace(id)\
            );",
        [],
    )?;
    connection.execute(
        "
            CREATE TABLE IF NOT EXISTS activity(\
                id TEXT NOT NULL PRIMARY KEY,\
                activity_id TEXT NOT NULL,\
                timetable_id TEXT NOT NULL,\
                name TEXT NOT NULL,\
                teacher TEXT,\
                occurrence TEXT NOT NULL,\
                occurrence_weekday INTEGER,\
                occurrence_date TEXT,\
                group_symbol TEXT NOT NULL,\
                group_id TEXT NOT NULL,\
                group_name TEXT NOT NULL,\
                group_number TEXT,\
                start_time TEXT NOT NULL,\
                end_time TEXT NOT NULL,\
                duration TEXT NOT NULL,\
                room TEXT,\
                FOREIGN KEY(timetable_id) REFERENCES timetable(id)\
            );",
        [],
    )
}

fn as_db_id(timetable: &TimetableId) -> String {
    format!("{}_{}", timetable.namespace, timetable.id)
}

fn db_to_variant(variant: &str, variant_value: Option<u32>) -> Option<TimetableVariant> {
    match variant {
        SEMESTER_VARIANT => variant_value.map(TimetableVariant::Semester),
        YEAR_VARIANT => variant_value.map(TimetableVariant::Year),
        _ => Some(TimetableVariant::Unique),
    }
}

fn variant_to_db(variant: &TimetableVariant) -> (&str, Option<i64>){
    match variant {
        TimetableVariant::Semester(semester) => (SEMESTER_VARIANT, Some(*semester as i64)),
        TimetableVariant::Year(year) => (YEAR_VARIANT, Some(*year as i64)),
        TimetableVariant::Unique => (UNIQUE_VARIANT, None),
    }
}

const SEMESTER_VARIANT: &str = "semester";
const YEAR_VARIANT: &str = "year";
const UNIQUE_VARIANT: &str = "unique";

fn db_to_occurrence(kind: &str, weekday: Option<u8>, date: Option<String>) -> Option<ActivityOccurrence> {
    match kind {
        OCCURRENCE_SPECIAL => date.map(|date| ActivityOccurrence::Special {
            date
        }),
        OCCURRENCE_REGULAR => weekday.map(|weekday| ActivityOccurrence::Regular {
            weekday: weekday.into()
        }),
        unknown => {
            error!("Unknown occurrence type: [{}]", unknown);
            None
        }
    }
}

fn occurrence_to_db(occurrence: &ActivityOccurrence) -> (&str, Option<u8>, Option<String>) {
    match occurrence {
        ActivityOccurrence::Regular { weekday } => (OCCURRENCE_REGULAR, Some(u8::from(weekday.clone())), None),
        ActivityOccurrence::Special { date } => (OCCURRENCE_SPECIAL, None, Some(date.clone())),
    }
}

const OCCURRENCE_REGULAR: &str = "regular";
const OCCURRENCE_SPECIAL: &str = "special";