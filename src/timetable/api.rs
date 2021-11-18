use crate::timetable::repository::{TimetableId, TimetableProvider, ShareableTimetableProvider};
use rocket::State;
use rocket::response::{status, content};
use rocket::http::Status;
use serde::Serialize;
use log::Level;

#[get("/timetable")]
pub fn get_all_namespaces(repo: &State<ShareableTimetableProvider>) -> status::Custom<content::Json<String>> {
    serialize_response(
        repo.namespaces(),
        || format!("available namespaces")
    )
}

#[get("/timetable/<namespace>")]
pub fn get_all_timetables(repo: &State<ShareableTimetableProvider>, namespace: &str) -> status::Custom<content::Json<String>> {
    repo.available_timetables(namespace)
        .map(|value|
                 serialize_response(value, || format!("timetables in [{}]", namespace))
        )
        .unwrap_or_else(||
            status::Custom(Status::NotFound, content::Json("{}".to_string()))
        )
}

#[get("/timetable/<namespace>/<id>")]
pub fn get_timetable(repo: &State<ShareableTimetableProvider>, namespace: &str, id: &str) -> status::Custom<content::Json<String>> {
    let timetable = repo.get(
        TimetableId::new(namespace.to_string(), id.to_string())
    );

    timetable
        .map(|value|
            serialize_response(value, || format!("timetable [{}:{}]", namespace, id))
        )
        .unwrap_or_else(||
            status::Custom(Status::NotFound, content::Json("{}".to_string()))
        )
}

fn serialize_response<T, F>(value: T, error_description: F) -> status::Custom<content::Json<String>>
    where T: Serialize,
          F: FnOnce() -> String,
{
    serde_json::to_string(&value)
        .map(|json|
            status::Custom(Status::Ok, content::Json(json))
        )
        .unwrap_or_else(|e| {
            if log_enabled!(Level::Error) {
                error!("Cannot serialize {}: {}", error_description(), e);
            }
            status::Custom(Status::InternalServerError, content::Json("{}".to_string()))
        })
}