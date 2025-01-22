#include "relocation_helpers.h"

__attribute__((always_inline)) int magic_number(int x) {
  return x - 1;
}

__attribute__((always_inline)) struct mm_struct **
task_struct_mm(struct task_struct *task) {
  return &task->mm;
}

__attribute__((always_inline)) s32 *task_struct_pid(struct task_struct *task) {
  return &task->pid;
}

__attribute__((always_inline)) s32 *task_struct_tgid(struct task_struct *task) {
  return &task->tgid;
}

__attribute__((always_inline)) u64 *
task_struct_start_time(struct task_struct *task) {
  return &task->start_time;
}

__attribute__((always_inline)) char (
    *task_struct_comm(struct task_struct *task))[16] {
  return &task->comm;
}

__attribute__((always_inline)) struct task_struct **
task_struct_real_parent(struct task_struct *task) {
  return &task->real_parent;
}

__attribute__((always_inline)) struct task_struct **
task_struct_group_leader(struct task_struct *task) {
  return &task->group_leader;
}

__attribute__((always_inline)) u64 *mm_struct_arg_start(struct mm_struct *mm) {
  return &mm->arg_start;
}

__attribute__((always_inline)) u64 *mm_struct_arg_end(struct mm_struct *mm) {
  return &mm->arg_end;
}