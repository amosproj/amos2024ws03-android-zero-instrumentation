#![no_std]
#![no_main]

// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

use aya_ebpf::{bindings::xdp_action, macros::{map, xdp, kprobe}, maps::{PerCpuArray, RingBuf}, programs::{XdpContext, ProbeContext}, EbpfContext};
use example_common::{KProbeData, KProbeTypes};

#[map(name="COUNTER")]
static PACKET_COUNTER: PerCpuArray<u32> = PerCpuArray::with_max_entries(1, 0);

#[map(name="EVENTS")]
static EVENTS: RingBuf = RingBuf::with_byte_size(1024, 0);

#[map(name="KProbes")]
static KPROBES: RingBuf = RingBuf::with_byte_size(1024, 0);

#[kprobe]
pub fn vfs_write(ctx: ProbeContext) -> Result<(), u32> {
    let pid = ctx.pid();
    let tid = ctx.tgid();

    let data = KProbeData{pid, tid, probe_type: KProbeTypes::VfsWrite};
    let mut entry = match KPROBES.reserve::<KProbeData>(0) {
        Some(entry) => entry,
        None => return Err(0)
    };

    entry.write(data);
    entry.submit(0);

    Ok(())
}

#[xdp]
pub fn example(ctx: XdpContext) -> u32 {
    match try_example(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_example(_: XdpContext) -> Result<u32, ()> {
    unsafe {
        let counter = PACKET_COUNTER.get_ptr_mut(0).ok_or(())?;
        *counter += 1;
        let mut entry = match EVENTS.reserve::<u32>(0) {
            Some(entry) => entry,
            None => return Ok(xdp_action::XDP_PASS)
        };
        
        entry.write(*counter);
        entry.submit(0);

    }
    Ok(xdp_action::XDP_PASS)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
