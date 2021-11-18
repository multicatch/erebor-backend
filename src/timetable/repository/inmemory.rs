use crate::timetable::repository::{TimetableConsumer, TimetableProvider, TimetableId};
use std::sync::{Mutex, Arc};
use crate::timetable::Timetable;
use std::collections::HashMap;

#[derive(Clone)]
pub struct InMemoryRepo {
    local: Arc<Mutex<TimetableRepository>>
}

impl InMemoryRepo {
    fn new(repo: Arc<Mutex<TimetableRepository>>) -> InMemoryRepo {
        InMemoryRepo { local: repo }
    }
}

#[derive(Clone)]
struct TimetableRepository {
    timetables: HashMap<TimetableId, Timetable>,
    available: HashMap<String, Vec<TimetableId>>,
}

impl TimetableRepository {
    pub fn new() -> TimetableRepository {
        TimetableRepository {
            timetables: HashMap::new(),
            available: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: TimetableId, timetable: Timetable) {
        self.timetables.insert(id.clone(), timetable);
        let available = self.available.get(&id.namespace)
            .map(|v| v.clone())
            .unwrap_or_else(|| vec![id.clone()]);
        self.available.insert(id.namespace.clone(), available);
    }

    pub fn get(&self, id: TimetableId) -> Option<&Timetable> {
        self.timetables.get(&id)
    }

    pub fn namespaces(&self) -> Vec<String> {
        self.available.keys().cloned().into_iter().collect()
    }

    pub fn available_timetables(&self, namespace: &str) -> Option<&Vec<TimetableId>> {
        self.available.get(namespace)
    }
}

impl Default for TimetableRepository {
    fn default() -> Self {
        Self::new()
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