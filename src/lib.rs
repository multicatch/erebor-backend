use tokio_cron_scheduler::JobScheduler;
use crate::moria::{sync_moria};
use std::sync::mpsc::Sender;
use crate::timetable::scheduler::TimetableSyncScheduler;
use crate::timetable::repository::{TimetablePacket, TimetableRepository, listen_for_timetables};
use std::sync::{Mutex, Arc};

pub mod timetable;
mod moria;

pub fn setup_repository() -> (Arc<Mutex<TimetableRepository>>, JobScheduler) {
    let repository = TimetableRepository::new();
    let (repository, tx) = listen_for_timetables(repository);
    let mut sched = JobScheduler::new();
    register_provider_jobs(&mut sched, tx);
    (repository, sched)
}

pub fn register_provider_jobs(scheduler: &mut JobScheduler, tx: Sender<TimetablePacket>) {
    scheduler.register("0 0 0 * * * *", sync_moria, tx);
}