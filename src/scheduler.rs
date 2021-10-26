use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use tokio::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::repository::TimetablePacket;

pub trait TimetableSyncScheduler {
    fn register<J>(&mut self, time: &str, job: J, tx: Sender<TimetablePacket>) -> Result<(), Box<dyn std::error::Error + '_>>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone ;
}

impl TimetableSyncScheduler for JobScheduler {
    fn register<J>(&mut self, time: &str, job: J, tx: Sender<TimetablePacket>) -> Result<(), Box<dyn Error + '_>>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone {
        register_one_shot(self, job.clone(), tx.clone());
        register_timed(self, time, job, tx)
    }
}

fn register_one_shot<'a, J>(scheduler: &'a mut JobScheduler, job: J, tx: Sender<TimetablePacket>) -> Result<(), Box<dyn Error + 'a>>
    where J: 'static,
          J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone {

    let tx = Arc::new(Mutex::new(tx));
    let tx = move || tx.lock().unwrap().clone();
    let mut job = job;
    let one_shot = Job::new_one_shot(
        Duration::from_secs(10),
        move |uuid, sched| {
            job(uuid, sched, tx());
        })?;

    scheduler.add(one_shot)
}

fn register_timed<'a, J>(scheduler: &'a mut JobScheduler, time: &str, job: J, tx: Sender<TimetablePacket>) -> Result<(), Box<dyn Error + 'a>>
    where J: 'static,
          J: FnMut(Uuid, JobScheduler, Sender<TimetablePacket>) + Send + Sync + Clone {

    let tx = Arc::new(Mutex::new(tx));
    let tx = move || tx.lock().unwrap().clone();
    let mut job = job;
    let timed = Job::new(
        time,
        move |uuid, sched| {
            job(uuid, sched, tx());
        })?;

    scheduler.add(timed)
}