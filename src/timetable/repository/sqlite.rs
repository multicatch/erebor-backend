use crate::timetable::repository::TimetableConsumer;
use crate::timetable::{Timetable, TimetableDescriptor, TimetableVariant};
use sqlite::{Error, Statement};
use chrono::Timelike;

struct SqliteConsumer<'a> {
    connection: sqlite::Connection,
    timetable_insert: Statement<'a>
}

impl<'a> SqliteConsumer<'a> {
    fn new(connection: sqlite::Connection) -> Result<SqliteConsumer<'a>, Error> {
        init_tables(&connection)?;
        let timetable_insert = connection.prepare(
            "INSERT INTO timetable (id, name, variant, variant_value, update_time) VALUES (?, ?, ?, ?, ?);"
        )?;

        Ok(SqliteConsumer {
            connection,
            timetable_insert
        })
    }
}

impl<'a> TimetableConsumer for SqliteConsumer<'a> {
    fn consume(&mut self, timetable: &Timetable) {
        self.timetable_insert.bind(1, format!("{}_{}", timetable.descriptor.id.namespace, timetable.descriptor.id.id));
        self.timetable_insert.bind(2, timetable.descriptor.name.clone());
        let (variant, variant_value) = match timetable.descriptor.variant {
            TimetableVariant::Semester(semester) => ("semester", Some(semester)),
            TimetableVariant::Year(year) => ("year", Some(year)),
            TimetableVariant::Unique => ("unique", None),
        };
        self.timetable_insert.bind(3, variant);
        self.timetable_insert.bind(4, variant_value);
        self.timetable_insert.bind(5, timetable.update_time.timestamp());

        self.timetable_insert.next();

        self.timetable_insert.reset();
    }
}

fn init_tables(connection: &sqlite::Connection) -> Result<(), Error> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS timetable(\
                id TEXT NOT NULL PRIMARY KEY,\
                name TEXT NOT NULL,\
                variant TEXT NOT NULL,\
                variant_value INTEGER,\
                update_time INTEGER NOT NULL\
            );\
            \
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
            );"
    )
}