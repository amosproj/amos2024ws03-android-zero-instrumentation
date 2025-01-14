// SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
//
// SPDX-License-Identifier: MIT

use std::{io, mem, os::fd::{AsFd, AsRawFd}};

use aya::{maps::{HashMap, RingBuf}, programs::RawTracePoint, EbpfLoader};
use aya_obj::generated::{bpf_attr, bpf_cmd};
use backend_common::{SysSigquitCall, TryFromRaw};
use libc::{getpid, gettid, syscall, SYS_bpf};


#[test]
fn prog_test_run_example() {
    let mut ebpf = EbpfLoader::default()
        .load(aya::include_bytes_aligned!(concat!(
            env!("OUT_DIR"),
            "/backend-ebpf"
        )))
        .unwrap();
    
    let p: &mut RawTracePoint = ebpf.program_mut("sys_sigquit").unwrap().try_into().unwrap();
    p.load().unwrap();
    p.attach("sys_enter").unwrap();
    
    let fd = p.fd().unwrap().as_fd().as_raw_fd();
    
    let mut pids: HashMap<_, u32, u64> = ebpf.take_map("SYS_SIGQUIT_PIDS").unwrap().try_into().unwrap();
    let old = pids.iter().filter_map(Result::ok).map(|x| x.0).collect::<Vec<_>>();
    for old in old {
        pids.remove(&old).unwrap();
    }
    // Pid of the program seems to always be the next pid
    pids.insert(unsafe { gettid() as u32 }, 0, 0).unwrap();
    let mut events: RingBuf<_> = ebpf.take_map("SYS_SIGQUIT_EVENTS").unwrap().try_into().unwrap();
    
    let target_pid = 1111;
    let signal = 3; // sigquit
    let args = [0u64, 0u64, target_pid, signal];
    
    let mut attr = unsafe { mem::zeroed::<bpf_attr>() };
    
    attr.test.prog_fd = fd as u32;
    attr.test.ctx_in = args.as_ptr() as u64;
    attr.test.ctx_size_in = args.len() as u32 * 8;

    let _ = {
        let ret = unsafe { syscall(SYS_bpf, bpf_cmd::BPF_PROG_TEST_RUN, &mut attr, size_of::<bpf_attr>()) };
        
        match ret {
            0.. => Ok(ret),
            ret => Err((ret, io::Error::last_os_error())),
        }
    }.unwrap();
    
    println!("{:?}", unsafe { attr.test });
    
    let next = events.next().unwrap();
    
    println!("{next:?}");
    println!("{:?}", SysSigquitCall::try_from_raw(&next));
    println!("{} {}", unsafe { gettid() }, unsafe { getpid() });
}