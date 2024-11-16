// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use procfs::{process::all_processes, ProcError};
use shared::ziofa::{self, ProcessList};

pub fn list_processes() -> Result<ProcessList, ProcError> {
    // Get all processes
    all_processes().and_then(|op| {
        let processes = op
            .filter_map(|el| {
                // filter out all Errors
                let process = el.ok()?;
                let stat = process.stat().ok()?;
                let cmdline = process.cmdline().ok()?;

                match cmdline {
                    // filter out all processes with empty cmdline
                    c if c.len() == 0 => None,
                    c => Some(ziofa::Process {
                        pid: stat.pid,
                        ppid: stat.ppid,
                        cmdline: c.join(" "),
                        comm: stat.comm,
                        state: stat.state.to_string(),
                    }),
                }
            })
            .collect();
        Ok(ProcessList {processes})
    })
}
