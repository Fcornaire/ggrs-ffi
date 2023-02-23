use std::{mem::forget, os::raw::c_char};

use crate::{
    model::{
        ffi::{
            config_ffi::ConfigFFI, game_state_ffi::GameStateFFI, input_ffi::Inputs,
            netplay_request_ffi::NetplayRequests,
        },
        input::Input,
        netplay_request::NetplayRequest,
        network_stats::NetworkStats,
    },
    Events, Status, NETPLAY,
};
use std::ffi::CString;

#[no_mangle]
pub unsafe extern "C" fn netplay_init(config_ffi: *mut ConfigFFI) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    match np.init(config_ffi) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
    }
}

#[no_mangle]
pub extern "C" fn netplay_poll() -> Status {
    let mut np = NETPLAY.lock().unwrap();

    match np.poll_remote() {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
    }
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
        let _ = Vec::from_raw_parts(events.data as *mut Events, 0, 0);
    };
}

#[no_mangle]
pub extern "C" fn netplay_advance_frame(input: Input) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    match np.advance_frame(input) {
        Ok(_) => Status::ok(),
        Err(e) => {
            if e.eq(&"PredictionThreshold") {
                Status::ok()
            } else {
                Status::ko(Box::leak(e.into_boxed_str()))
            }
        }
    }
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
        let _ = Vec::from_raw_parts(requests.data as *mut NetplayRequest, 0, 0);
    };
}

//TODO: catch_unwind on other method for more safety/sending err msg
#[no_mangle]
pub unsafe extern "C" fn netplay_save_game_state(game_state_ffi: *mut GameStateFFI) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    let result =
        std::panic::catch_unwind(
            move || match np.handle_save_game_state_request(game_state_ffi) {
                Ok(_) => Status::ok(),
                Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
            },
        );

    match result {
        Ok(status) => status,
        Err(e) => Status::ko(e.downcast_ref::<&str>().unwrap()),
    }
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
pub unsafe extern "C" fn netplay_load_game_state(game_state: *mut GameStateFFI) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    match np.handle_load_game_state_request(game_state) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
    }
}

#[no_mangle]
pub unsafe extern "C" fn netplay_inputs_free(inputs: Inputs) {
    if inputs.data.is_null() {
        return;
    }
    let _ = Vec::from_raw_parts(inputs.data as *mut Inputs, 0, 0);
}

#[no_mangle]
pub extern "C" fn netplay_skip_frames() -> u32 {
    let np = NETPLAY.lock().unwrap();

    np.skip_frames()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_network_stats(network_stats: *mut NetworkStats) -> Status {
    let mut np = NETPLAY.lock().unwrap();

    match np.network_stats(network_stats) {
        Ok(_) => Status::ok(),
        Err(e) => Status::ko(Box::leak(e.into_boxed_str())),
    }
}
