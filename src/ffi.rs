use std::{net::SocketAddr, os::raw::c_char};

use crate::{input::Input, Events, GGRSConfig, Status, NETPLAY};
use ggrs::{GGRSEvent, PlayerType, SessionBuilder, UdpNonBlockingSocket};
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
        let arc = NETPLAY.clone();
        let mut np = arc.lock().unwrap();
        if np.session().is_none() {
            np.update_session(sess_ptr);
        }
    }

    Status::ok()
}

#[no_mangle]
pub unsafe extern "C" fn netplay_pool() -> Status {
    let arc = NETPLAY.clone();
    let mut np = arc.lock().unwrap();

    if np.session().is_some() {
        let session = np.session().take().unwrap();
        (*session).poll_remote_clients();
        np.update_session(session);
        return Status::ok();
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub unsafe extern "C" fn netplay_add_local_input(input: Input) -> Status {
    let arc = NETPLAY.clone();
    let np = arc.lock().unwrap();

    if np.session().is_some() {
        let session = np.session().take().unwrap();
        (*session).add_local_input(0, input).unwrap();
        return Status::ok();
    }

    Status::ko("Netplay is null")
}

#[no_mangle]
pub unsafe extern "C" fn netplay_events() -> Events {
    let arc = NETPLAY.clone();
    let np = arc.lock().unwrap();

    if np.session().is_some() {
        let session = np.session().take().unwrap();

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
