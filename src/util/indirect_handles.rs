use std::{collections::HashMap, rc::{Rc, Weak}};

// ID should be a unit type (serving as a name for this type of handle)
#[derive(Debug, Clone)]
pub struct Handle<ID: Default + Clone>(BaseID, Rc<ID>);
#[derive(Debug, Clone)]
pub struct WeakHandle<ID: Default + Clone>(BaseID, ID);
pub type HandleTrackerObj<ID> = Weak<ID>;

pub struct HandleTracker<ID, T> where ID: Eq + Default {
    id_counter: BaseID,

    // TODO: use a better structure than HashMap
    // We will be iterating over it a lot, and our keys are numbers from 0 to X.
    // There should be some data structure that lets us prune at O(1) and get()
    // with a lower O(1) constant
    handles: HashMap<BaseID, HandleEntry<ID, T>>,
}

type BaseID = u32;
struct HandleEntry<ID, T> {
    user_data: T,
    tracker: HandleTrackerObj<ID>,
}

impl<ID: Default + Clone> Handle<ID> {
    pub fn make_weak(&self) -> WeakHandle<ID> {
        WeakHandle(self.0, Default::default())
    }
}

impl<ID, T> HandleTracker<ID, T> where ID: Eq + Default + Clone {
    pub fn new() -> HandleTracker<ID, T> {
        Self {
            handles: HashMap::new(),
            id_counter: 0,
        }
    }

    pub fn put(&mut self, user_data: T) -> Handle<ID> {
        let base_id = self.id_counter;
        self.id_counter += 1;
        let rc: Rc<ID> = Rc::new(Default::default());
        let tracker = Rc::downgrade(&rc);
        let entry = HandleEntry { user_data, tracker };
        let empty = self.handles.insert(base_id, entry);
        if empty.is_some() {
            unreachable!("Duplicate BaseID used");
        }
        return Handle(base_id, rc);
    }

    pub fn get<'a, H>(&self, handle: H) -> Option<&T>
            where H: Into<&'a WeakHandle<ID>>, ID: 'a {
        let weak_handle: &WeakHandle<ID> = handle.into();
        return self.handles.get(&weak_handle.0).map(|data| &data.user_data);
    }

    pub fn get_mut<'a, H>(&mut self, handle: H) -> Option<&mut T>
            where H: Into<&'a WeakHandle<ID>>, ID: 'a {
        let weak_handle: &WeakHandle<ID> = handle.into();
        return self.handles.get_mut(&weak_handle.0).map(|data| &mut data.user_data);
    }

    pub fn prune(&mut self) {
        for key in 0..self.id_counter {
            if let Some(HandleEntry { user_data: _, tracker }) = self.handles.get(&key) {
                if tracker.strong_count() == 0 {
                    self.handles.remove(&key);
                }
            }
        }
    }
}

impl<ID, T> Default for HandleTracker<ID, T> where ID: Eq + Default + Clone {
    fn default() -> Self {
        Self::new()
    }
}

impl<ID: Eq + Default + Clone> From<Handle<ID>> for WeakHandle<ID> {
    fn from(Handle(id, _): Handle<ID>) -> Self {
        Self(id, Default::default())
    }
}

impl<ID: Eq + Default + Clone> From<&Handle<ID>> for WeakHandle<ID> {
    fn from(Handle(id, _): &Handle<ID>) -> Self {
        Self(*id, Default::default())
    }
}
