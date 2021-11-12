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

    let api = path!("timetable" / String)
        .map(move |id: String| {
            let timetable = repository.get(TimetableId(id));
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

    warp::serve(api)
        .run(([0, 0, 0, 0], 3030))
        .await;
}