use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::repository::{TimetableId, TimetablePacket};
use crate::timetable::Timetable;

pub fn sync_moria(_uuid: Uuid, _sched: JobScheduler, tx: Sender<TimetablePacket>) {
    tx.send(TimetablePacket(
        TimetableId("id".to_string()),
        Timetable::new("name".to_string(), vec![],
        )));
}
