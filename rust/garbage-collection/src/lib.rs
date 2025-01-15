#![cfg_attr(all(not(feature = "parse"), not(feature = "serialize")), no_std)]

// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use bytemuck::{CheckedBitPattern, Pod, PodInOption, Zeroable, ZeroableInOption};

#[cfg(feature = "parse")]
pub mod parse;

#[cfg(feature = "read")]
pub mod read;

// SAFETY: When all fields are 0, the struct might as well be None.
unsafe impl PodInOption for HeapMetadata {}

// SAFETY: When all fields are 0, the struct might as well be None.
unsafe impl ZeroableInOption for HeapMetadata {}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldMetadata {
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct HeapMetadata {
    pub target_footprint: FieldMetadata,
    pub num_bytes_allocated: FieldMetadata,
    pub gc_cause: FieldMetadata,
    pub duration_ns: FieldMetadata,
    pub freed_objects: FieldMetadata,
    pub freed_bytes: FieldMetadata,
    pub freed_los_objects: FieldMetadata,
    pub freed_los_bytes: FieldMetadata,
    pub gcs_completed: FieldMetadata,
    pub pause_times_begin: FieldMetadata,
    pub pause_times_end: FieldMetadata,
}

/// See https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/gc/gc_cause.h
#[repr(u32)]
#[derive(Debug, Clone, Copy, Default, CheckedBitPattern)]
pub enum GcCause {
    /// Invalid GC cause used as a placeholder.
    #[default]
    GcCauseNone,
    /// GC triggered by a failed allocation. Thread doing allocation is blocked waiting for GC before
    /// retrying allocation.
    GcCauseForAlloc,
    /// A background GC trying to ensure there is free memory ahead of allocations.
    GcCauseBackground,
    /// An explicit System.gc() call.
    GcCauseExplicit,
    /// GC triggered for a native allocation when NativeAllocationGcWatermark is exceeded.
    /// (This may be a blocking GC depending on whether we run a non-concurrent collector).
    GcCauseForNativeAlloc,
    /// GC triggered for a collector transition.
    GcCauseCollectorTransition,
    /// Not a real GC cause, used when we disable moving GC (currently for GetPrimitiveArrayCritical).
    GcCauseDisableMovingGc,
    /// Not a real GC cause, used when we trim the heap.
    GcCauseTrim,
    /// Not a real GC cause, used to implement exclusion between GC and instrumentation.
    GcCauseInstrumentation,
    /// Not a real GC cause, used to add or remove app image spaces.
    GcCauseAddRemoveAppImageSpace,
    /// Not a real GC cause, used to implement exclusion between GC and debugger.
    GcCauseDebugger,
    /// GC triggered for background transition when both foreground and background collector are CMS.
    GcCauseHomogeneousSpaceCompact,
    /// Class linker cause, used to guard filling art methods with special values.
    GcCauseClassLinker,
    /// Not a real GC cause, used to implement exclusion between code cache metadata and GC.
    GcCauseJitCodeCache,
    /// Not a real GC cause, used to add or remove system-weak holders.
    GcCauseAddRemoveSystemWeakHolder,
    /// Not a real GC cause, used to prevent hprof running in the middle of GC.
    GcCauseHprof,
    /// Not a real GC cause, used to prevent GetObjectsAllocated running in the middle of GC.
    GcCauseGetObjectsAllocated,
    /// GC cause for the profile saver.
    GcCauseProfileSaver,
    /// GC cause for deleting dex cache arrays at startup.
    GcCauseDeletingDexCacheArrays,
}

impl From<u32> for GcCause {
    fn from(value: u32) -> Self {
        match value {
            0 => GcCause::GcCauseNone,
            1 => GcCause::GcCauseForAlloc,
            2 => GcCause::GcCauseBackground,
            3 => GcCause::GcCauseExplicit,
            4 => GcCause::GcCauseForNativeAlloc,
            5 => GcCause::GcCauseCollectorTransition,
            6 => GcCause::GcCauseDisableMovingGc,
            7 => GcCause::GcCauseTrim,
            8 => GcCause::GcCauseInstrumentation,
            9 => GcCause::GcCauseAddRemoveAppImageSpace,
            10 => GcCause::GcCauseDebugger,
            11 => GcCause::GcCauseHomogeneousSpaceCompact,
            12 => GcCause::GcCauseClassLinker,
            13 => GcCause::GcCauseJitCodeCache,
            14 => GcCause::GcCauseAddRemoveSystemWeakHolder,
            15 => GcCause::GcCauseHprof,
            16 => GcCause::GcCauseGetObjectsAllocated,
            17 => GcCause::GcCauseProfileSaver,
            18 => GcCause::GcCauseDeletingDexCacheArrays,
            _ => GcCause::GcCauseNone,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, CheckedBitPattern)]
pub struct Heap {
    pub target_footprint: usize,
    pub num_bytes_allocated: usize,
    pub gcs_completed: u32,
    pub gc_cause: GcCause,
    pub duration_ns: u64,
    pub freed_objects: u64,
    pub freed_bytes: i64,
    pub freed_los_objects: u64,
    pub freed_los_bytes: i64,
    pub pause_times: [u64; 8]
}