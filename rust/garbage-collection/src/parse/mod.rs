// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

mod field;
mod namespace;
mod ty;

use clang::{Clang, Index, TranslationUnit};
pub use field::Field;
pub use namespace::Namespace;
use thiserror::Error;
pub use ty::Type;

use crate::{FieldMetadata, HeapMetadata};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing namespace: {namespace}")]
    MissingNamespace { namespace: &'static str },
    #[error("missing type: {ty}")]
    MissingType { ty: &'static str },
    #[error("missing field: {field}")]
    MissingField { field: &'static str },
    #[error("missing size: {field}")]
    MissingSize { field: &'static str },
    #[error("missing offset: {field}")]
    MissingOffset { field: &'static str },
}

fn get_namespace<'a>(
    parent: &'a Namespace,
    name: &'static str,
) -> Result<&'a Namespace, ParseError> {
    parent
        .try_get_namespace(name)
        .ok_or(ParseError::MissingNamespace { namespace: name })
}

fn get_type<'a>(parent: &'a Namespace, name: &'static str) -> Result<&'a Type, ParseError> {
    parent
        .try_get_type(name)
        .ok_or(ParseError::MissingType { ty: name })
}

fn get_field<'a>(parent: &'a Type, name: &'static str) -> Result<&'a Field, ParseError> {
    parent
        .try_get_field(name)
        .ok_or(ParseError::MissingField { field: name })
}

fn field_to_metadata(
    field: &Field,
    name: &'static str,
    base_offset: usize,
) -> Result<FieldMetadata, ParseError> {
    let offset = field
        .try_get_offset()
        .ok_or(ParseError::MissingOffset { field: name })?;
    let ty = field
        .try_get_type()
        .ok_or(ParseError::MissingType { ty: name })?;
    let size = ty
        .try_get_size()
        .ok_or(ParseError::MissingSize { field: name })?;

    Ok(FieldMetadata {
        offset: (offset + base_offset) / 8,
        size,
    })
}

pub const INCLUDE_FILES: &[&str] = &[
    "system/libbase/include",
    "art/runtime",
    "art/libartbase",
    "external/fmtlib/include",
    "external/tinyxml2",
    "art/libdexfile",
    "bionic/libc/platform",
];

pub const DEFAULT_ARGS: &[&str] = &[
    "-xc++",
    "-std=c++20",
    "-Wno-invalid-offsetof",
    "-DART_DEFAULT_GC_TYPE_IS_CMC",
    "-DART_STACK_OVERFLOW_GAP_arm=8192",
    "-DART_STACK_OVERFLOW_GAP_arm64=8192",
    "-DART_STACK_OVERFLOW_GAP_riscv64=8192",
    "-DART_STACK_OVERFLOW_GAP_x86=8192",
    "-DART_STACK_OVERFLOW_GAP_x86_64=8192",
    "-DART_USE_GENERATIONAL_CC=1",
    "-DART_USE_TLAB=1",
    "-DART_PAGE_SIZE_AGNOSTIC=1",
    "-DART_TARGET",
    "-DART_TARGET_ANDROID",
    "-DUSE_D8_DESUGAR=1",
];

pub fn get_includes(base: Option<&str>) -> Vec<String> {
    let prefix = if let Some(base) = base {
        format!("-I{}", base)
    } else {
        "-I".to_string()
    };
    INCLUDE_FILES
        .iter()
        .map(|path| format!("{}{}", prefix, path))
        .collect()
}

pub fn get_system_includes() -> Vec<String> {
    let clang = clang_sys::support::Clang::find(None, &[]).unwrap();
    let args = clang.cpp_search_paths.unwrap();

    args.into_iter()
        .map(|arg| arg.canonicalize().unwrap())
        .map(|path| path.to_string_lossy().into_owned())
        .map(|path| format!("-isystem{}", path))
        .collect::<Vec<_>>()
}

pub fn parse(base: Option<String>) -> Result<HeapMetadata, ParseError> {
    let raw_args = DEFAULT_ARGS.iter().map(|s| s.to_string());
    let all_args = get_system_includes()
        .into_iter()
        .chain(get_includes(base.as_deref()))
        .chain(raw_args);

    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);

    let path = if let Some(base) = base {
        format!("{}/art/runtime/gc/heap.h", base)
    } else {
        "art/runtime/gc/heap.h".to_string()
    };
    let tu = index
        .parser(path)
        .arguments(all_args.collect::<Vec<_>>().as_slice())
        .parse()
        .unwrap();

    for diagnostic in tu.get_diagnostics() {
        eprintln!("{}", diagnostic);
    }

    parse_heap_metadata(tu)
}

pub fn parse_heap_metadata(tu: TranslationUnit) -> Result<HeapMetadata, ParseError> {
    let root = tu.get_entity();
    let namespace = Namespace::parse(root);

    let art = get_namespace(&namespace, "art")?;
    let gc = get_namespace(art, "gc")?;
    let heap = get_type(gc, "Heap")?;

    let target_footprint = get_field(heap, "target_footprint_")?;
    let num_bytes_allocated = get_field(heap, "num_bytes_allocated_")?;
    let gcs_completed = get_field(heap, "gcs_completed_")?;

    let current_gc_iteration = get_field(heap, "current_gc_iteration_")?;
    let current_gc_iteration_offset =
        current_gc_iteration
            .try_get_offset()
            .ok_or(ParseError::MissingOffset {
                field: "current_gc_iteration_",
            })?;
    let current_gc_iteration_type = current_gc_iteration
        .try_get_type()
        .ok_or(ParseError::MissingType { ty: "Iteration" })?;

    let pause_times = get_field(current_gc_iteration_type, "pause_times_")?;
    let pause_times_offset = pause_times
        .try_get_offset()
        .ok_or(ParseError::MissingOffset {
            field: "pause_times_",
        })?
        + current_gc_iteration_offset;
    let pause_times_ty = pause_times.try_get_type().ok_or(ParseError::MissingType {
        ty: "vector<uint64_t>",
    })?;

    let pause_times_begin = get_field(pause_times_ty, "__begin_")?;
    let pause_times_end = get_field(pause_times_ty, "__end_")?;

    let gc_cause = get_field(current_gc_iteration_type, "gc_cause_")?;
    let duration_ns = get_field(current_gc_iteration_type, "duration_ns_")?;

    let freed = get_field(current_gc_iteration_type, "freed_")?;
    let freed_offset = freed
        .try_get_offset()
        .ok_or(ParseError::MissingOffset { field: "freed_" })?
        + current_gc_iteration_offset;
    let freed_type = freed.try_get_type().ok_or(ParseError::MissingType {
        ty: "ObjectBytePair",
    })?;

    let freed_objects = get_field(freed_type, "objects")?;
    let freed_bytes = get_field(freed_type, "bytes")?;

    let freed_los = get_field(current_gc_iteration_type, "freed_los_")?;
    let freed_los_offset = freed_los
        .try_get_offset()
        .ok_or(ParseError::MissingOffset {
            field: "freed_los_",
        })?
        + current_gc_iteration_offset;
    let freed_los_type = freed_los.try_get_type().ok_or(ParseError::MissingType {
        ty: "ObjectBytePair",
    })?;

    let freed_los_objects = get_field(freed_los_type, "objects")?;
    let freed_los_bytes = get_field(freed_los_type, "bytes")?;

    Ok(HeapMetadata {
        target_footprint: field_to_metadata(target_footprint, "target_footprint_", 0)?,
        num_bytes_allocated: field_to_metadata(num_bytes_allocated, "num_bytes_allocated_", 0)?,
        gcs_completed: field_to_metadata(gcs_completed, "gcs_completed_", 0)?,
        gc_cause: field_to_metadata(gc_cause, "gc_cause_", current_gc_iteration_offset)?,
        duration_ns: field_to_metadata(duration_ns, "duration_ns_", current_gc_iteration_offset)?,
        freed_objects: field_to_metadata(freed_objects, "freed_objects", freed_offset)?,
        freed_bytes: field_to_metadata(freed_bytes, "freed_bytes", freed_offset)?,
        freed_los_objects: field_to_metadata(
            freed_los_objects,
            "freed_los_objects",
            freed_los_offset,
        )?,
        freed_los_bytes: field_to_metadata(freed_los_bytes, "freed_los_bytes", freed_los_offset)?,
        pause_times_begin: field_to_metadata(pause_times_begin, "__begin_", pause_times_offset)?,
        pause_times_end: field_to_metadata(pause_times_end, "__end_", pause_times_offset)?,
    })
}
