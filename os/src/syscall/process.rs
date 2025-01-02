//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translated_byte_buffer, MapPermission, PageTableEntry, VirtAddr, VirtPageNum},  task::{
        change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER
    }, timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

fn write_translated_byte_buffer(token: usize, src: *const u8, len: usize, dst: *const u8) -> usize {
    let mut dst_slice_vec = translated_byte_buffer(token, dst, len);

    for (idx, dst_slice) in dst_slice_vec.iter_mut().enumerate() {
        let src_ptr = src.wrapping_add(idx);
        let src_slice = unsafe { core::slice::from_raw_parts(src_ptr, dst_slice.len()) };
        dst_slice.copy_from_slice(src_slice);
    }

    len
}
/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // -1
    let us = get_time_us();
    let ref time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    write_translated_byte_buffer(
        current_user_token(), 
        time_val as *const _ as *const u8,
        core::mem::size_of::<TimeVal>(),
        _ts as *const u8,
    );
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    // -1
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let task = &inner.tasks[current];
    let current_time = get_time_ms();
    let time = current_time - task.start_time;
    let mut syscall_times = [0; MAX_SYSCALL_NUM];
    syscall_times.copy_from_slice(&task.syscall_times);
    let status = task.task_status;
    drop(inner);
    if _ti.is_null() {
        return -1;
    }
    let task_info = TaskInfo {
        status,
        syscall_times,
        time
    };

    write_translated_byte_buffer(current_user_token(), &task_info as *const _ as *const u8, size_of::<TaskInfo>(), _ti as *const u8);
    0
}

pub fn get_pte(vpn: VirtPageNum) -> Option<PageTableEntry> {
    let task_manager_inner = TASK_MANAGER.inner.exclusive_access();
    let current_task_id = task_manager_inner.current_task;
    let current_task = &task_manager_inner.tasks[current_task_id];
    let pte = current_task.memory_set.translate(vpn);
    drop(task_manager_inner);
    pte
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    // -1
    if start % PAGE_SIZE != 0 || port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    let start_vpn_usize: usize = start_vpn.into();
    let end_vpn_usize: usize = end_vpn.into();
    for vpn in start_vpn_usize..end_vpn_usize {
        if let Some(pte) = get_pte(vpn.into()) {
            if pte.is_valid() {
                return -1;
            }
        }
    }

    let mut task_manager_inner = TASK_MANAGER.inner.exclusive_access();
    let current_task_id = task_manager_inner.current_task;
    let current_task = &mut task_manager_inner.tasks[current_task_id];
    current_task.memory_set.insert_framed_area(start_vpn.into(), end_vpn.into(), 
        MapPermission::from_bits_truncate((port << 1) as u8) | MapPermission::U);
    drop(task_manager_inner);

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    // -1
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);
    let mut task_manager_inner = TASK_MANAGER.inner.exclusive_access();
    let current_task_id = task_manager_inner.current_task;
    let current_task = &mut task_manager_inner.tasks[current_task_id];
    if !current_task.memory_set.remove_framed_area(start_va, end_va) {
        return -1;
    }else{
        return 0;
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
