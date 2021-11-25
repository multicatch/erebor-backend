use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::timetable::{Timetable, TimetableVariant, TimetableDescriptor, TimetableId, Activity, ActivityGroup, ActivityOccurrence, Weekday, ActivityTime};
use serde::Deserialize;
use std::collections::HashMap;
use chrono::Utc;
use tokio::time::Duration;
use crate::httpclient::{HttpClient, HttpClientError};

struct MoriaClient {
    base_address: String,
    client: HttpClient,
}

impl MoriaClient {
    pub fn new() -> MoriaClient {
        MoriaClient {
            base_address: "http://moria.umcs.lublin.pl/api".to_string(),
            client: HttpClient::new(5, Duration::from_millis(300))
        }
    }

    pub async fn fetch_timetable_list(&self) -> Result<MoriaResult<MoriaArray<MoriaTimetableId>>, HttpClientError> {
        debug!("Fetching available moria timetables");
        let url = format!("{}/students_list", self.base_address);

        self.client.make_retry_request(
            url.clone(),
            |client| client.get(url.clone()),
        ).await
    }

    pub async fn fetch_activities(&self, id: &str) -> Result<MoriaResult<MoriaArray<MoriaEventWrapper>>, HttpClientError> {
        debug!("Fetching activities for {}", id);
        let url = format!("{}/activity_list_for_students", self.base_address);

        let mut params = HashMap::new();
        params.insert("id", id);

        self.client.make_retry_request(
            url.clone(),
            |client| client
                .get(format!("{}/activity_list_for_students", self.base_address))
                .json(&params),
        ).await
    }
}

pub fn sync_moria(_uuid: Uuid, _sched: JobScheduler, tx: Sender<Timetable>) {
    tokio::spawn(async {
        if let Err(e) = fetch_timetables(tx).await {
            error!("Moria sync task was aborted due to an error. Description: {}", e)
        };
    });
}

async fn fetch_timetables(tx: Sender<Timetable>) -> Result<(), HttpClientError> {
    trace!("Creating moria client...");
    let client = MoriaClient::new();
    trace!("Fetching timetable list...");
    let timetable_ids: MoriaResult<MoriaArray<MoriaTimetableId>> = client.fetch_timetable_list().await?;

    let ids: Vec<_> = timetable_ids
        .result
        .array
        .into_iter()
        .map(|id|
            (TimetableId::new("moria".to_string(), format!("{}", id.id)), id.name)
        )
        .collect();

    let mut sent_timetables = 0;

    for (id, name) in ids {
        let id_str = id.id.clone();
        trace!("Fetching activities for [{}]", id);
        let activities = fetch_activities(&client, &id_str).await?;

        if activities.is_empty() {
            info!("Moria timetable [{}]: Ignoring, there are no activities.", id_str);
        } else {
            debug!("Moria timetable [{}]: Sending to repository...", id_str);

            if send_timetable(&tx, id, name, activities) {
                sent_timetables += 1;
            }
        }
    }

    info!("Successfully sent {} timetables to repository.", sent_timetables);

    Ok(())
}

async fn fetch_activities(client: &MoriaClient, id: &str) -> Result<Vec<Activity>, HttpClientError> {
    let moria_activities: MoriaResult<MoriaArray<MoriaEventWrapper>> = client.fetch_activities(id).await?;

    let activities: Vec<Activity> = moria_activities
        .result
        .array
        .into_iter()
        .filter(|wrapper| {
            let empty_array = wrapper.event_array.is_empty();
            if empty_array {
                warn!("Moria timetable [{}]: Activity [{}] has empty event array and will be ignored.", id, wrapper.id);
            }

            let empty = vec![];
            let empty_students = wrapper.students_array.as_ref()
                .unwrap_or_else(|| &empty)
                .is_empty();
            if empty_students {
                warn!("Moria timetable [{}]: Activity [{}] has empty student list and will be ignored.", id, wrapper.id);
            }

            !empty_array && !empty_students
        })
        .map(|wrapper| {
            let event = wrapper.event_array.get(0).unwrap();
            let teacher = wrapper.teacher_array.get(0)
                .map(|t| t.name.clone());

            let id_num: u32 = id.parse().unwrap_or(0);
            let student_array = wrapper.students_array
                .as_ref()
                .unwrap();
            let group = student_array
                .iter()
                .find(|s| s.id == id_num && s.groups != "1")
                .map(|s| s.group.clone());

            to_activity(&wrapper, event, teacher, group)
        })
        .collect();

    Ok(activities)
}

fn to_activity(wrapper: &MoriaEventWrapper, event: &MoriaEvent, teacher: Option<String>, group: Option<String>) -> Activity {
    Activity {
        id: wrapper.id.to_string(),
        name: wrapper.subject.clone(),
        teacher,
        occurrence: ActivityOccurrence::Regular {
            weekday: to_weekday(event.weekday),
        },
        group: ActivityGroup {
            symbol: wrapper.kind.shortcut.clone(),
            name: wrapper.kind.name.clone(),
            id: wrapper.kind.id,
            number: group.clone(),
        },
        time: ActivityTime {
            start_time: event.start_time.clone(),
            end_time: event.end_time.clone(),
            duration: event.length.clone(),
        },
        room: Some(event.room.clone()),
    }
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

fn send_timetable(tx: &Sender<Timetable>, id: TimetableId, name: String, activities: Vec<Activity>) -> bool {
    let (name, variant) = parse_variant(name);

    let timetable = Timetable::new(
        TimetableDescriptor::new(id, name, variant),
        activities,
        Utc::now(),
    );

    let result = tx.send(timetable);

    if let Err(e) = &result {
        error!("Cannot send timetable [{}] to repository - MPSC error!", e.0.descriptor.id);
    }

    result.is_ok()
}

fn parse_variant(name: String) -> (String, TimetableVariant) {
    let name = name.trim();

    if name.chars()
        .nth(1)
        .map(|c| c == ' ')
        .unwrap_or(false) {
        let year = name.chars()
            .nth(0)
            .unwrap()
            .to_digit(10);

        if let Some(year) = year {
            return (name[2..].trim().to_string(), TimetableVariant::Year(year));
        }
    }
    (name.to_string(), TimetableVariant::Unique)
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
    id: u32,
    event_array: Vec<MoriaEvent>,
    subject: String,
    teacher_array: Vec<MoriaTeacher>,
    students_array: Option<Vec<MoriaStudentGroup>>,
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

#[derive(Deserialize)]
struct MoriaStudentGroup {
    id: u32,
    group: String,
    groups: String,
}