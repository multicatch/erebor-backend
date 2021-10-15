use uuid::Uuid;
use tokio_cron_scheduler::JobScheduler;

pub fn sync_moria(_uuid: Uuid, _sched: JobScheduler) {
    println!("Test");
}
