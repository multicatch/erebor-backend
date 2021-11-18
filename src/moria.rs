use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::timetable::Timetable;
use crate::timetable::repository::{TimetablePacket, TimetableId};

pub fn sync_moria(_uuid: Uuid, _sched: JobScheduler, tx: Sender<TimetablePacket>) {
    tx.send(TimetablePacket(
        TimetableId::new("moria".to_string(), "id".to_string()),
        Timetable::new("name".to_string(), vec![])
    ));
}
