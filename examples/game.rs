extern crate bindkey;
#[macro_use]
extern crate lazy_static;

use std::sync::atomic::{AtomicBool, Ordering};

use bindkey::*;

lazy_static! {
    static ref WIN: AtomicBool = AtomicBool::new(false);
}

fn main() {
    let ctrl_alt_q = HotKey::new(keysym::XK_q,
                                 vec![Modifier::Ctrl, Modifier::Alt],
                                 TriggerOn::Press);
    ctrl_alt_q.add(ctrl_alt_q_pressed);

    let ctrl_alt_win_p = HotKey::new(keysym::XK_p,
                                     vec![Modifier::Ctrl, Modifier::Alt, Modifier::Window],
                                     TriggerOn::Release);

    ctrl_alt_win_p.add(ctrl_alt_win_p_released);

    let hadle = start_async();

    // remove callbacks after 5 secs
    std::thread::sleep_ms(5_000);
    ctrl_alt_win_p.clear();

    if !WIN.load(Ordering::SeqCst) {
        println!("you lose");
    }

    hadle.join();
}

fn ctrl_alt_q_pressed() {
    println!("ctrl+alt+q");
}

fn ctrl_alt_win_p_released() {
    if WIN.load(Ordering::SeqCst) {
        println!("you won again");
    } else {
        WIN.store(true, Ordering::SeqCst);
        println!("you won
    }
}