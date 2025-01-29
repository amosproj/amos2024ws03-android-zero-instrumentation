// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use crate::{FieldMetadata, Heap, HeapMetadata};

pub trait MemReader {
    type Error;

    fn read<T>(
        &self,
        out: &mut T,
        offset: usize,
        len: usize,
        rel: bool,
    ) -> core::result::Result<(), Self::Error>;
}

pub enum Error<T> {
    SizeMismatch,
    ReadError(T),
}

fn read_inner<T, M: MemReader>(
    field: &mut T,
    meta: FieldMetadata,
    reader: &M,
) -> Result<(), Error<M::Error>> {
    let expected = meta.size;
    let actual = size_of::<T>();

    if expected != actual {
        return Err(Error::SizeMismatch);
    }

    reader
        .read(field, meta.offset, size_of::<T>(), true)
        .map_err(Error::ReadError)
}

pub fn read<M: MemReader>(
    heap: &mut Heap,
    meta: HeapMetadata,
    reader: &M,
) -> Result<(), Error<M::Error>> {
    read_inner(&mut heap.target_footprint, meta.target_footprint, reader)?;
    read_inner(
        &mut heap.num_bytes_allocated,
        meta.num_bytes_allocated,
        reader,
    )?;
    read_inner(&mut heap.gcs_completed, meta.gcs_completed, reader)?;
    let mut gc_cause = 0u32;
    read_inner(&mut gc_cause, meta.gc_cause, reader)?;
    heap.gc_cause = gc_cause.into();
    read_inner(&mut heap.duration_ns, meta.duration_ns, reader)?;
    read_inner(&mut heap.freed_objects, meta.freed_objects, reader)?;
    read_inner(&mut heap.freed_bytes, meta.freed_bytes, reader)?;
    read_inner(&mut heap.freed_los_objects, meta.freed_los_objects, reader)?;
    read_inner(&mut heap.freed_los_bytes, meta.freed_los_bytes, reader)?;

    let mut start_ptr = 0usize;
    read_inner(&mut start_ptr, meta.pause_times_begin, reader)?;
    let mut end_ptr = 0usize;
    read_inner(&mut end_ptr, meta.pause_times_end, reader)?;
    let len = (end_ptr - start_ptr) / size_of::<u64>();
    let min_len = heap.pause_times.len().min(len);
    reader
        .read::<[u64; 8]>(
            &mut heap.pause_times,
            start_ptr,
            min_len * size_of::<u64>(),
            false,
        )
        .map_err(Error::ReadError)?;

    Ok(())
}
