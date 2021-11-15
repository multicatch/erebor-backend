use erebor_backend::run_scheduler;
use erebor_backend::timetable::repository::{TimetableId, TimetableProvider};
use warp::{path, Filter};
use warp::http::StatusCode;
use log::LevelFilter;
use erebor_backend::timetable::repository::inmemory::in_memory_repo;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_module("erebor_backend", LevelFilter::Trace).init();

    let repository = run_scheduler(in_memory_repo).unwrap();

    let timetable_repo = repository.clone();
    let timetable = path!("timetable" / String)
        .map(move |id: String| {
            let timetable = timetable_repo.get(TimetableId(id));
            match timetable {
                Some(value) => warp::reply::with_status(
                    serde_json::to_string(&value).unwrap(),
                    StatusCode::OK
                ),
                _ => warp::reply::with_status(
                    "".to_string(),
                    StatusCode::NOT_FOUND
                ),
            }
        });

    let all_timetables = path!("timetable")
        .map(move || {
            let timetables = repository.available();
            warp::reply::with_status(
                serde_json::to_string(&timetables).unwrap(),
                StatusCode::OK
            )
    });

    warp::serve(timetable.or(all_timetables))
        .run(([0, 0, 0, 0], 3030))
        .await;
}