use std::{mem::forget, net::SocketAddr, os::raw::c_char};

use crate::{
    model::{
        input::Input,
        netplay_request::{NetplayRequest, NetplayRequests},
    },
    Events, GGRSConfig, Status, NETPLAY,
};
use ggrs::{GGRSError, GGRSEvent, PlayerType, SessionBuilder, UdpNonBlockingSocket};
use std::ffi::CString;

#[no_mangle]
pub unsafe extern "C" fn netplay_init() -> Status {
    let local_port = 7000;
    let remote_addr: SocketAddr = "192.168.1.19:7000".parse().unwrap();
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
pub unsafe extern "C" fn netplay_add_local_input(input: Input) -> Status {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session().take().unwrap();
        (*session).add_local_input(0, input).unwrap();
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
pub unsafe extern "C" fn netplay_advance_frame() -> Status {
    if NETPLAY.session().is_some() {
        let session = NETPLAY.session().take().unwrap();

        if NETPLAY.requests().is_empty() {
            match (*session).advance_frame() {
                Ok(requests) => {
                    NETPLAY.update_requests(requests);

                    return Status::ok();
                }
                Err(GGRSError::PredictionThreshold) => Status::ko("PredictionThreshold"),
                Err(e) => Status::ko(Box::leak(Box::new(format!(
                    "GGRSError : {}",
                    e.to_string()
                )))), //TODO: send error
            };
        } else {
            return Status::ko(
                "Netplay request is not empty. Finish using request before advancing",
            );
        }

        NETPLAY.update_session(session);
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
