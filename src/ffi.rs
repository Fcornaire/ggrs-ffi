use std::{mem::forget, net::SocketAddr, os::raw::c_char};

use crate::{
    model::{
        ffi::{input_ffi::Inputs, netplay_request_ffi::NetplayRequests, state_ffi::GameStateFFI},
        input::Input,
        netplay_request::NetplayRequest,
        network_stats::NetworkStats,
    },
    Events, GGRSConfig, Status, NETPLAY,
};
use ggrs::{GGRSError, GGRSEvent, PlayerType, SessionBuilder, UdpNonBlockingSocket};
use std::ffi::CString;

#[no_mangle]
pub unsafe extern "C" fn netplay_init() -> Status {
    let local_port = 7000;
    let remote_addr: SocketAddr = "192.168.1.14:7000".parse().unwrap();
    let socket = UdpNonBlockingSocket::bind_to_port(local_port).unwrap();

    let sess_ptr = Box::into_raw(Box::new(
        SessionBuilder::<GGRSConfig>::new()
            .with_num_players(2)
            .add_player(PlayerType::Local, 0)
            .unwrap()
            .add_player(PlayerType::Remote(remote_addr), 1)
            .unwrap()
            .start_p2p_session(socket)
            .unwrap(),
    ));

    {
        if NETPLAY.session().is_none() {
            NETPLAY.update_session(sess_ptr);
        }
    }

    Status::ok()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_init_test() -> Status {
    let sess_ptr = Box::into_raw(Box::new(
        SessionBuilder::new()
            .with_num_players(2)
            .with_check_distance(2)
            .with_input_delay(0)
            .start_synctest_session()
            .unwrap(),
    ));

    {
        if NETPLAY.session().is_none() {
            NETPLAY.update_session_test(sess_ptr);
        }
    }

    Status::ok()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_poll() -> Status {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session().take().unwrap();
        (*session).poll_remote_clients();
        NETPLAY.update_session(session);
        return Status::ok();
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub unsafe extern "C" fn netplay_events() -> Events {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session().take().unwrap();

        let mut events: Vec<&'static str> = vec![];

        for (_, event) in (*session).events().enumerate() {
            match event {
                GGRSEvent::Synchronizing { addr, total, count } => {
                    let str = format!(
                        "Synchronizing addr {} total {} count {}",
                        addr, total, count
                    );
                    let str: &'static str = Box::leak(str.into_boxed_str());

                    events.push(str)
                }
                GGRSEvent::Synchronized { addr } => {
                    let str = format!("Synchronized addr {}", addr);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }
                GGRSEvent::Disconnected { addr: _ } => events.push("Disconnected"),

                GGRSEvent::NetworkInterrupted {
                    addr,
                    disconnect_timeout,
                } => {
                    let str = format!(
                        "NetworkInterrupted addr {} disconnect timout {}",
                        addr, disconnect_timeout
                    );
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }

                GGRSEvent::WaitRecommendation { skip_frames } => {
                    NETPLAY.update_skip_frames(skip_frames + 1);

                    let str = format!("WaitRecommendation skip frames {}", skip_frames);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }

                GGRSEvent::NetworkResumed { addr } => {
                    let str = format!("NetworkResumed addr {}", addr);
                    let str: &'static str = Box::leak(str.into_boxed_str());
                    events.push(str)
                }
            }
        }

        NETPLAY.minus_skip_frames();

        NETPLAY.update_session(session);

        return Events::new(events);
    }

    Events::empty()
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
pub unsafe extern "C" fn netplay_advance_frame(input: Input) -> Status {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session().take().unwrap();

        (*session).add_local_input(0, input).unwrap();

        if NETPLAY.requests().is_empty() {
            match (*session).advance_frame() {
                Ok(requests) => {
                    NETPLAY.update_requests(requests);
                    NETPLAY.update_session(session);

                    return Status::ok();
                }
                Err(GGRSError::PredictionThreshold) => {
                    NETPLAY.update_session(session);
                    return Status::ko("PredictionThreshold");
                }
                Err(e) => {
                    NETPLAY.update_session(session);
                    return Status::ko(Box::leak(Box::new(format!(
                        "GGRSError : {}",
                        e.to_string()
                    ))));
                } //TODO: send error
            };
        } else {
            NETPLAY.update_session(session);

            return Status::ko(
                "Netplay request is not empty. Finish using request before advancing",
            );
        }
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub unsafe extern "C" fn netplay_advance_frame_test(input: Input) -> Status {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session_test().take().unwrap();

        (*session).add_local_input(0, input).unwrap();
        (*session).add_local_input(0, Input::default()).unwrap();

        if NETPLAY.requests().is_empty() {
            match (*session).advance_frame() {
                Ok(requests) => {
                    NETPLAY.update_requests(requests);
                    NETPLAY.update_session_test(session);

                    return Status::ok();
                }
                Err(GGRSError::PredictionThreshold) => {
                    NETPLAY.update_session_test(session);
                    return Status::ko("PredictionThreshold");
                }
                Err(e) => {
                    NETPLAY.update_session_test(session);
                    return Status::ko(Box::leak(Box::new(format!(
                        "GGRSError : {}",
                        e.to_string()
                    ))));
                } //TODO: send error
            };
        } else {
            NETPLAY.update_session_test(session);

            return Status::ko(
                "Netplay request is not empty. Finish using request before advancing",
            );
        }
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub unsafe extern "C" fn netplay_get_requests() -> NetplayRequests {
    if NETPLAY.session().is_some() {
        let requests = NETPLAY.requests();
        let reqs = NetplayRequests::new(requests.clone());

        forget(requests);

        return reqs;
    }
    NetplayRequests::empty()
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

#[no_mangle]
pub unsafe extern "C" fn netplay_save_game_state(game_state: *mut GameStateFFI) -> Status {
    if NETPLAY.session().is_some() {
        let game_state = (*game_state).clone().to_model(NETPLAY.game_state().frame());
        return NETPLAY.handle_save_game_state_request(Some(game_state));
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub unsafe extern "C" fn netplay_advance_game_state() -> Inputs {
    if NETPLAY.session().is_some() {
        let inputs = NETPLAY.handle_advance_frame_request();

        let inputs_ffi = Inputs::new(inputs.clone());

        forget(inputs);

        return inputs_ffi;
    }

    return Inputs::empty();
}

#[no_mangle]
pub unsafe extern "C" fn netplay_load_game_state(game_state: *mut GameStateFFI) -> Status {
    if NETPLAY.session().is_some() {
        let result = NETPLAY.handle_load_game_state_request();

        if result.is_ok() {
            let to_load = NETPLAY.game_state();

            (*game_state).update(to_load);
        }

        return result;
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub extern "C" fn netplay_inputs_free(inputs: Inputs) {
    unsafe {
        if inputs.data.is_null() {
            return;
        }
        let _ = Vec::from_raw_parts(inputs.data as *mut Inputs, 0, 0);
    };
}

#[no_mangle]
pub unsafe extern "C" fn netplay_skip_frames() -> u32 {
    if NETPLAY.session().is_some() {
        return NETPLAY.skip_frames();
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn netplay_network_stats(network_stats: *mut NetworkStats) -> Status {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session().take().unwrap();
        let stats = (*session).network_stats(1);
        let str = format!("{:?}", stats);
        let str: &'static str = Box::leak(str.into_boxed_str());
        if let Ok(net) = stats {
            (*network_stats) = NetworkStats::new(
                net.send_queue_len,
                net.ping,
                net.kbps_sent,
                net.local_frames_behind,
                net.remote_frames_behind,
            );

            NETPLAY.update_session(session);
            return Status::msg(str);
        }

        NETPLAY.update_session(session);
        return Status::ko(str);
    }

    Status::ko("Netplay is null")
}
