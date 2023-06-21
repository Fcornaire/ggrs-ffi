use std::{mem::forget, os::raw::c_char};

use macros::{catch_action_result, catch_status};

use crate::{
    config::app_config::AppConfig,
    core::{
        action_result::ActionResult,
        unmanaged::{safe_bytes::SafeBytes, unmanaged_bytes::UnmanagedBytes},
    },
    model::{
        ffi::{input_ffi::Inputs, netplay_request_ffi::NetplayRequests},
        game_state::GameState,
        input::Input,
        netplay_request::NetplayRequest,
        network_stats::NetworkStats,
    },
    Events, Status, NETPLAY, SHOULD_STOP_MATCHBOX_FUTURE,
};
use std::ffi::CString;

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_init(config: SafeBytes) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    let safe_config = AppConfig::new(config);

    np.init(safe_config)
}

#[no_mangle]
#[catch_status]
pub extern "C" fn netplay_poll() -> Status {
    let is_disconnected = is_disconnected();

    if is_disconnected {
        return Status::ko(Box::leak("Peer Disconnected!".to_string().into_boxed_str()));
    }

    let mut np = NETPLAY.lock().unwrap();

    np.poll_remote()
}

#[no_mangle]
pub extern "C" fn netplay_events() -> Events {
    let mut np = NETPLAY.lock().unwrap();

    return Events::new(np.events());
}

#[no_mangle]
pub extern "C" fn status_info_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}

#[no_mangle]
pub extern "C" fn netplay_events_free(events: Events) {
    unsafe {
        if events.data.is_null() {
            return;
        }
        let _ = Vec::from_raw_parts(events.data as *mut Events, events.len, events.cap);
    };
}

#[no_mangle]
pub extern "C" fn netplay_advance_frame(input: Input) -> Status {
    let is_disconnected = is_disconnected();

    if !is_disconnected {
        let mut np = NETPLAY.lock().unwrap();

        let res = std::panic::catch_unwind(move || match np.advance_frame(input) {
            Ok(_) => Status::ok(),
            Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
        });

        match res {
            Ok(status) => return status,
            Err(e) => {
                let error_msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown error".to_string()
                };
                return Status::ko(Box::leak(error_msg.into_boxed_str()));
            }
        }
    }

    Status::ko(Box::leak("Peer Disconnected!".to_string().into_boxed_str()))
}

#[no_mangle]
pub extern "C" fn netplay_get_requests() -> NetplayRequests {
    let np = NETPLAY.lock().unwrap();

    let requests = np.requests();
    let reqs = NetplayRequests::new(requests.clone());

    forget(requests);

    return reqs;
}

#[no_mangle]
pub extern "C" fn netplay_requests_free(requests: NetplayRequests) {
    unsafe {
        if requests.data.is_null() {
            return;
        }
        let _ = Vec::from_raw_parts(
            requests.data as *mut NetplayRequest,
            requests.len,
            requests.len,
        );
    };
}

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_save_game_state(game_state: SafeBytes) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    let safe_game_state = GameState::new(game_state);

    np.handle_save_game_state_request(safe_game_state)
}

#[no_mangle]
pub extern "C" fn netplay_advance_game_state() -> Inputs {
    let mut np = NETPLAY.lock().unwrap();

    let inputs = np.handle_advance_frame_request();
    let inputs_ffi = Inputs::new(inputs.clone());
    forget(inputs);

    return inputs_ffi;
}

#[no_mangle]
#[catch_action_result]
pub unsafe extern "C" fn netplay_load_game_state() -> ActionResult {
    let mut np = NETPLAY.lock().unwrap();
    np.handle_load_game_state_request()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_inputs_free(inputs: Inputs) {
    if inputs.data.is_null() {
        return;
    }
    let _ = Vec::from_raw_parts(inputs.data as *mut Inputs, inputs.len, inputs.len);
}

#[no_mangle]
pub unsafe extern "C" fn netplay_network_stats(network_stats: *mut NetworkStats) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    match np.network_stats(network_stats) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_frames_ahead() -> i32 {
    let mut np = NETPLAY.lock().unwrap();

    match np.frames_ahead() {
        Ok(frames_ahead) => frames_ahead,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_free_game_state(safe_bytes: SafeBytes) {
    let mut np = NETPLAY.lock().unwrap();

    let slice = safe_bytes.slice();
    drop(slice);

    np.reset_game_state();
}

#[no_mangle]
pub unsafe extern "C" fn netplay_current_frame() -> i32 {
    let np = NETPLAY.lock().unwrap();
    np.game_state().frame()
}

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_reset() -> Status {
    let mut np = NETPLAY.lock().unwrap();
    np.reset()
}

#[no_mangle]
pub extern "C" fn netplay_local_player_handle() -> i32 {
    let np = NETPLAY.lock().unwrap();

    np.local_player_handle()
}

#[no_mangle]
pub extern "C" fn netplay_remote_player_handle() -> i32 {
    let np = NETPLAY.lock().unwrap();

    np.remote_player_handle()
}

fn is_disconnected() -> bool {
    match SHOULD_STOP_MATCHBOX_FUTURE.try_lock() {
        Ok(should_stop) => {
            if *should_stop {
                return true;
            }
        }
        Err(_) => {}
    }

    false
}
