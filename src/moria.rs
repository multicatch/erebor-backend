use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::timetable::{Timetable, TimetableVariant, TimetableDescriptor, TimetableId};
use serde::Deserialize;

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

    timetable_ids
        .result
        .array
        .into_iter()
        .map(|id| TimetableId::new("moria".to_string(), format!("{}", id.id)))
        .for_each(|id| {
            if let Err(e) = tx.send(Timetable::new(
                TimetableDescriptor::new(
                    id,
                    "name".to_string(),
                    TimetableVariant::Unique,
                ),
                vec![]),
            ) {
                error!("Cannot send timetable [{}] to repository - MPSC error!", e.0.descriptor.id);
            }
        });
}

#[derive(Deserialize)]
struct MoriaResult<T> {
    result: T,
    status: String,
}

#[derive(Deserialize)]
struct MoriaArray<T> {
    array: Vec<T>,
    count: u64,
}

#[derive(Deserialize)]
struct MoriaTimetableId {
    id: u64,
    name: String,
}