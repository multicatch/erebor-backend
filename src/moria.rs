use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::timetable::{Timetable, TimetableVariant, TimetableDescriptor, TimetableId, Activity, ActivityGroup, ActivityOccurrence, Weekday, ActivityTime};
use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;
use serde::de::DeserializeOwned;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
enum MoriaError {
    RequestError(reqwest::Error),
    DeserializationError(serde_json::Error),
}

impl Display for MoriaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MoriaError::RequestError(e) => write!(f, "Request error. {}", e),
            MoriaError::DeserializationError(e) => write!(f, "Deserialization error. {}", e),
        }
    }
}

struct MoriaClient {
    base_address: String,
    client: reqwest::Client,
}

impl MoriaClient {
    pub fn new() -> MoriaClient {
        MoriaClient {
            base_address: "http://moria.umcs.lublin.pl/api".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_timetable_list(&self) -> Result<MoriaResult<MoriaArray<MoriaTimetableId>>, MoriaError> {
        self.make_request(async {
            self.client.get(format!("{}/students_list", self.base_address))
                .send()
                .await
        }).await
    }

    pub async fn fetch_activities(&self, id: &str) -> Result<MoriaResult<MoriaArray<MoriaEventWrapper>>, MoriaError> {
        let mut params = HashMap::new();
        params.insert("id", id);

        self.make_request(async {
            self.client.get("http://moria.umcs.lublin.pl/api/activity_list_for_students")
                .json(&params)
                .send()
                .await
        }).await
    }

    async fn make_request<F, T>(&self, request: F) -> Result<T, MoriaError>
        where F: Future<Output=Result<reqwest::Response, reqwest::Error>>,
              T: DeserializeOwned {
        let result = request.await
            .map_err(|e| {
                error!("Cannot make request. {}", e);
                MoriaError::RequestError(e)
            })?
            .text()
            .await
            .map_err(|e| {
                error!("Cannot read response. {}", e);
                MoriaError::RequestError(e)
            })?;

        serde_json::from_str(&result)
            .map_err(|e| {
                error!("Cannot deserialize response. {}", e);
                MoriaError::DeserializationError(e)
            })
    }
}

pub fn sync_moria(_uuid: Uuid, _sched: JobScheduler, tx: Sender<Timetable>) {
    tokio::spawn(async {
        if let Err(e) = fetch_timetables(tx).await {
            error!("Moria sync task was aborted due to an error. Description: {}", e)
        };
    });
}

async fn fetch_timetables(tx: Sender<Timetable>) -> Result<(), MoriaError> {
    let client = MoriaClient::new();
    let timetable_ids: MoriaResult<MoriaArray<MoriaTimetableId>> = client.fetch_timetable_list().await?;

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
            fetch_activities(&client, &id_str).await?);
        if let Err(e) = tx.send(timetable) {
            error!("Cannot send timetable [{}] to repository - MPSC error!", e.0.descriptor.id);
        }
    }

    Ok(())
}

async fn fetch_activities(client: &MoriaClient, id: &str) -> Result<Vec<Activity>, MoriaError> {
    let moria_activities: MoriaResult<MoriaArray<MoriaEventWrapper>> = client.fetch_activities(id).await?;

    let result: Vec<Activity> = moria_activities
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
                room: Some(event.room.clone()),
            }
        })
        .collect();

    Ok(result)
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