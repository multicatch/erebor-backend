use crate::timetable::repository::{TimetableConsumer, TimetableProvider};
use crate::timetable::{Timetable, TimetableVariant, TimetableId, ActivityOccurrence, Activity, TimetableDescriptor, ActivityGroup, ActivityTime, Weekday};
use rusqlite::{params, Error, Statement, Connection};
use std::sync::{Arc, Mutex};
use chrono::{Utc, DateTime};
use std::time::{UNIX_EPOCH, Duration};

pub fn sqlite() -> (SqliteConsumer, SqliteProvider) {
    let connection = Connection::open("test.db").unwrap();
    let consumer = SqliteConsumer::new(connection).unwrap();

    let connection = Connection::open("test.db").unwrap();
    let provider = SqliteProvider::new(connection).unwrap();

    (consumer, provider)
}

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

        insert_namespace(&self.connection, &id.namespace);

        let timetable_insert = self.connection.prepare(
            "INSERT OR REPLACE INTO timetable (id, timetable_id, name, variant, variant_value, update_time, namespace_id) VALUES (?, ?, ?, ?, ?, ?, ?);"
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

pub struct SqliteProvider {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteProvider {
    pub fn new(connection: Connection) -> Result<SqliteProvider, Error> {
        init_tables(&connection)?;

        Ok(SqliteProvider {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

impl TimetableProvider for SqliteProvider {

    fn get(&self, id: TimetableId) -> Option<Timetable> {
        let connection = Connection::open("test.db").unwrap();
        let mut prepared_timetable = connection.prepare(
            "SELECT name, variant, variant_value, update_time FROM timetable WHERE id = ?;"
        ).unwrap();

        let timetable_id = as_db_id(&id);
        let (name, variant, variant_value, update_time) = prepared_timetable.query_map(params![
            timetable_id.clone()
        ], |row| {
            Ok((row.get(0).unwrap(), row.get(1).unwrap(), row.get(2).unwrap(), row.get(3).unwrap()))
        }).unwrap().next()?.unwrap();

        let timestamp = UNIX_EPOCH + Duration::from_secs(update_time);
        let update_time = DateTime::<Utc>::from(timestamp);

        let mut activities = connection.prepare(
            "SELECT activity_id,\
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
                room FROM activity WHERE timetable_id = ?;"
        ).unwrap();

        let activities: Result<Vec<Activity>, _> = activities.query_map(params![
            timetable_id
        ], |row| {
            let occurrence_type: String = row.get(3).unwrap();
            let occurrence = if occurrence_type == "special".to_string() {
                ActivityOccurrence::Special {
                    date: row.get(5).unwrap()
                }
            } else {
                let weekday: u8 = row.get(4).unwrap();
                ActivityOccurrence::Regular {
                    weekday: Weekday::from(weekday)
                }
            };

            let group_id: String = row.get(7).unwrap();
            Ok(Activity {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                teacher: row.get(2).unwrap(),
                occurrence,
                group: ActivityGroup {
                    symbol: row.get(6).unwrap(),
                    id: group_id.parse().unwrap(),
                    name: row.get(8).unwrap(),
                    number: row.get(9).unwrap(),
                },
                time: ActivityTime {
                    start_time: row.get(10).unwrap(),
                    end_time: row.get(11).unwrap(),
                    duration: row.get(12).unwrap(),
                },
                room: row.get(13).unwrap(),
            })
        }).unwrap().collect();

        Some(Timetable {
            descriptor: TimetableDescriptor::new(id, name, db_to_variant(variant, variant_value)),
            activities: activities.unwrap(),
            update_time,
        })
    }

    fn namespaces(&self) -> Vec<String> {
        let connection = Connection::open("test.db").unwrap();
        let mut prepared_namespaces = connection.prepare(
            "SELECT id FROM namespace;"
        ).unwrap();

        let result: Result<Vec<String>, _> = prepared_namespaces.query_map([], |row| {
            Ok(row.get(0).unwrap())
        }).unwrap().collect();

        result.unwrap()
    }

    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableDescriptor>> {
        let connection = Connection::open("test.db").unwrap();
        let mut prepared_timetables = connection.prepare(
            "SELECT namespace_id, timetable_id, name, variant, variant_value FROM timetable WHERE namespace_id = ?;"
        ).unwrap();

        let result: Result<Vec<TimetableDescriptor>, _> = prepared_timetables.query_map(params![
            namespace
        ], |row| {
            Ok(TimetableDescriptor::new(
                TimetableId::new(
                    namespace.to_string(),
                    row.get(1).unwrap()
                ),
                row.get(2).unwrap(),
                db_to_variant(row.get(3).unwrap(), row.get(4).unwrap())
            ))
        }).unwrap().collect();

        Some(result.unwrap())
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

fn db_to_variant(variant: String, variant_value: Option<u32>) -> TimetableVariant {
    if variant == "semester".to_string() {
        TimetableVariant::Semester(variant_value.unwrap())
    } else if variant == "year".to_string() {
        TimetableVariant::Year(variant_value.unwrap())
    } else {
        TimetableVariant::Unique
    }
}