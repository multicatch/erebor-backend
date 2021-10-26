use tokio_cron_scheduler::JobScheduler;
use erebor_backend::register_provider_jobs;
use erebor_backend::repository::{TimetableRepository, listen_for_timetables};

#[tokio::main]
async fn main() {
    let repository = TimetableRepository::new();
    let (repository, tx) = listen_for_timetables(repository);
    let mut sched = JobScheduler::new();
    register_provider_jobs(&mut sched, tx);
    sched.start().await;
}