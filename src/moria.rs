use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::timetable::{Timetable, TimetableVariant, TimetableDescriptor, TimetableId, Activity, ActivityGroup, ActivityOccurrence, Weekday, ActivityTime};
use serde::Deserialize;
use std::collections::HashMap;

pub fn sync_moria(_uuid: Uuid, _sched: JobScheduler, tx: Sender<Timetable>) {
    tokio::spawn(async {
        fetch_timetables(tx).await;
    });
}

async fn fetch_timetables(tx: Sender<Timetable>) {
    let client = reqwest::Client::new();
    let timetable_ids: MoriaResult<MoriaArray<MoriaTimetableId>> = client.get("http://moria.umcs.lublin.pl/api/students_list")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .map(|result| serde_json::from_str(&result))
        .unwrap()
        .unwrap();

    let ids: Vec<_> = timetable_ids
        .result
        .array
        .into_iter()
        .map(|id|
            (TimetableId::new("moria".to_string(), format!("{}", id.id)), id.name)
        )
        .collect();

    for (id, name) in ids {
        let id_str = id.id.clone();
        let timetable = Timetable::new(
            TimetableDescriptor::new(
                id,
                name,
                TimetableVariant::Unique,
            ),
            fetch_activities(&client, &id_str).await);
        if let Err(e) = tx.send(timetable) {
            error!("Cannot send timetable [{}] to repository - MPSC error!", e.0.descriptor.id);
        }
    }
}

async fn fetch_activities(client: &reqwest::Client, id: &str) -> Vec<Activity> {
    let mut params = HashMap::new();
    params.insert("id", id);

    let moria_activities: MoriaResult<MoriaArray<MoriaEventWrapper>> = client.get("http://moria.umcs.lublin.pl/api/activity_list_for_students")
        .json(&params)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .map(|result| serde_json::from_str(&result))
        .unwrap()
        .unwrap();

    moria_activities
        .result
        .array
        .into_iter()
        .map(|wrapper| {
            let event = wrapper.event_array.get(0).unwrap();
            let teacher = wrapper.teacher_array.get(0).unwrap();

            Activity {
                name: wrapper.subject,
                teacher: teacher.name.clone(),
                occurrence: ActivityOccurrence::Regular {
                    weekday: to_weekday(event.weekday),
                },
                group: ActivityGroup {
                    symbol: wrapper.kind.shortcut.clone(),
                    name: wrapper.kind.name.clone(),
                    id: wrapper.kind.id,
                },
                time: ActivityTime {
                    start_time: event.start_time.clone(),
                    end_time: event.end_time.clone(),
                    duration: event.length.clone(),
                },
                room: Some(event.room.clone())
            }
        })
        .collect()
}

fn to_weekday(number: u8) -> Weekday {
    match number {
        1 => Weekday::Monday,
        2 => Weekday::Tuesday,
        3 => Weekday::Wednesday,
        4 => Weekday::Thursday,
        5 => Weekday::Friday,
        7 => Weekday::Saturday,
        _ => Weekday::Sunday
    }
}

#[derive(Deserialize)]
struct MoriaResult<T> {
    result: T,
}

#[derive(Deserialize)]
struct MoriaArray<T> {
    array: Vec<T>,
}

#[derive(Deserialize)]
struct MoriaTimetableId {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
struct MoriaEventWrapper {
    event_array: Vec<MoriaEvent>,
    subject: String,
    teacher_array: Vec<MoriaTeacher>,
    #[serde(rename = "type")]
    kind: MoriaEventType,
}

#[derive(Deserialize)]
struct MoriaEvent {
    room: String,
    start_time: String,
    end_time: String,
    length: String,
    weekday: u8,
}

#[derive(Deserialize)]
struct MoriaTeacher {
    name: String,
}

#[derive(Deserialize)]
struct MoriaEventType {
    name: String,
    id: u8,
    shortcut: String,
}