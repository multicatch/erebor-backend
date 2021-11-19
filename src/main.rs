use erebor_backend::run_scheduler;
use erebor_backend::timetable::repository::{ShareableTimetableProvider};
use log::LevelFilter;
use erebor_backend::timetable::repository::inmemory::in_memory_repo;
use erebor_backend::timetable::api::{get_all_namespaces, get_all_timetables, get_timetable};
use rocket::routes;

#[rocket::main]
async fn main() {
    env_logger::builder()
        .filter_module("erebor_backend", LevelFilter::Trace)
        .filter_module("rocket", LevelFilter::Info)
        .filter_module("reqwest", LevelFilter::Info)
        .init();

    let repository = run_scheduler(in_memory_repo).unwrap();

    rocket::build()
        .manage(ShareableTimetableProvider::new(repository))
        .mount("/", routes![get_all_namespaces, get_all_timetables, get_timetable])
        .launch()
        .await;
}