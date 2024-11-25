
<!--  
SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>  
  
SPDX-License-Identifier: MIT  
-->  

# eBPF programs

The entries in the maps are the structs defined in `../common/src/lib.rs`.

## overview by hook name

|            |type        | functions to hook                                  |map                 |  
|-----------|-----------|---------------------------------------|-------------------|  
|vfs_write  |KProbe          |`vfs_write`, `vfs_write_ret`                |`VFS_WRITE_MAP`    |  
|sendmsg       |Tracepoint    |`sys_enter_sendmsg`, `sys_exit_sendmsg`|`SYS_SENDMSG_MAP`  |  
|...          |...            |...                                                      |...                        |
