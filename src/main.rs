use tokio_cron_scheduler::JobScheduler;
use erebor_backend::register_provider_jobs;

#[tokio::main]
async fn main() {
    let mut sched = JobScheduler::new();
    register_provider_jobs(&mut sched);
    sched.start().await;
}