// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

// TODO: fork aya temporarily to export these types
// transmuting is sound because of repr(C) but it's not good practice

use aya_obj::btf::{Btf, BtfError, BtfKind, BtfType};
use alloc::vec::Vec;

use crate::HeapMetadata;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Int {
    name_offset: u32,
    info: u32,
    size: u32,
    data: u32,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct BtfMember {
    name_offset: u32,
    btf_type: u32,
    offset: u32,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct Struct {
    name_offset: u32,
    info: u32,
    size: u32,
    members: Vec<BtfMember>,
}

pub fn apply_to_btf(btf: &mut Btf, data: &HeapMetadata) -> Result<(), BtfError> {
    let u64_ty = btf
        .id_by_type_name_kind("long long unsigned int", BtfKind::Int)?;
    let u32_ty = btf
        .id_by_type_name_kind("unsigned int", BtfKind::Int)?;

    let members = [
        ("target_footprint", data.target_footprint.size, data.target_footprint.offset),
        ("num_bytes_allocated", data.num_bytes_allocated.size, data.num_bytes_allocated.offset),
        ("gc_cause", data.gc_cause.size, data.gc_cause.offset),
        ("duration_ns", data.duration_ns.size, data.duration_ns.offset),
        ("freed_objects", data.freed_objects.size, data.freed_objects.offset),
        ("freed_bytes", data.freed_bytes.size, data.freed_bytes.offset),
        ("freed_los_objects", data.freed_los_objects.size, data.freed_los_objects.offset),
        ("freed_los_bytes", data.freed_los_bytes.size, data.freed_los_bytes.offset),
        ("gcs_completed", data.gcs_completed.size, data.gcs_completed.offset),
        ("pause_times_begin", data.pause_times_begin.size, data.pause_times_begin.offset),
        ("pause_times_end", data.pause_times_end.size, data.pause_times_end.offset),
    ]
    .into_iter()
    .map(|(name, size, offset)| BtfMember {
        name_offset: btf.add_string(name),
        btf_type: if size == 8 { u64_ty } else if size == 4 { u32_ty } else { panic!("invalid size {size}") },
        offset: offset as u32,
    })
    .collect::<Vec<_>>();

    let heap_name = btf.add_string("art_heap");
    let strct = Struct {
        name_offset: heap_name,
        info: ((BtfKind::Struct as u32) << 24) | ((members.len() as u32) & 0xFFFF),
        size: members.len() as u32 * 8,
        members 
    };

    
    btf.add_type(BtfType::Struct(unsafe { core::mem::transmute::<Struct, aya_obj::btf::Struct>(strct) }));
        
    Ok(())
}