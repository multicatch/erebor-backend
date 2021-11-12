use erebor_backend::{setup_repository};
use erebor_backend::timetable::repository::{TimetableId, TimetableProvider};
use warp::{path, Filter};
use warp::http::StatusCode;

#[tokio::main]
async fn main() {
    let (repository, sched) = setup_repository();

    tokio::spawn(async move {
        sched.start().await;
    });

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