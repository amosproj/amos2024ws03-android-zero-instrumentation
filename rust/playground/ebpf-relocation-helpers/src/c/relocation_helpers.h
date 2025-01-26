/*
 * SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef __RELOCATION_HELPERS_H__
#define __RELOCATION_HELPERS_H__

#if defined(__bpf__)
#pragma clang attribute push(__attribute__((preserve_access_index)),           \
                             apply_to = record)
#endif

typedef long long unsigned int u64;
typedef unsigned int u32;
typedef int s32;

// We cannot use __builtin_offsetof because of CO:RE relocations
// ((TYPE *)0)->MEMBER gives us the right offset with relocations when
// the struct is marked with preserve_access_index, __builtin_offsetof does not
#define offsetof(TYPE, MEMBER) ((u64) & ((TYPE *)0)->MEMBER)

#define container_of(ptr, type, member)                                        \
  ({                                                                           \
    void *__mptr = (void *)(ptr);                                              \
    ((type *)(__mptr - offsetof(type, member)));                               \
  })

#define inline __attribute__((always_inline))

struct qstr {
	union {
		struct {
			u32 hash;
			u32 len;
		};
		u64 hash_len;
	};
	const unsigned char *name;
};

struct dentry {
	struct qstr d_name;
	struct dentry *d_parent;
};

struct vfsmount {
	struct dentry *mnt_root;
};

struct mount {
	struct mount *mnt_parent;
	struct dentry *mnt_mountpoint;
	struct vfsmount mnt;
};

struct path {
	struct vfsmount *mnt;
	struct dentry *dentry;
};

struct file {
	struct path f_path;
};

struct mm_struct {
	u64 arg_start;
	u64 arg_end;
	struct file *exe_file;
};

struct fdtable {
	unsigned int max_fds;
	struct file **fd;
	long unsigned int *open_fds;
};

typedef struct {
	int counter;
} atomic_t;

struct files_struct {
	atomic_t count;
	struct fdtable *fdt;
};

struct task_struct {
	struct mm_struct *mm;
	s32 pid;
	s32 tgid;
	u64 start_time;
	char comm[16];
	struct files_struct *files;
	struct task_struct *real_parent;
	struct task_struct *group_leader;
};

inline struct mm_struct **task_struct_mm(struct task_struct *task)
{
	return &task->mm;
}

inline s32 *task_struct_pid(struct task_struct *task)
{
	return &task->pid;
}

inline s32 *task_struct_tgid(struct task_struct *task)
{
	return &task->tgid;
}

inline u64 *task_struct_start_time(struct task_struct *task)
{
	return &task->start_time;
}

inline char (*task_struct_comm(struct task_struct *task))[16] {
	return &task->comm;
}

inline struct files_struct **task_struct_files(struct task_struct *task)
{
	return &task->files;
}

inline struct task_struct **task_struct_real_parent(struct task_struct *task)
{
	return &task->real_parent;
}

inline struct task_struct **task_struct_group_leader(struct task_struct *task)
{
	return &task->group_leader;
}

inline u64 *mm_struct_arg_start(struct mm_struct *mm)
{
	return &mm->arg_start;
}

inline u64 *mm_struct_arg_end(struct mm_struct *mm)
{
	return &mm->arg_end;
}

inline struct file **mm_struct_exe_file(struct mm_struct *mm)
{
	return &mm->exe_file;
}

inline struct path *file_f_path(struct file *file)
{
	return &file->f_path;
}

inline struct dentry **path_dentry(struct path *path)
{
	return &path->dentry;
}

inline struct vfsmount **path_mnt(struct path *path)
{
	return &path->mnt;
}

inline struct dentry **vfsmount_mnt_root(struct vfsmount *mnt)
{
	return &mnt->mnt_root;
}

inline struct mount *vfsmount_container(struct vfsmount *mnt)
{
	return container_of(mnt, struct mount, mnt);
}

inline struct qstr *dentry_d_name(struct dentry *dentry)
{
	return &dentry->d_name;
}

inline struct dentry **dentry_d_parent(struct dentry *dentry)
{
	return &dentry->d_parent;
}

inline u32 *qstr_len(struct qstr *qstr)
{
	return &qstr->len;
}

inline const unsigned char **qstr_name(struct qstr *qstr)
{
	return &qstr->name;
}

inline struct mount **mount_mnt_parent(struct mount *mnt)
{
	return &mnt->mnt_parent;
}

inline struct dentry **mount_mnt_mountpoint(struct mount *mnt)
{
	return &mnt->mnt_mountpoint;
}

inline struct vfsmount *mount_mnt(struct mount *mnt)
{
	return &mnt->mnt;
}

inline struct fdtable **files_struct_fdt(struct files_struct *files)
{
	return &files->fdt;
}

inline atomic_t *files_struct_count(struct files_struct *files)
{
	return &files->count;
}

inline unsigned int *fdtable_max_fds(struct fdtable *fdt)
{
	return &fdt->max_fds;
}

inline struct file ***fdtable_fd(struct fdtable *fdt)
{
	return &fdt->fd;
}

inline long unsigned int **fdtable_open_fds(struct fdtable *fdt)
{
	return &fdt->open_fds;
}

#if defined(__bpf__)
#pragma clang attribute pop
#endif

#endif
