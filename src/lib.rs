///! This is a very simple keybinder to functions using xlib.
///! For now it does not support binding same key with different modifiers
///! Callbacks used as simple functions, to execute it in paralell.
#[macro_use]
extern crate lazy_static;
extern crate x11_dl;
extern crate fnv;

use std::os::raw::c_uint;
use std::thread;

use x11_dl::xlib;

mod storage;
mod raw;

use storage::{STORAGE, Callback};
use raw::next_event;

/// X11 keysyms, used as keycodes
/// Use lowercase letters
pub use x11_dl::keysym;


/// Start grabbing keys and executing callbacks
pub fn start() {
    let mut event = xlib::XEvent { pad: [0; 24] };
    loop {
        next_event(&mut event);
        STORAGE.dispatch(&mut event);
    }
}

/// Start, grabbing keys and executing callbacks in separate thread, returns `JoinHandle` for this thread.
/// Shortcut for `std::thread::spawn(|| bindkey::start())`.
pub fn start_async() -> thread::JoinHandle<()> {
    thread::spawn(|| start())
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Modifier {
    Shift,
    CapsLock,
    Ctrl,
    Alt,
    NumLock,
    ScrollLock,
    Window,
    Mod5,
}

impl Modifier {
    fn mask(&self) -> c_uint {
        match *self {
            Modifier::Shift => xlib::ShiftMask,
            Modifier::CapsLock => xlib::LockMask,
            Modifier::Ctrl => xlib::ControlMask,
            Modifier::Alt => xlib::Mod1Mask,
            Modifier::NumLock => xlib::Mod2Mask,
            Modifier::ScrollLock => xlib::Mod3Mask,
            Modifier::Window => xlib::Mod4Mask,
            Modifier::Mod5 => xlib::Mod5Mask,
        }
    }
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

    /// Add callback to execute on new thread when key is pressed.
    pub fn add(&self, callback: Callback) {
        STORAGE.add(self, callback);
    }

    /// Remove all callbacks atached to this key.
    pub fn clear(&self) {
        STORAGE.remove_all(self);
    }
}
