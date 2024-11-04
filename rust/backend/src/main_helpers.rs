use log::debug;
use std::net::{SocketAddr, ToSocketAddrs};

pub fn bump_rlimit() {
    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }
}

pub fn get_socket_addr() -> SocketAddr {
    // unsave, but does that matter?
    // addr. shouldn't really change right
    "[::1]:50051".to_socket_addrs().unwrap().next().unwrap()
}