// SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

use aya_ebpf::helpers::bpf_probe_read_kernel_buf;

use crate::{
    bounds_check::EbpfBoundsCheck,
    iterator_ext::IteratorExt,
    relocation_helpers::{Dentry, Mount, Path, Vfsmount},
};

pub struct PathWalker {
    vfs_mount: Vfsmount,
    dentry: Dentry,
    mnt_p: Mount,
    mnt_parent_p: Mount,
}

impl PathWalker {
    pub fn new(path: Path) -> Result<Self, i64> {
        let vfs_mount = path.mnt()?;
        let dentry = path.dentry()?;
        let mnt_p = vfs_mount.container();
        let mnt_parent_p = mnt_p.mnt_parent()?;

        Ok(Self {
            vfs_mount,
            dentry,
            mnt_p,
            mnt_parent_p,
        })
    }

    fn traverse_mount(&mut self) -> Result<(), i64> {
        self.dentry = self.mnt_p.mnt_mountpoint()?;
        self.mnt_p = self.mnt_p.mnt_parent()?;
        self.mnt_parent_p = self.mnt_p.mnt_parent()?;
        self.vfs_mount = self.mnt_p.mnt();
        Ok(())
    }

    fn mnt_root(&self) -> Result<Dentry, i64> {
        self.vfs_mount.mnt_root()
    }

    fn parent(&self) -> Result<Dentry, i64> {
        self.dentry.d_parent()
    }

    fn get_name(&self) -> Result<*const [u8], i64> {
        let qstr = self.dentry.d_name();
        let len = qstr.len()? as usize;
        let name = qstr.name()?;

        let s = slice_from_raw_parts(name, len);

        Ok(s)
    }

    fn is_parent(&self) -> Result<bool, i64> {
        Ok(self.dentry == self.parent()?)
    }

    fn is_mnt_root(&self) -> Result<bool, i64> {
        Ok(self.dentry == self.mnt_root()?)
    }

    fn can_traverse_mount(&self) -> Result<bool, i64> {
        Ok(self.mnt_p != self.mnt_parent_p)
    }

    fn next_inner(&mut self) -> Result<Option<PathComponent>, i64> {
        if self.is_parent()? || self.is_mnt_root()? {
            if self.can_traverse_mount()? {
                self.traverse_mount()?;
                return Ok(Some(PathComponent::Mount));
            }
            return Ok(None);
        }

        let name = self.get_name()?;

        self.dentry = self.parent()?;

        Ok(Some(PathComponent::Name(name)))
    }
}

pub enum PathComponent {
    /// Got a new component, must read via bpf_probe_read_kernel
    Name(*const [u8]),
    /// Traversed a mountpoint
    Mount,
}

impl Iterator for PathWalker {
    type Item = PathComponent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner().ok().flatten()
    }
}

pub const PATH_MAX: usize = 4096;

struct Offset<'a> {
    max: usize,
    off: usize,
    buf: &'a mut [u8],
}

impl<'a> Offset<'a> {
    pub fn new(buf: &'a mut [u8], max: usize) -> Self {
        Self { max, off: max, buf }
    }

    unsafe fn next_slice(&mut self, len: usize) -> Option<&'a mut [u8]> {
        self.off = (self.off - len).bounded(self.max)?;

        let ptr = self.buf.as_mut_ptr().add(self.off);
        let slice = &mut *slice_from_raw_parts_mut(ptr, len);

        Some(slice)
    }
}

pub fn get_path_str(path: Path, buf: &mut [u8; PATH_MAX * 2]) -> Option<usize> {
    let mut offset = Offset::new(buf.as_mut_slice(), PATH_MAX);

    let components = PathWalker::new(path)
        .ok()?
        .const_take::<20>()
        .filter_map(|item| match item {
            PathComponent::Name(name) => Some(name),
            PathComponent::Mount => None,
        });

    for name in components {
        let len = unsafe { name.len().bounded(PATH_MAX)? };

        let slice = unsafe { offset.next_slice(len + 1)? };

        slice[0] = b'/';
        unsafe { bpf_probe_read_kernel_buf(name as *const u8, &mut slice[1..]).ok()? };
    }

    Some(offset.off)
}
