// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Franz Schlicht <franz.schlicht@gmail.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use aya::{Ebpf, EbpfError};
use shared::config::Configuration;
use thiserror::Error;

use crate::features::{SysSendmsgFeature, VfsFeature, JNIReferencesFeature};

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
    sys_sendmsg_feature: SysSendmsgFeature,
    jni_references_feature: JNIReferencesFeature,
}

impl State {
    pub fn new() -> State {
        State {
            vfs_write_feature: VfsFeature::new(),
            sys_sendmsg_feature: SysSendmsgFeature::new(),
            jni_references_feature: JNIReferencesFeature::new()
        }
    }

    pub fn init(&mut self, ebpf: &mut Ebpf) -> Result<(), EbpfError> {
        self.vfs_write_feature.create(ebpf)?;
        self.sys_sendmsg_feature.create(ebpf)?;
        self.jni_references_feature.create(ebpf)?;


        Ok(())
    }

    pub fn update_from_config(
        &mut self,
        ebpf: &mut Ebpf,
        config: &Configuration,
    ) -> Result<(), EbpfError> {
        self.vfs_write_feature.apply(ebpf, config.vfs_write.as_ref())?;
        self.sys_sendmsg_feature.apply(ebpf, config.sys_sendmsg.as_ref())?;
        self.jni_references_feature.apply(ebpf, config.jni_references.as_ref())?;


        Ok(())
    }
}
