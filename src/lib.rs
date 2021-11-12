use std::sync::mpsc::Sender;

use tokio_cron_scheduler::JobScheduler;

use crate::moria::sync_moria;
use crate::timetable::repository::{listen_for_timetables, TimetablePacket};
use crate::timetable::repository::inmemory::{in_memory_repo, InMemoryRepo};
use crate::timetable::scheduler::TimetableSyncScheduler;

pub mod timetable;
mod moria;

pub fn setup_repository() -> (InMemoryRepo, JobScheduler) {
    //let repository = TimetableRepository::new();
    let (consumer, provider) = in_memory_repo();
    let tx = listen_for_timetables(Box::new(consumer));
    let mut sched = JobScheduler::new();
    register_provider_jobs(&mut sched, tx);
    (provider, sched)
}

pub fn register_provider_jobs(scheduler: &mut JobScheduler, tx: Sender<TimetablePacket>) -> Result<(), Box<dyn std::error::Error + '_>> {
    scheduler.register("0 0 0 * * * *", sync_moria, tx)
}
