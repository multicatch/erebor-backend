use tokio_cron_scheduler::JobScheduler;
use crate::scheduler::TimetableSyncScheduler;
use crate::moria::sync_moria;

pub mod timetable;
mod moria;
mod scheduler;

pub fn register_provider_jobs(scheduler: &mut JobScheduler) {
    scheduler.register("0 0 0 * * * *", sync_moria);
}