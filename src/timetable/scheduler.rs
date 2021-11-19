use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use tokio::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;
use std::fmt::{Debug, Formatter, Display};
use crate::timetable::Timetable;

#[derive(Debug)]
pub enum SchedulingError {
    OneShotErr,
    PeriodicErr,
    ScheduleErr(Box<dyn std::error::Error>),
}

impl Display for SchedulingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SchedulingError {}

pub trait TimetableSyncScheduler {
    fn register<J>(&mut self, name: &str, time: &str, job: J, tx: Sender<Timetable>) -> Result<(), SchedulingError>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler, Sender<Timetable>) + Send + Sync + Clone ;
}

impl TimetableSyncScheduler for JobScheduler {
    fn register<J>(&mut self, name: &str, time: &str, job: J, tx: Sender<Timetable>) -> Result<(), SchedulingError>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler, Sender<Timetable>) + Send + Sync + Clone {
        trace!("Registering a job named [{}]...", name);
        register_one_shot(self, name, job.clone(), tx.clone())?;
        register_periodic(self, name, time, job, tx)?;
        info!("Job [{}] registered successfully. Scheduled to run at [{}].", name, time);
        Ok(())
    }
}

fn register_one_shot<J>(scheduler: &mut JobScheduler, name: &str, job: J, tx: Sender<Timetable>) -> Result<(), SchedulingError>
    where J: 'static,
          J: FnMut(Uuid, JobScheduler, Sender<Timetable>) + Send + Sync + Clone {

    let tx = Arc::new(Mutex::new(tx));
    let tx = move || tx.lock().unwrap().clone();
    let mut job = job;
    let secs = 10;
    let one_shot = Job::new_one_shot(
        Duration::from_secs(secs),
        move |uuid, sched| {
            job(uuid, sched, tx());
        }).unwrap();

    debug!("Job [{}] will run in [{}] seconds.", name, secs);
    scheduler.add(one_shot).map_err(|_| SchedulingError::OneShotErr)
}

fn register_periodic<J>(scheduler: &mut JobScheduler, name: &str, time: &str, job: J, tx: Sender<Timetable>) -> Result<(), SchedulingError>
    where J: 'static,
          J: FnMut(Uuid, JobScheduler, Sender<Timetable>) + Send + Sync + Clone {

    let tx = Arc::new(Mutex::new(tx));
    let tx = move || tx.lock().unwrap().clone();
    let mut job = job;
    let periodic = Job::new(
        time,
        move |uuid, sched| {
            job(uuid, sched, tx());
        }).map_err(|e| SchedulingError::ScheduleErr(e))?;

    debug!("Job [{}] will run at [{}].", name, time);
    scheduler.add(periodic).map_err(|_| SchedulingError::PeriodicErr)
}