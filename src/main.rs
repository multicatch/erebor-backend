use tokio_cron_scheduler::JobScheduler;
use erebor_backend::{register_provider_jobs, setup_repository};
use erebor_backend::timetable::repository::{TimetableRepository, listen_for_timetables, TimetableId};
use erebor_backend::timetable::api::timetable_api;
use std::thread;
use warp::{path, Filter};
use warp::http::StatusCode;
use std::sync::{Arc, Mutex};
use tokio::task;

#[tokio::main]
async fn main() {
    let (repo, sched) = setup_repository();

    tokio::spawn(async move {
        sched.start().await;
    });

    let api = path!("timetable" / String)
        .map(move |id: String| {
            let repository = repo.lock().unwrap();
            let timetable = repository.get(TimetableId(id));
            match timetable {
                Some(value) => warp::reply::with_status(serde_json::to_string(value).unwrap(), StatusCode::OK),
                _ => warp::reply::with_status("".to_string(), StatusCode::NOT_FOUND),
            }
        });

    warp::serve(api)
        .run(([0, 0, 0, 0], 3030))
        .await;
}