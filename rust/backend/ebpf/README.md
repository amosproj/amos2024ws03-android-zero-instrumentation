
<!--  
SPDX-FileCopyrightText: 2025 Tom Weisshuhn <tom.weisshuhn@fau.de>  
  
SPDX-License-Identifier: MIT  
-->

# eBPF programs

The entries in the maps are the structs defined in `../common/src/lib.rs`.
The maps `<hook-name>_PIDS` are HashMaps that store the pid as key and as value the duration for a call to be considered blocking in nanosec.
For the features `SIGQUIT` and `JNIReferences` the durations are irrelevant as they are not relevant for the use-cases.

## overview by hook name

|               | type       | functions to hook                                                          | map<entry-type>                          |  
|---------------|------------|----------------------------------------------------------------------------|------------------------------------------|  
| vfs_write     | KProbe     | `vfs_write`, `vfs_write_ret`                                               | `VFS_WRITE_EVENTS<VfsWriteCall>`         |  
| sendmsg       | Tracepoint | `sys_enter_sendmsg`, `sys_exit_sendmsg`                                    | `SYS_SENDMSG_CALLS<SysSendmsgCall>`      |
| SIGQUIT       | Tracepoint | `sys_sigquit`                                                              | `SYS_SIGQUIT_CALLS<SysSigquitCall>`      |
| fd_tracking   | Tracepoint | `sys_create_fd`, `sys_fd_destroy`                                           | `SYS_FDTRACKING_EVENTS<SysFdActionCall>` |
| JNIReferences | UProbe     | `trace_add_local`, `trace_del_local`, `trace_add_global`, `trace_del_global` | `JNI_REF_CALLS<JNIRefCall>`              |
| ...           | ...        | ...                                                                        | ...                                      |
