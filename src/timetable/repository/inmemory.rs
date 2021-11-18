use crate::timetable::repository::{TimetableConsumer, TimetableProvider, TimetableRepository, TimetableId};
use std::sync::{Mutex, Arc};
use crate::timetable::Timetable;

#[derive(Clone)]
pub struct InMemoryRepo {
    local: Arc<Mutex<TimetableRepository>>
}

impl InMemoryRepo {
    pub fn new(repo: Arc<Mutex<TimetableRepository>>) -> InMemoryRepo {
        InMemoryRepo { local: repo }
    }
}

impl TimetableProvider for InMemoryRepo {
    fn get(&self, id: TimetableId) -> Option<Timetable> {
        let guard = self.local.lock().unwrap();
        guard.get(id).cloned()
    }

    fn namespaces(&self) -> Vec<String> {
        let repo = self.local.lock().unwrap();
        repo.namespaces()
    }

    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableId>> {
        let repo = self.local.lock().unwrap();
        repo.available_timetables(namespace).cloned()
    }
}

impl TimetableConsumer for InMemoryRepo {
    fn consume(&mut self, id: TimetableId, timetable: Timetable) {
        self.local.lock().unwrap().insert(id, timetable)
    }
}

pub fn in_memory_repo() -> (InMemoryRepo, InMemoryRepo) {
    debug!("Creating in memory repository for timetable.");
    let repository = TimetableRepository::new();
    let repo_arc = Arc::new(Mutex::new(repository));
    let consumer = InMemoryRepo::new(repo_arc.clone());
    let provider = InMemoryRepo::new(repo_arc);
    (consumer, provider)
}