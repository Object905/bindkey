extern crate bindkey;

use bindkey::*;


fn main() {
    let mut storage = CallbackStorage::new();

    let ctrl_alt_q = HotKey::new(keysym::XK_q,
                                 vec![Modifier::Ctrl, Modifier::Alt],
                                 TriggerOn::Press);
    storage.add(&ctrl_alt_q, ctrl_alt_q_pressed);

    let ctrl_alt_win_p = HotKey::new(keysym::XK_p,
                                     vec![Modifier::Ctrl, Modifier::Alt, Modifier::Window],
                                     TriggerOn::Release);

    storage.add(&ctrl_alt_win_p, ctrl_alt_win_p_released);

    let panic_on = HotKey::new(keysym::XK_equal, vec![Modifier::Ctrl], TriggerOn::Press);
    storage.add(&panic_on, panik);

    start(storage);
}

fn panik() {
    panic!("KAPPA");
}

fn ctrl_alt_q_pressed() {
    println!("ctrl+alt+q");
}

fn ctrl_alt_win_p_released() {
    println!("lul");
}
