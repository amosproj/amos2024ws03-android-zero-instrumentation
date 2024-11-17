// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

use procfs::{process::all_processes, ProcError};
use shared::ziofa::{self, ProcessList};
use tonic;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcErrorWrapper {
    #[error(transparent)]
    ProcError(#[from] ProcError),
}

impl From<ProcErrorWrapper> for tonic::Status {
    fn from(err: ProcErrorWrapper) -> Self {
        Self::from_error(Box::new(err))
    }
}


pub fn list_processes() -> Result<ProcessList, ProcError> {
    // Get all processes
    all_processes().and_then(|op| {
        let processes = op
            .filter_map(|el| {
                // filter out all Errors
                let process = el.ok()?;
                let stat = process.stat().ok()?;
                Some(ziofa::Process {
                    pid: stat.pid,
                    ppid: stat.ppid,
                    comm: stat.comm,
                    state: stat.state.to_string(),
                })
            })
            .collect();
        Ok(ProcessList { processes })
    })
}
