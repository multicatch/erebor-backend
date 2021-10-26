use tokio_cron_scheduler::JobScheduler;
use crate::scheduler::TimetableSyncScheduler;
use crate::moria::{sync_moria};
use crate::repository::TimetablePacket;
use std::sync::mpsc::Sender;

pub mod timetable;
mod moria;
mod scheduler;
pub mod repository;

pub fn register_provider_jobs(scheduler: &mut JobScheduler, tx: Sender<TimetablePacket>) {
    scheduler.register("0 0 0 * * * *", sync_moria, tx);
}