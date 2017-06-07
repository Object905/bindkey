///! This is a very simple keybinder to functions using xlib.
///! For now it does not support binding same key with different modifiers
///! Callbacks used as simple functions, to execute it in paralell.
extern crate x11;
extern crate fnv;

use std::sync::RwLock;
use std::thread;

use fnv::FnvHashMap;
use x11::xlib;

mod raw;
use raw::next_event;

/// X11 keysyms, used as keycodes
/// Prefer lowercase letters
pub use x11::keysym;

/// Start grabbing keys and executing callbacks
pub fn start(mut storage: CallbackStorage) {
    let mut event = xlib::XEvent { pad: [0; 24] };
    loop {
        unsafe {
            next_event(storage.get_display_mut(), &mut event);
        }
        storage.dispatch(&mut event);
    }
}

/// Start, grabbing keys and executing callbacks in separate thread, returns `JoinHandle` for this thread.
/// Shortcut for `std::thread::spawn(|| bindkey::start())`.
pub fn start_async(storage: CallbackStorage) -> thread::JoinHandle<()> {
    thread::spawn(move || start(storage))
}

#[repr(u32)]
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Modifier {
    Shift = xlib::ShiftMask,
    CapsLock = xlib::LockMask,
    Ctrl = xlib::ControlMask,
    Alt = xlib::Mod1Mask,
    NumLock = xlib::Mod2Mask,
    ScrollLock = xlib::Mod3Mask,
    Window = xlib::Mod4Mask,
    Mod5 = xlib::Mod5Mask,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TriggerOn {
    Press,
    Release,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct HotKey {
    pub key: xlib::KeySym,
    pub modifiers: Vec<Modifier>,
    pub trigger: TriggerOn,
}

impl HotKey {
    pub fn new(key: u32, mut modifiers: Vec<Modifier>, trigger: TriggerOn) -> Self {
        modifiers.sort();
        modifiers.dedup();

        HotKey {
            key: key as xlib::KeySym,
            modifiers: modifiers,
            trigger: trigger,
        }
    }
}


pub type Callback = fn() -> ();
type KeyMapStorage = RwLock<FnvHashMap<xlib::KeySym, Vec<Callback>>>;

#[derive(Debug)]
pub struct CallbackStorage {
    released_storage: KeyMapStorage,
    pressed_storage: KeyMapStorage,
    display: *mut xlib::Display,
    root: xlib::Window,
}

impl Default for CallbackStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackStorage {
    pub fn new() -> Self {
        unsafe {
            let display = raw::get_display();
            let root = raw::get_root(display);

            CallbackStorage {
                released_storage: RwLock::new(FnvHashMap::default()),
                pressed_storage: RwLock::new(FnvHashMap::default()),
                display: display,
                root: root,
            }
        }
    }

    /// Add callback to execute on new thread when key is pressed.
    pub fn add(&mut self, key: &HotKey, callback: Callback) {
        use std::collections::hash_map::Entry;
        // select storage
        let mut storage = if key.trigger == TriggerOn::Press {
            self.pressed_storage.write().unwrap()
        } else {
            self.released_storage.write().unwrap()
        };

        let entry = storage.entry(key.key);
        if let Entry::Vacant(_) = entry {
            // if first registered callback grab this key.
            unsafe {
                raw::grab(self.display, self.root, key);
            }
        }
        // add callback
        entry.or_insert_with(Vec::new).push(callback);
    }

    /// Remove all callbacks atached to this key.
    pub fn remove_all(&mut self, key: &HotKey) {
        // select storage
        let mut storage = if key.trigger == TriggerOn::Press {
            self.pressed_storage.write().unwrap()
        } else {
            self.released_storage.write().unwrap()
        };

        if storage.remove(&key.key).is_some() {
            // ungrab key if it was present
            raw::ugrab(self.display, self.root, key);
        }
    }

    #[inline]
    fn dispatch(&self, event: &mut xlib::XEvent) {
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

        if let Some(callbacks) = self.pressed_storage.read().unwrap().get(&keysym) {
            for callback in callbacks {
                let callback = callback.to_owned();
                thread::spawn(move || { callback(); });
            }
        }
    }

    #[inline]
    fn trigger_release(&self, event: &mut xlib::XEvent) {
        let keysym = raw::get_keysym(event.as_mut());

        if let Some(callbacks) = self.released_storage.read().unwrap().get(&keysym) {
            for callback in callbacks {
                let callback = callback.to_owned();
                thread::spawn(move || { callback(); });
            }
        }
    }

    unsafe fn get_display_mut(&mut self) -> *mut xlib::Display {
        self.display
    }
}

unsafe impl Send for CallbackStorage {}
