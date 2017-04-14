use std::ptr;
use std::os::raw::{c_int, c_uint, c_ulonglong};

use x11_dl::xlib;

use {HotKey, Modifier};
use storage::STORAGE;

lazy_static! {
    static ref XLIB: xlib::Xlib = xlib::Xlib::open().expect("failed to open xlib");
}

// CapsLock + NumLock + ScrollLock + Mod5
const IGNORED_MODIFIER_MASK: c_uint = xlib::LockMask | xlib::Mod2Mask | xlib::Mod3Mask |
                                      xlib::Mod5Mask;

pub fn grab(display: *mut xlib::Display, root: xlib::Window, key: &HotKey) {
    unsafe {
        let keycode = (XLIB.XKeysymToKeycode)(display, key.key);

        for mask in make_ignored_mask(&key.modifiers) {
            (XLIB.XGrabKey)(display,
                            keycode as c_int,
                            mask,
                            root,
                            true as c_int,
                            xlib::GrabModeAsync,
                            xlib::GrabModeAsync);
        }
    }
}

pub fn ugrab(display: *mut xlib::Display, root: xlib::Window, key: &HotKey) {
    unsafe {
        let keycode = (XLIB.XKeysymToKeycode)(display, key.key as c_ulonglong);

        for mask in make_ignored_mask(&key.modifiers) {
            (XLIB.XUngrabKey)(display, keycode as c_int, mask, root);
        }
    }
}

pub fn next_event(event: &mut xlib::XEvent) {
    unsafe {
        (XLIB.XNextEvent)(STORAGE.display, event as *mut _);
    }
}

pub fn get_display() -> *mut xlib::Display {
    unsafe { (XLIB.XOpenDisplay)(ptr::null()) }
}

pub fn get_root(display: *mut xlib::Display) -> xlib::Window {
    unsafe { (XLIB.XDefaultRootWindow)(display) }
}

pub fn get_keysym(press: &mut xlib::XKeyEvent) -> xlib::KeySym {
    unsafe { (XLIB.XLookupKeysym)(press as *mut _, 0) }
}

fn make_ignored_mask(modifiers: &[Modifier]) -> Vec<c_uint> {
    let mut modifier_mask;

    if let Some(first_mask) = modifiers.get(0) {
        modifier_mask = first_mask.mask();

        for mask in &modifiers[1..] {
            modifier_mask |= mask.mask();
        }
    } else {
        modifier_mask = xlib::AnyModifier;
    }

    let mut ignored_mask = 0;
    let mut result = Vec::new();

    while ignored_mask <= IGNORED_MODIFIER_MASK {
        if (ignored_mask & !IGNORED_MODIFIER_MASK) > 0 {
            // Contains some non-ignored modifiers
            ignored_mask += 1;
            continue;
        }

        result.push(modifier_mask | ignored_mask);

        ignored_mask += 1;
    }

    result
}