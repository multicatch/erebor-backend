use crate::timetable::repository::{TimetableConsumer, TimetableProvider, TimetableId};
use std::sync::{Mutex, Arc};
use crate::timetable::{Timetable, TimetableDescriptor};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct InMemoryRepo {
    local: Arc<Mutex<TimetableRepository>>,
}

impl InMemoryRepo {
    fn new(repo: Arc<Mutex<TimetableRepository>>) -> InMemoryRepo {
        InMemoryRepo { local: repo }
    }
}

#[derive(Clone)]
struct TimetableRepository {
    timetables: HashMap<TimetableId, Timetable>,
    available: HashMap<String, HashSet<TimetableDescriptor>>,
}

impl TimetableRepository {
    pub fn new() -> TimetableRepository {
        TimetableRepository {
            timetables: HashMap::new(),
            available: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: TimetableId, timetable: Timetable) {
        let namespace = id.namespace.clone();
        let descriptor = timetable.descriptor.clone();

        self.timetables.insert(id, timetable);

        if let Some(set) = self.available.get_mut(&namespace) {
            set.insert(descriptor);
        } else {
            let mut set = HashSet::new();
            set.insert(descriptor);
            self.available.insert(namespace, set);
        }
    }

    pub fn get(&self, id: TimetableId) -> Option<&Timetable> {
        self.timetables.get(&id)
    }

    pub fn namespaces(&self) -> Vec<String> {
        self.available.keys().cloned().into_iter().collect()
    }

    pub fn available_timetables(&self, namespace: &str) -> Option<&HashSet<TimetableDescriptor>> {
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

    fn available_timetables(&self, namespace: &str) -> Option<Vec<TimetableDescriptor>> {
        let repo = self.local.lock().unwrap();
        repo.available_timetables(namespace).cloned().map(|set|
            set.into_iter().collect()
        )
    }
}

impl TimetableConsumer for InMemoryRepo {
    fn consume(&mut self, timetable: Timetable) {
        self.local.lock().unwrap().insert(timetable.descriptor.id.clone(), timetable)
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