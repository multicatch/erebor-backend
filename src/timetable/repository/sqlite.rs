mod load;

use crate::timetable::repository::TimetableConsumer;
use crate::timetable::{Timetable, TimetableVariant, TimetableId, ActivityOccurrence, Activity, TimetableDescriptor, ActivityGroup, ActivityTime, Weekday};
use crate::timetable::repository::sqlite::load::load_from_db;
use rusqlite::{params, Error, Statement, Connection};
use std::sync::mpsc;
use chrono::{Utc, DateTime};
use std::time::{UNIX_EPOCH, Duration};
use crate::timetable::repository::inmemory::{in_memory_repo, InMemoryRepo};
use std::sync::mpsc::{Sender, Receiver};

pub fn sqlite() -> (SqliteConsumer, InMemoryRepo) {
    let (consumer, mut provider) = in_memory_repo();
    let (sender, receiver) = mpsc::channel();
    let connection = Connection::open("test.db").unwrap();
    load_from_db(&connection, &mut provider).unwrap();
    listen_for_updates(connection, receiver);

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

fn listen_for_updates(connection: Connection, receiver: Receiver<Timetable>) {
    tokio::spawn(async move {
        info!("Initializing SQLite tables...");
        init_tables(&connection).unwrap();
        info!("Starting SQLite persist task...");
        loop {
            let timetable = receiver.recv().unwrap();
            insert(&connection, timetable);
        }
    });
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

fn insert(connection: &Connection, timetable: Timetable) {
    let id = timetable.descriptor.id.clone();
    insert_namespace(connection, &id.namespace);

    let timetable_insert = connection.prepare(
        "INSERT OR REPLACE INTO timetable (id, timetable_id, name, variant, variant_value, update_time, namespace_id) VALUES (?, ?, ?, ?, ?, ?, ?);"
    );

    let timetable_insert = timetable_insert
        .and_then(|statement| insert_timetable(statement, &timetable));

    if let Err(e) = timetable_insert {
        error!("Cannot save timetable [{}]: {}", id, e);
        return;
    }

    let result = delete_old_activities(connection, &id)
        .and_then(|_| {
            connection.prepare(
                "INSERT INTO activity(\
                id,\
                activity_id,\
                timetable_id,\
                name,\
                teacher,\
                occurrence,\
                occurrence_weekday,\
                occurrence_date,\
                group_symbol,\
                group_id,\
                group_name,\
                group_number,\
                start_time,\
                end_time,\
                duration,\
                room) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);"
            )
        })
        .and_then(|statement| insert_new_activities(statement, &id, &timetable.activities));

    if let Err(e) = result {
        error!("Cannot update activities for [{}]: {}", id, e)
    }
}

fn insert_namespace(connection: &Connection, namespace: &str) {
    if let Err(e) = connection.prepare(
        "INSERT OR IGNORE INTO namespace (id) VALUES (?);"
    ).and_then(|mut statement|
        statement.execute(params![namespace])
    ) {
        error!("Cannot insert namespace [{}]. It might already exist, but it is not certain. Details: {}", namespace, e)
    }
}

fn insert_timetable(statement: Statement, timetable: &Timetable) -> Result<usize, Error> {
    let mut statement = statement;

    let id = as_db_id(&timetable.descriptor.id);

    let (variant, variant_value) = match timetable.descriptor.variant {
        TimetableVariant::Semester(semester) => ("semester", Some(semester as i64)),
        TimetableVariant::Year(year) => ("year", Some(year as i64)),
        TimetableVariant::Unique => ("unique", None),
    };

    statement.execute(params![
        id, timetable.descriptor.id.id, timetable.descriptor.name, variant, variant_value,
        timetable.update_time.timestamp(), timetable.descriptor.id.namespace
    ])
}

fn delete_old_activities(connection: &Connection, timetable: &TimetableId) -> Result<usize, Error> {
    connection.prepare(
        "DELETE FROM activity WHERE timetable_id = ?;"
    ).and_then(|mut statement| {
        statement.execute(params![
            as_db_id(timetable)
        ])
    })
}

fn insert_new_activities(statement: Statement, id: &TimetableId, activities: &[Activity]) -> Result<(), Error> {
    let mut statement = statement;

    activities.iter().try_for_each(|activity| {
        let (occurrence, occurrence_weekday, occurrence_date) = match activity.occurrence.clone() {
            ActivityOccurrence::Regular { weekday } => ("regular", Some(u8::from(weekday)), None),
            ActivityOccurrence::Special { date } => ("special", None, Some(date)),
        };

        let activity_id = format!("{}_{}", id.namespace, activity.id);
        statement.execute(params![
                activity_id, activity.id, as_db_id(id),
                activity.name, activity.teacher, occurrence, occurrence_weekday, occurrence_date,
                activity.group.symbol, activity.group.id, activity.group.name, activity.group.number,
                activity.time.start_time, activity.time.end_time, activity.time.duration, activity.room
        ]).map(|_| ())
    })
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