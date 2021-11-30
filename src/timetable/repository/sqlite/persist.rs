use crate::timetable::{Timetable, TimetableId, Activity};
use rusqlite::{params, Error, Statement, Connection};
use std::sync::mpsc::Receiver;
use crate::timetable::repository::sqlite::{as_db_id, variant_to_db, occurrence_to_db};
use std::process::exit;

pub fn listen_for_db_updates(connection: Connection, receiver: Receiver<Timetable>) {
    tokio::spawn(async move {
        info!("Starting SQLite persist task...");
        loop {
            match receiver.recv() {
                Ok(timetable) => insert(&connection, timetable),
                Err(_) => {
                    error!("Critical error in database updates listener - MPSC channel dropped.");
                    exit(255);
                }
            };
        }
    });
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

    let (variant, variant_value) = variant_to_db(&timetable.descriptor.variant);

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
        let (occurrence, occurrence_weekday, occurrence_date) = occurrence_to_db(&activity.occurrence);

        let timetable_id = as_db_id(id);
        let activity_id = format!("{}_{}", timetable_id, activity.id);
        statement.execute(params![
                activity_id, activity.id, timetable_id,
                activity.name, activity.teacher, occurrence, occurrence_weekday, occurrence_date,
                activity.group.symbol, activity.group.id, activity.group.name, activity.group.number,
                activity.time.start_time, activity.time.end_time, activity.time.duration, activity.room
        ]).map(|_| ())
    })
}