#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;

use std::sync::mpsc::Sender;
use log::Level::Error;
use tokio_cron_scheduler::JobScheduler;

use crate::moria::sync_moria;
use crate::timetable::repository::{listen_for_timetables, TimetableProvider, TimetableConsumer};
use crate::timetable::scheduler::{TimetableSyncScheduler, SchedulingError};
use tokio::task::JoinError;
use std::process::exit;
use std::panic;
use crate::timetable::Timetable;

pub mod timetable;
mod moria;

pub fn run_scheduler<F, C, P>(repo: F) -> Result<P, SchedulingError>
    where F: FnOnce() -> (C, P),
          C: TimetableConsumer + Send + 'static,
          P: TimetableProvider + Send + Sync,
{
    let (consumer, provider) = repo();
    let sched = setup_repository(Box::new(consumer), true)?;

    tokio::spawn(async move {
        info!("Starting scheduler task...");
        match sched.start().await {
            Ok(_) => {
                info!("Scheduler task finished.")
            }
            Err(e) => finish_app_on_sched_err(e)
        };
    });

    Ok(provider)
}

fn finish_app_on_sched_err(e: JoinError) {
    if log_enabled!(Error) {
        if e.is_cancelled() {
            error!("Scheduler task was cancelled.");
        } else {
            error!("Scheduler panicked.");
            panic::resume_unwind(e.into_panic());
        };
    }
    exit(255);
}

pub fn setup_repository<C>(consumer: Box<C>, exit_on_failure: bool) -> Result<JobScheduler, SchedulingError>
    where C: TimetableConsumer + Send + 'static,
{
    let tx = listen_for_timetables(consumer, exit_on_failure);
    let mut sched = JobScheduler::new();
    register_provider_jobs(&mut sched, tx)?;
    info!("Repository setup finished.");
    Ok(sched)
}

pub fn register_provider_jobs(scheduler: &mut JobScheduler, tx: Sender<Timetable>) -> Result<(), SchedulingError> {
    debug!("Registering timetable providers...");
    scheduler.register("moria", "0 0 0 * * * *", sync_moria, tx)
}
