use crate::timetable::repository::TimetableConsumer;
use crate::timetable::{Timetable, TimetableVariant, TimetableId, ActivityOccurrence, Activity};
use rusqlite::{params, Error, Statement, Connection};

pub struct SqliteConsumer {
    connection: Connection,
}

impl SqliteConsumer {
    pub fn new(connection: Connection) -> Result<SqliteConsumer, Error> {
        init_tables(&connection)?;

        Ok(SqliteConsumer {
            connection,
        })
    }
}

impl TimetableConsumer for SqliteConsumer {
    fn consume(&mut self, timetable: Timetable) {
        let id = timetable.descriptor.id.clone();
        let timetable_insert = self.connection.prepare(
            "INSERT OR REPLACE INTO timetable (id, name, variant, variant_value, update_time) VALUES (?, ?, ?, ?, ?);"
        );

        let timetable_insert = timetable_insert
            .and_then(|statement| insert_timetable(statement, &timetable));

        if let Err(e) = timetable_insert {
            error!("Cannot save timetable [{}]: {}", id, e);
            return;
        }

        let result = delete_old_activities(&self.connection, &id)
            .and_then(|_| {
                self.connection.prepare(
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
}

fn init_tables(connection: &Connection) -> Result<usize, Error> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS timetable(\
                id TEXT NOT NULL PRIMARY KEY,\
                name TEXT NOT NULL,\
                variant TEXT NOT NULL,\
                variant_value INTEGER,\
                update_time INTEGER NOT NULL\
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
                group_number TEXT NOT NULL,\
                start_time TEXT NOT NULL,\
                end_time TEXT NOT NULL,\
                duration TEXT NOT NULL,\
                room TEXT,\
                FOREIGN KEY(timetable_id) REFERENCES timetable(id)\
            );",
        [],
    )
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
            id, timetable.descriptor.name, variant, variant_value, timetable.update_time.timestamp()
        ])
}

fn delete_old_activities(connection: &Connection, timetable: &TimetableId) -> Result<usize, Error> {
    connection.prepare(
        "DELETE FROM activity WHERE timetable_id = ?;"
    ).and_then(|mut statement| {
        statement.execute(params![
            as_db_id(&timetable)
        ])
    })
}

fn insert_new_activities(statement: Statement, id: &TimetableId, activities: &[Activity]) -> Result<(), Error> {
    let mut statement = statement;

    activities.iter().map(|activity| {
        let (occurrence, occurrence_weekday, occurrence_date) = match activity.occurrence.clone() {
            ActivityOccurrence::Regular { weekday } => ("regular", Some(u8::from(weekday)), None),
            ActivityOccurrence::Special { date } => ("special", None, Some(date)),
        };

        statement.execute(params![
                format!("{}_{}", id.namespace, activity.id), activity.id, as_db_id(&id),
                activity.name, activity.teacher, occurrence, occurrence_weekday, occurrence_date,
                activity.group.symbol, activity.group.id, activity.group.name, activity.group.number,
                activity.time.start_time, activity.time.end_time, activity.time.duration, activity.room
        ]).map(|_| ())
    }).collect()
}

fn as_db_id(timetable: &TimetableId) -> String {
    format!("{}_{}", timetable.namespace, timetable.id)
}