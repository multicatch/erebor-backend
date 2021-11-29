use std::time::{Duration, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Error, params, Row};

use crate::timetable::{Activity, ActivityGroup, ActivityOccurrence, ActivityTime, Timetable, TimetableDescriptor, TimetableId, Weekday};
use crate::timetable::repository::sqlite::{as_db_id, db_to_variant};
use crate::timetable::repository::TimetableConsumer;

pub fn load_from_db<T>(connection: &Connection, provider: &mut T) -> Result<usize, Error>
    where T: TimetableConsumer {
    let timetables: Vec<TimetableDescriptor> = fetch_available_timetables(connection)?;

    Ok(timetables.into_iter()
        .filter_map(|desc| fetch_timetable(connection, desc.id).ok()?)
        .map(|timetable| {
            provider.consume(timetable)
        })
        .count())
}

fn fetch_available_timetables(connection: &Connection) -> Result<Vec<TimetableDescriptor>, Error> {
    let mut prepared_timetables = connection.prepare(
        "SELECT namespace_id, timetable_id, name, variant, variant_value FROM timetable;"
    )?;

    prepared_timetables.query_map(params![], |row| {
        Ok(TimetableDescriptor::new(
            TimetableId::new(
                row.get(0)?,
                row.get(1)?,
            ),
            row.get(2)?,
            db_to_variant(row.get(3)?, row.get(4)?),
        ))
    }).unwrap().collect()
}

fn fetch_timetable(connection: &Connection, id: TimetableId) -> Result<Option<Timetable>, Error> {
    let timetable_details = fetch_descriptor_and_update_date(connection, id)?;
    if timetable_details.is_none() {
        return Ok(None);
    }
    let (descriptor, update_time) = timetable_details.unwrap();

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
    )?;

    let timetable_id = as_db_id(&descriptor.id);
    let activities: Vec<Activity> = activities
        .query_map(
            params![
                timetable_id
            ],
            try_create_activity
        )
        .unwrap()
        .filter_map(|result| match result {
            Ok(activities) => Some(activities),
            Err(e) => {
                error!("Cannot create activity: {}", e);
                None
            }
        })
        .collect();

    Ok(Some(Timetable {
        descriptor,
        activities,
        update_time,
    }))
}

fn fetch_descriptor_and_update_date(connection: &Connection, id: TimetableId) -> Result<Option<(TimetableDescriptor, DateTime<Utc>)>, Error> {
    let mut statement = connection.prepare(
        "SELECT name, variant, variant_value, update_time FROM timetable WHERE id = ?;"
    )?;

    let timetable_id = as_db_id(&id);
    let query_result = statement
        .query_map(
            params![
                timetable_id
            ],
            |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
        .unwrap()
        .next();

    if query_result.is_none() {
        return Ok(None);
    }

    let (name, variant, variant_value, update_time) = query_result.unwrap()?;

    let timestamp = UNIX_EPOCH + Duration::from_secs(update_time);
    let update_time = DateTime::<Utc>::from(timestamp);

    let descriptor = TimetableDescriptor::new(id, name, db_to_variant(variant, variant_value));

    Ok(Some((descriptor, update_time)))
}

fn try_create_activity(row: &Row) -> Result<Activity, Error> {
    let occurrence_type: String = row.get(3)?;
    let occurrence = if occurrence_type == *"special" {
        ActivityOccurrence::Special {
            date: row.get(5)?
        }
    } else {
        let weekday: u8 = row.get(4)?;
        ActivityOccurrence::Regular {
            weekday: Weekday::from(weekday)
        }
    };

    let group_id: String = row.get(7)?;
    Ok(Activity {
        id: row.get(0)?,
        name: row.get(1)?,
        teacher: row.get(2)?,
        occurrence,
        group: ActivityGroup {
            symbol: row.get(6)?,
            id: group_id.parse().unwrap_or(0),
            name: row.get(8)?,
            number: row.get(9)?,
        },
        time: ActivityTime {
            start_time: row.get(10)?,
            end_time: row.get(11)?,
            duration: row.get(12)?,
        },
        room: row.get(13)?,
    })
}