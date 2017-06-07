use std::ptr;
use std::os::raw::{c_int, c_uint, c_ulonglong};

use x11::xlib;

use {HotKey, Modifier};

// CapsLock + NumLock + ScrollLock + Mod5
const IGNORED_MODIFIER_MASK: c_uint = xlib::LockMask | xlib::Mod2Mask | xlib::Mod3Mask |
                                      xlib::Mod5Mask;

pub unsafe fn grab(display: *mut xlib::Display, root: xlib::Window, key: &HotKey) {

    let keycode = xlib::XKeysymToKeycode(display, key.key);

    for mask in make_ignored_mask(&key.modifiers) {
        xlib::XGrabKey(display,
                       keycode as c_int,
                       mask,
                       root,
                       true as c_int,
                       xlib::GrabModeAsync,
                       xlib::GrabModeAsync);
    }
}


pub fn ugrab(display: *mut xlib::Display, root: xlib::Window, key: &HotKey) {
    unsafe {
        let keycode = xlib::XKeysymToKeycode(display, key.key as c_ulonglong);

        for mask in make_ignored_mask(&key.modifiers) {
            xlib::XUngrabKey(display, keycode as c_int, mask, root);
        }
    }
}

pub unsafe fn next_event(display: *mut xlib::Display, event: &mut xlib::XEvent) {
    xlib::XNextEvent(display, event as *mut _);
}

pub unsafe fn get_display() -> *mut xlib::Display {
    xlib::XOpenDisplay(ptr::null())
}

pub unsafe fn get_root(display: *mut xlib::Display) -> xlib::Window {
    xlib::XDefaultRootWindow(display)
}

pub fn get_keysym(press: &mut xlib::XKeyEvent) -> xlib::KeySym {
    unsafe { xlib::XLookupKeysym(press as *mut _, 0) }
}

fn make_ignored_mask(modifiers: &[Modifier]) -> Vec<c_uint> {
    let mut modifier_mask;

    if let Some(first_mask) = modifiers.get(0) {
        modifier_mask = *first_mask as u32;

        for mask in &modifiers[1..] {
            modifier_mask |= *mask as u32;
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
