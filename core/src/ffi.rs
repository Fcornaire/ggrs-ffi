use std::os::raw::c_char;

use macros::{catch_action_result, catch_status};

use crate::{
    config::app_config::AppConfig,
    core::{
        action_result::ActionResult,
        unmanaged::{safe_bytes::SafeBytes, unmanaged_bytes::UnmanagedBytes},
    },
    has_netplay_disconnected, guard_netplay_instance,
    model::{
        ffi::{input_ffi::Inputs, netplay_request_ffi::NetplayRequests},
        game_state::GameState,
        input::Input,
        netplay_request::NetplayRequest,
        network_stats::NetworkStats,
    },
    start_background_poller, stop_background_poller, Events, Status,
};
use std::ffi::CString;

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_init(config: SafeBytes) -> Status {
    let safe_config = AppConfig::new(config);
    let is_test = safe_config.is_test();

    let status = {
        let mut np = guard_netplay_instance();

        np.init(safe_config)
    };

    if status.is_ok() && !is_test {
        start_background_poller();
    }

    status
}

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_poll() -> Status {
    if has_netplay_disconnected() {
        return Status::msg("Peer Disconnected!");
    }

    let mut np = guard_netplay_instance();

    np.poll_remote()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_is_synchronized() -> Status {
    let mut np = guard_netplay_instance();

    match np.is_synchronized() {
        true => Status::ok(),
        false => Status::ko("not synchronized"),
    }
}

#[no_mangle]
pub extern "C" fn netplay_is_disconnected() -> Status {
    match has_netplay_disconnected() {
        true => Status::ok(),
        false => Status::ko("not disconnected"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_events() -> Events {
    let mut np = guard_netplay_instance();

    return Events::new(np.events());
}

#[no_mangle]
pub unsafe extern "C" fn status_info_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}

#[no_mangle]
pub unsafe extern "C" fn netplay_events_free(events: Events) {
    if events.data.is_null() {
        return;
    }
    let strings = Vec::from_raw_parts(
        events.data,
        events.len.try_into().unwrap(),
        events.cap.try_into().unwrap(),
    );

    for s in strings {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_set_test_inputs(data: *const Input, len: i32) -> Status {
    let mut np = guard_netplay_instance();

    if data.is_null() || len <= 0 {
        np.set_test_inputs(vec![]);

        return Status::ok();
    }

    let slice = std::slice::from_raw_parts(data, len as usize);

    np.set_test_inputs(slice.to_vec());

    Status::ok()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_advance_frame(input: Input) -> Status {
    let mut np = guard_netplay_instance();

    let res = std::panic::catch_unwind(move || match np.advance_frame(input) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(&e),
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
            return Status::ko(&error_msg);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_get_requests() -> NetplayRequests {
    let np = guard_netplay_instance();

    return NetplayRequests::new(np.requests());
}

#[no_mangle]
pub unsafe extern "C" fn netplay_requests_free(requests: NetplayRequests) {
    if requests.data.is_null() {
        return;
    }
    let _ = Vec::from_raw_parts(
        requests.data as *mut NetplayRequest,
        requests.len as usize,
        requests.len as usize,
    );
}

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_save_game_state(game_state: SafeBytes) -> Status {
    let mut np = guard_netplay_instance();

    let safe_game_state = GameState::new(game_state);

    np.handle_save_game_state_request(safe_game_state)
}

#[no_mangle]
pub unsafe extern "C" fn netplay_advance_game_state() -> Inputs {
    let mut np = guard_netplay_instance();

    return Inputs::new(np.handle_advance_frame_request());
}

#[no_mangle]
#[catch_action_result]
pub unsafe extern "C" fn netplay_load_game_state() -> ActionResult {
    let mut np = guard_netplay_instance();

    np.handle_load_game_state_request()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_inputs_free(inputs: Inputs) {
    if inputs.data.is_null() {
        return;
    }
    let _ = Vec::from_raw_parts(
        inputs.data as *mut Input,
        inputs.len as usize,
        inputs.len as usize,
    );
}

#[no_mangle]
pub unsafe extern "C" fn netplay_network_stats(
    player_handle: i32,
    network_stats: *mut NetworkStats,
) -> Status {
    let mut np = guard_netplay_instance();

    match np.network_stats(player_handle, network_stats) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(&e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_frames_ahead() -> i32 {
    let mut np = guard_netplay_instance();

    match np.frames_ahead() {
        Ok(frames_ahead) => frames_ahead,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_free_game_state(safe_bytes: SafeBytes) {
    let mut np = guard_netplay_instance();

    safe_bytes.release();

    np.reset_game_state();
}

#[no_mangle]
pub unsafe extern "C" fn netplay_current_frame() -> i32 {
    let np = guard_netplay_instance();

    np.game_state().frame()
}

#[no_mangle]
#[catch_status]
pub unsafe extern "C" fn netplay_reset() -> Status {
    stop_background_poller();

    let mut np = guard_netplay_instance();

    np.reset()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_local_player_handle() -> i32 {
    let np = guard_netplay_instance();

    np.local_player_handle()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_remote_player_handle() -> i32 {
    let np = guard_netplay_instance();

    np.remote_player_handle()
}

/// Host only registers a late spectator
#[no_mangle]
pub unsafe extern "C" fn netplay_add_spectator(peer_id: *const c_char) -> Status {
    if peer_id.is_null() {
        return Status::ko("add_spectator : peer id is null");
    }

    let peer_id = match std::ffi::CStr::from_ptr(peer_id).to_str() {
        Ok(s) => s,
        Err(_) => return Status::ko("add_spectator : peer id is not valid"),
    };

    let mut np = guard_netplay_instance();

    match np.add_spectator(peer_id) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(&e),
    }
}

/// Spectator only: confirmed frames left to replay before reaching the host
#[no_mangle]
pub unsafe extern "C" fn netplay_frames_behind() -> i32 {
    let np = guard_netplay_instance();

    np.frames_behind()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_remote_player_handle_count() -> i32 {
    let np = guard_netplay_instance();

    np.remote_player_handles().len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn netplay_remote_player_handle_at(index: i32) -> i32 {
    let np = guard_netplay_instance();

    if index < 0 {
        return -1;
    }

    np.remote_player_handles()
        .get(index as usize)
        .map(|handle| *handle as i32)
        .unwrap_or(-1)
}
