
<!--  
SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>  
  
SPDX-License-Identifier: MIT  
-->

# eBPF programs

The entries in the maps are the structs defined in `../common/src/lib.rs`.
The maps `<hook-name>_PIDS` store the pid as key and as value the duration for a call to be considered blocking in nanosec. 

## overview by hook name

|            |type        | functions to hook                                  |map                 |  
|-----------|-----------|---------------------------------------|-------------------|  
|vfs_write  |KProbe          |`vfs_write`, `vfs_write_ret`                |`VFS_WRITE_EVENTS`, `VFS_WRITE_PIDS`    |  
|sendmsg       |Tracepoint    |`sys_enter_sendmsg`, `sys_exit_sendmsg`|`SYS_SENDMSG_EVENTS`, `SYS_SENDMSG_PIDS`  |  
|...          |...            |...                                                      |...                        |
