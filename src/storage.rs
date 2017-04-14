use std::sync::Mutex;
use std::thread;

use fnv::FnvHashMap;
use x11_dl::xlib;

use raw;
use {TriggerOn, HotKey};

lazy_static! {
    pub static ref STORAGE: CallbackStorage = CallbackStorage::new();
}

pub type Callback = fn() -> ();
type KeyMapStorage = Mutex<FnvHashMap<xlib::KeySym, Vec<Callback>>>;


pub struct CallbackStorage {
    released_storage: KeyMapStorage,
    pressed_storage: KeyMapStorage,
    pub display: *mut xlib::Display,
    root: xlib::Window,
}

impl CallbackStorage {
    fn new() -> Self {
        let display = raw::get_display();
        let root = raw::get_root(display);

        CallbackStorage {
            released_storage: Mutex::new(FnvHashMap::default()),
            pressed_storage: Mutex::new(FnvHashMap::default()),
            display: display,
            root: root,
        }
    }

    pub fn add(&self, key: &HotKey, callback: Callback) {
        use std::collections::hash_map::Entry;
        // select storage
        let mut storage;
        if key.trigger == TriggerOn::Press {
            storage = self.pressed_storage.lock().unwrap();
        } else {
            storage = self.released_storage.lock().unwrap();
        }

        let entry = storage.entry(key.key);
        if let Entry::Vacant(_) = entry {
            // if first registered callback grab this key.
            raw::grab(self.display, self.root, &key);
        }
        // add callback
        entry.or_insert(Vec::new()).push(callback);
    }

    pub fn remove_all(&self, key: &HotKey) {
        // select storage
        let mut storage;
        if key.trigger == TriggerOn::Press {
            storage = self.pressed_storage.lock().unwrap();
        } else {
            storage = self.released_storage.lock().unwrap();
        }

        if let Some(_) = storage.remove(&key.key) {
            // ungrab key if it was present
            raw::ugrab(self.display, self.root, key);
        }
    }

    #[inline]
    pub fn dispatch(&self, event: &mut xlib::XEvent) {
        let event_type = event.get_type();

        if event_type == xlib::KeyPress {
            self.trigger_press(event);
        } else if event_type == xlib::KeyRelease {
            self.trigger_release(event);
        }
    }

    #[inline]
    fn trigger_press(&self, event: &mut xlib::XEvent) {
        let keysym = raw::get_keysym(event.as_mut());

        if let Some(callbacks) = self.pressed_storage.lock().unwrap().get_mut(&keysym) {
            for callback in callbacks {
                let callback = callback.to_owned();
                thread::spawn(move || { callback(); });
            }
        }
    }

    #[inline]
    fn trigger_release(&self, event: &mut xlib::XEvent) {
        let keysym = raw::get_keysym(event.as_mut());

        if let Some(callbacks) = self.released_storage.lock().unwrap().get_mut(&keysym) {
            for callback in callbacks {
                let callback = callback.to_owned();
                thread::spawn(move || { callback(); });
            }
        }
    }
}


unsafe impl Send for CallbackStorage {}
unsafe impl Sync for CallbackStorage {}
