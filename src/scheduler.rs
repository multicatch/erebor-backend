use tokio_cron_scheduler::{JobScheduler, Job};
use uuid::Uuid;
use std::error::Error;
use tokio::time::Duration;

pub trait TimetableSyncScheduler {
    fn register<J>(&mut self, time: &str, job: J) -> Result<(), Box<dyn std::error::Error + '_>>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler) + Send + Sync + Clone;
}

impl TimetableSyncScheduler for JobScheduler {
    fn register<J>(&mut self, time: &str, job: J) -> Result<(), Box<dyn Error + '_>>
        where J: 'static,
              J: FnMut(Uuid, JobScheduler) + Send + Sync + Clone {
        let mut one_shot_job = job.clone();
        let mut timed_job = job;

        let one_shot = Job::new_one_shot(
            Duration::from_secs(10),
            move |uuid, sched| {
                one_shot_job(uuid, sched);
            })?;


        let timed = Job::new(time, move |uuid, sched| {
            timed_job(uuid, sched);
        })?;

        self.add(one_shot);
        self.add(timed)
    }
}