// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{Ebpf, EbpfError};
use shared::config::Configuration;
use thiserror::Error;

use crate::features::{SysSendmsgFeature, VfsFeature};

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

