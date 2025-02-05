<!--
SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
SPDX-FileCopyrightText: 2025 Tom Weisshuhn <tom.weisshuhn@fau.de>

SPDX-License-Identifier: MIT
-->


# Ebpf

## Events

Currently there are the following events:

- Blocking
- FileDescriptorChange
- Signal
- Write
- GarbageCollect
- JniReference

### Blocking

For every syscall the time is measured between enter and exit.

```rust
pub struct Blocking {
    /// The syscall id
    pub syscall_id: u64,
    /// The duration of the blocking event in nanoseconds
    pub duration: u64,
}
```

> [!WARNING]
> The duration is only an estimate and might be wrong, due to the fact that syscalls can be preempted.

### FileDescriptorChange

Every time a syscall that modifies the amount of open file descriptors is used.

```rust
pub struct FileDescriptorChange {
    /// The current number of open file descriptors
    pub open_fds: u64,
    /// Whether the operation was a open or close
    pub operation: FileDescriptorOp,
}
```

> [!NOTE]
> The number of open file descriptors is gathered from the current processes task struct.
> This is done by getting the pointer to the file descriptor table (`task_struct->files->fdt`) and reading the `max_fds` and `open_fds` fields.
> `max_fds` is the maximum number of file descriptors that can be opened at the same time.
> `open_fds` is a bitmap that indicates which file descriptors are currently open.
> By summing all the ones in the bitmap up to `max_fds` we get the number of open file descriptors.
>
> However to pass the verifier an upper limit has to be specified (currently 1024).
> So the theoretical maximum number of open file descriptors that can be detected is `1024 * sizeof(u64)`.
>
> Also typically when accessing kernel structures a `rcu_read_lock` is required.
> As this is only available in very new kernels for ebpf we are not making use of it (https://docs.ebpf.io/linux/kfuncs/bpf_rcu_read_lock/).
> This means the data could be in an inconsistent state.


### Signal

The event is emitted each time a singal is sent to another process.

```rust
pub struct Signal {
    /// The PID of the target process
    pub target_pid: i32,
    /// The signal that was sent
    pub signal: u32,
}
```

> [!NOTE]
> This event only detects whether a signal was sent to another process.
> It does not detect whether the signal was actually received and handled by that target process.

### Write

Every time any of the write syscalls (`write`, `writev`, `pwrite64`, `pwritev`, `pwritev2`) is used.

```rust
pub struct Write {
    /// The number of bytes written
    pub bytes_written: u64,
    /// The file descriptor that was written to
    pub file_descriptor: u64,
    /// The file path of the file that was written to
    pub file_path: [u8; 4096],
    /// The source of the write event, e.g. the which syscall was used
    pub source: WriteSource,
}
```

> [!NOTE]
> This does not detect whether data was written to a real disk, a device mount or memory file descriptors (e.g. sockets, pipes, memfd).
> One can utilize the `file_path` field afterwards, or add additional logic inside ebpf to map the writes to specific block devices.

### GarbageCollect

Every ART garbage collection.

```rust
pub struct GarbageCollect {
    /// The footprint of the targeted object
    pub target_footprint: u64,
    /// The number of bytes allocated
    pub num_bytes_allocated: u64,
    /// The cause of the garbage collection
    pub gc_cause: u32,
    /// The duration of the garbage collection in nanoseconds
    pub duration_ns: u64,
    /// The number of objects that were freed
    pub freed_objects: u64,
    /// The number of bytes that were freed
    pub freed_bytes: u64,
    /// The number of los objects that were freed
    pub freed_los_objects: u64,
    /// The number of los bytes that were freed
    pub freed_los_bytes: u64,
    /// The number of garbage collections that were completed for this process
    pub gcs_completed: u32,
}
```

### JniReference

Every time a local or global reference is added or deleted.

```rust
pub struct JniReference {
    /// The JNI operation that was performed (AddLocalRef, DeleteLocalRef, AddGlobalRef, DeleteGlobalRef)
    pub operation: JniOperation,
    /// The JNI reference that was added or deleted
    pub reference: u64,
}
```

## Programs

All our events, except for `GarbageCollect` and `JniReference` are extracted from syscalls.

For that ebpf programs of type `RAW_TRACEPOINT` are used.
Originally separate programs of type `TRACEPOINT` in the `syscall` category were utilized (e.g. `syscalls:sys_enter_write`, `syscalls:sys_enter_kill`), however for them to be available the kernel needs to be compiled with `CONFIG_FTRACE_SYSCALLS=y`.

One can find a list of all supported tracepoints by reading the contents of `/sys/kernel/debug/tracing/available_events` or `/sys/kernel/tracing/available_events` (Standard Linux seems to have the former, Android the latter).

### `RAW_TRACEPOINT` on `raw_syscalls:sys_enter` and `raw_syscalls:sys_exit` for syscall events

The logic for all events is pretty much the same:

Program on `raw_syscalls:sys_enter`:

1. Get `task_struct`, `pt_regs` and `syscall_id` from the context
2. Compute `TaskContext` and `ProcessContext` from `task_struct`
3. Extract event specific data from the information above
4. Store data into an intermediate `HashMap` indexed by `EventKind` and `TID`.

Program on `raw_syscalls:sys_exit`:

1. Get `task_struct`, `pt_regs` and `return_value` from the context
2. Compute `TaskContext` and `ProcessContext` from `task_struct`
3. Get intermediate data from the `HashMap`
4. Compute event from the data above.
5. Apply filters to the event.
6. Submit the event into a ring buffer to be consumed by the userspace program.

### `UPROBE` and `URETPROBE` on `GcCollectInternal` for GarbageCollect

To gather the information needed for constructing the `GarbageCollect` event, access to the `art::gc::Heap` is required every time a garbage collection is performed.
This class has multiple methods to invoke garbage collections, however every one of them internally calls `CollectGarbageInternal`.

A program is attached to the entry and exit of the `CollectGarbageInternal` function (`uprobe` and `uretprobe` respectively).
The entry program is necessary for getting a pointer to the `art::gc::Heap` object.
The exit program is necessary to extract the information from the `art::gc::Heap` object after the garbage collection has been performed.

#### Finding the address of `CollectGarbageInternal`

The `C++` code in the AOSP has two macros called `EXPORT` and `HIDDEN` that essentially expand to `__attribute__((visibility("default")))` and `__attribute__((visibility("hidden")))`.
In newer versions of Android most of the `art::gc::Heap` methods are hidden because of that macro, including `CollectGarbageInternal`.

To find the address regardless, there are two strategies:

1. Adding the `EXPORT` macro to this method or removing the `HIDDEN` macro, recompiling `libart` and looking up the address in the symbol table.
2. Making use of the fact that `CollectGarbage` is exported and only ends up calling `CollectGarbageInternal`.
   After disassembling `libart` it is quite easy to locate `CollectGarbage` and look at the destination of the jump (`objdump -C -S libart.so > libart.asm`).

The assembly instructions on `x86_64` (generated via host `objdump`):

```asm
0000000000582d00 <art::gc::Heap::CollectGarbage(bool, art::gc::GcCause)@@Base>:
  582d00:	89 f1                	mov    %esi,%ecx
  582d02:	48 8b 87 28 03 00 00 	mov    0x328(%rdi),%rax
  582d09:	8b 70 fc             	mov    -0x4(%rax),%esi
  582d0c:	41 b8 ff ff ff ff    	mov    $0xffffffff,%r8d
  582d12:	e9 f9 7f ff ff       	jmp    57ad10 <art::gc::Heap::PerformHomogeneousSpaceCompact()@@Base+0x15a0>
  582d17:	cc                   	int3
  582d18:	cc                   	int3
  582d19:	cc                   	int3
  582d1a:	cc                   	int3
  582d1b:	cc                   	int3
  582d1c:	cc                   	int3
  582d1d:	cc                   	int3
  582d1e:	cc                   	int3
  582d1f:	cc                   	int3
```

The assembly instructions on `aarch64` (generated via `llvm-objdump`):

```asm
00000000004aaaf0 <art::gc::Heap::CollectGarbage(bool, art::gc::GcCause)>:
4aaaf0: f9419408     	ldr	x8, [x0, #0x328]
4aaaf4: 2a0103e3     	mov	w3, w1
4aaaf8: 12800004     	mov	w4, #-0x1               // =-1
4aaafc: b85fc108     	ldur	w8, [x8, #-0x4]
4aab00: 2a0803e1     	mov	w1, w8
4aab04: 17fffafb     	b	0x4a96f0 <art::gc::Heap::WaitForGcToComplete(art::gc::GcCause, art::Thread*)+0x990>
```

From the instructions of the `art::gc::Heap::CollectGarbage` function show that the address of `CollectGarbageInternal` is `0x57ad10` on `x86_64` and `0x4a96f0` on `aarch64`, because of the uncoconditional jump (`jmp`) and unconditional branch (`b`).

For reference, the original `C++` method:

```cpp
void Heap::CollectGarbage(bool clear_soft_references, GcCause cause) {
  // Even if we waited for a GC we still need to do another GC since weaks allocated during the
  // last GC will not have necessarily been cleared.
  CollectGarbageInternal(gc_plan_.back(), cause, clear_soft_references, GC_NUM_ANY);
}
```


#### Getting the memory layout of `art::gc::Heap`

To access the relevant information necessary for the `GarbageCollect` event, access to fields of the `art::gc::Heap` is required.
`libclang` is used to find the size and offset of the relevant fields.
This information is then converted to `btf` information which is used to the field accesses inside the ebpf program.
One could also compile `libart` with debug symbols enabled and use `pahole` to generate the `btf` information.
This means essentially that the `art::gc::Heap` can be accessed like a normal struct from the ebpf side.

```rust
GarbageCollect {
    target_footprint: heap.target_footprint().ok()?,
    num_bytes_allocated: heap.num_bytes_allocated().ok()?,
    gc_cause: heap.gc_cause().ok()?,
    duration_ns: heap.duration_ns().ok()?,
    freed_objects: heap.freed_objects().ok()?,
    freed_bytes: heap.freed_bytes().ok()?,
    freed_los_objects: heap.freed_los_objects().ok()?,
    freed_los_bytes: heap.freed_los_bytes().ok()?,
    gcs_completed: heap.gcs_completed().ok()?,
}
```


## Filtering

Currently the following filters are available:

- Pid/Tid
- Comm
- ExePath
- Cmdline

Each type of filter requires a `HashMap` from the key (e.g. pid, comm, etc.) to two bitmaps.
The first bitmap is 1 when the event should be accepted and 0 when it should be rejected.
The second bitmap is 1 when the filter is active for this event id and 0 when it is not.
Also there is a general setting for each event that specifies what should be done if the key cannot be found in the `HashMap`: e.g. whether to match or not by default.

Rejections have higher precedence, meaning that if a certain pid is both rejected and accepted, the event will be rejected.

This concept is inspired by [tracee](https://github.com/aquasecurity/tracee/blob/main/pkg/ebpf/c/common/filtering.h)

## Relocations

The ebpf programs are written in Rust using the `aya-ebpf` crate.
While the `aya` crate supports relocating ebpf programs with BTF debug information, the `aya-ebpf` counterpart does not support emitting those yet when writing ebpf programs (https://github.com/aya-rs/aya/issues/349).

To work around this issue, a separate `C` file containing all struct definitions with sort of `getter` functions is used. Every getter is marked with `__attribute__((always_inline))`, this makes it possible that they are inlined into our rust code:

```c
#define inline __attribute__((always_inline))

struct files_struct {
	atomic_t count;
	struct fdtable *fdt;
};

inline struct fdtable **files_struct_fdt(struct files_struct *files)
{
	return &files->fdt;
}

inline atomic_t *files_struct_count(struct files_struct *files)
{
	return &files->count;
}
```

Every struct is marked with `preserve_access_index`:

```c
#if defined(__bpf__)
#pragma clang attribute push(__attribute__((preserve_access_index)),           \
                             apply_to = record)
#endif
```

This file is then compiled to llvm bitcode using `clang`: `clang -g -emit-llvm -c relocation_helpers.c -o relocation_helpers.bc`.

Using `bpf-linker` the bitcode of our rust code and the bitcode of the helpers are combined and compiled into a single object file.

## Transfer between userspace and ebpf

A single RingBuffer is used to transfer all events from ebpf to userspace.
As the events have different sizes, the first 8 bytes of the event are used to specify the kind of event.
