// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{Ebpf, EbpfError};
use thiserror::Error;

use crate::features::VfsFeature;

#[derive(Debug, Error)]
pub enum EbpfErrorWrapper {
    #[error(transparent)]
    EbpfError(#[from] EbpfError),
}

impl From<EbpfErrorWrapper> for tonic::Status {
    fn from(err: EbpfErrorWrapper) -> Self {
        Self::from_error(Box::new(err))
    }
}

pub struct State {
    vfs_write_feature: VfsFeature,
}

impl State {
    pub fn new() -> State {
        State {
            vfs_write_feature: VfsFeature::new(),
        }
    }

    pub fn init(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        self.vfs_write_feature.create(ebpf)?;

        Ok(())
    }

    pub fn update_from_config(
        &mut self,
        ebpf: &mut Ebpf,
        _config_path: &str,
    ) -> Result<(), EbpfError> {
        self.vfs_write_feature.attach(ebpf)?;

        Ok(())
    }
}
