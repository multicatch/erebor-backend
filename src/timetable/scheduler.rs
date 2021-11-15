use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use tokio::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;
use crate::timetable::repository::TimetablePacket;
use std::fmt::{Debug, Formatter, Display};

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
    fn register<J>(&mut self, time: &str, job: J, tx: Sender<TimetablePacket>) -> Result<(), SchedulingError>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone ;
}

impl TimetableSyncScheduler for JobScheduler {
    fn register<J>(&mut self, time: &str, job: J, tx: Sender<TimetablePacket>) -> Result<(), SchedulingError>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone {
        register_one_shot(self, job.clone(), tx.clone())?;
        register_periodic(self, time, job, tx)
    }
}

fn register_one_shot<J>(scheduler: &mut JobScheduler, job: J, tx: Sender<TimetablePacket>) -> Result<(), SchedulingError>
    where J: 'static,
          J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone {

    let tx = Arc::new(Mutex::new(tx));
    let tx = move || tx.lock().unwrap().clone();
    let mut job = job;
    let one_shot = Job::new_one_shot(
        Duration::from_secs(10),
        move |uuid, sched| {
            job(uuid, sched, tx());
        }).unwrap();

    scheduler.add(one_shot).map_err(|_| SchedulingError::OneShotErr)
}

fn register_periodic<J>(scheduler: &mut JobScheduler, time: &str, job: J, tx: Sender<TimetablePacket>) -> Result<(), SchedulingError>
    where J: 'static,
          J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone {

    let tx = Arc::new(Mutex::new(tx));
    let tx = move || tx.lock().unwrap().clone();
    let mut job = job;
    let periodic = Job::new(
        time,
        move |uuid, sched| {
            job(uuid, sched, tx());
        }).map_err(|e| SchedulingError::ScheduleErr(e))?;

    scheduler.add(periodic).map_err(|_| SchedulingError::PeriodicErr)
}