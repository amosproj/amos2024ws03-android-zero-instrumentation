#ifndef __RELOCATION_HELPERS_H__
#define __RELOCATION_HELPERS_H__

typedef long long unsigned int u64;
typedef unsigned int u32;
typedef int s32;

struct mm_struct {
  u64 arg_start;
  u64 arg_end;
} __attribute__((preserve_access_index));

struct task_struct {
  struct mm_struct *mm;
  s32 pid;
  s32 tgid;
  u64 start_time;
  char comm[16];
  struct task_struct *real_parent;
  struct task_struct *group_leader;
} __attribute__((preserve_access_index));

struct mm_struct **task_struct_mm(struct task_struct *task);
s32 *task_struct_pid(struct task_struct *task);
s32 *task_struct_tgid(struct task_struct *task);
u64 *task_struct_start_time(struct task_struct *task);
char (*task_struct_comm(struct task_struct *task))[16];
struct task_struct **task_struct_real_parent(struct task_struct *task);
struct task_struct **task_struct_group_leader(struct task_struct *task);

u64* mm_struct_arg_start(struct mm_struct *mm);
u64* mm_struct_arg_end(struct mm_struct *mm);

#endif
