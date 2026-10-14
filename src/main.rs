#![cfg_attr(not(target_os = "windows"), allow(dead_code))]
#![cfg_attr(not(target_os = "windows"), allow(unused_imports))]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use parking_lot::{Mutex, RwLock};
use sysinfo::{System, Process};
use clap::{Parser, ValueEnum};

#[cfg(windows)]
use std::ffi::c_void;

// ============================================================================
// Windows API Bindings
// ============================================================================

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy)]
pub enum MemoryTrimMethod {
    EmptyWorkingSet = 0,
    SetWorkingSet = 1,
    SetWorkingSetEx = 2,
}


#[cfg(windows)]
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TrimMethod {
    /// EmptyWorkingSet API
    #[value(name = "empty")]
    Empty,
    /// SetProcessWorkingSetSize API
    #[value(name = "set")]
    Set,
    /// SetProcessWorkingSetSizeEx API
    #[value(name = "ex")]
    Ex,
}

#[cfg(windows)]
impl From<TrimMethod> for MemoryTrimMethod {
    fn from(method: TrimMethod) -> Self {
        match method {
            TrimMethod::Empty => MemoryTrimMethod::EmptyWorkingSet,
            TrimMethod::Set => MemoryTrimMethod::SetWorkingSet,
            TrimMethod::Ex => MemoryTrimMethod::SetWorkingSetEx,
        }
    }
}

#[cfg(windows)]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

#[cfg(windows)]
const PROCESS_SET_QUOTA: u32 = 0x0100;

#[cfg(windows)]
const QUOTA_LIMITS_HARDWS_MAX_DISABLE: u32 = 0x00000008;

#[cfg(windows)]
unsafe extern "system" {
    fn OpenProcess(
        dwDesiredAccess: u32,
        bInheritHandle: bool,
        dwProcessId: u32,
    ) -> *mut c_void;

    fn CloseHandle(hObject: *mut c_void) -> i32;

    fn EmptyWorkingSet(hProc: *mut c_void) -> i32;

    fn SetProcessWorkingSetSize(
        hProc: *mut c_void,
        dwMin: usize,
        dwMax: usize,
    ) -> i32;

    fn SetProcessWorkingSetSizeEx(
        hProc: *mut c_void,
        dwMin: usize,
        dwMax: usize,
        Flags: u32,
    ) -> i32;

    fn GetLastError() -> u32;

    fn FormatMessageA(
        dwFlags: u32,
        lpSource: *mut c_void,
        dwMessageId: u32,
        dwLanguageId: u32,
        lpBuffer: *mut u8,
        nSize: u32,
        Arguments: *mut c_void,
    ) -> u32;
}

// ============================================================================
// Error Handling
// ============================================================================

#[cfg(windows)]
const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x00001000;

#[cfg(windows)]
fn get_error_message(error_code: u32) -> String {
    if error_code == 0 {
        return "success".to_string();
    }

    unsafe {
        let mut buffer: [u8; 512] = [0; 512];
        let msg_len = FormatMessageA(
            FORMAT_MESSAGE_FROM_SYSTEM,
            std::ptr::null_mut(),
            error_code,
            0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            std::ptr::null_mut(),
        );

        if msg_len > 0 {
            let msg = String::from_utf8_lossy(&buffer[..msg_len as usize])
                .trim_end()
                .to_string();
            msg
        } else {
            format!("Error code: {}", error_code)
        }
    }
}

#[cfg(windows)]
fn get_last_error() -> u32 {
    unsafe { GetLastError() }
}

// ============================================================================
// Process Enumeration
// ============================================================================

#[cfg(windows)]
fn format_process_display(process: &Process) -> String {
    let cmd = process
        .cmd()
        .join(std::ffi::OsStr::new(" "))
        .to_string_lossy()
        .to_string();

    if cmd.is_empty() {
        process.name().to_string_lossy().to_string()
    } else {
        cmd
    }
}

#[cfg(windows)]
fn enumerate_processes() -> Vec<(u32, String)> {
    let sys = System::new_all();
    sys.processes()
        .values()
        .map(|process| {
            let pid = process.pid().as_u32();
            let display = format_process_display(process);
            (pid, display)
        })
        .collect()
}

// ============================================================================
// Windows API Helpers
// ============================================================================

#[cfg(windows)]
fn open_process_handle(pid: u32) -> Option<*mut c_void> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA,
            false,
            pid,
        );

        if handle.is_null() {
            None
        } else {
            Some(handle)
        }
    }
}

#[cfg(windows)]
fn trim_working_set(handle: *mut c_void, method: MemoryTrimMethod) -> bool {
    let result = unsafe {
        match method {
            MemoryTrimMethod::EmptyWorkingSet => EmptyWorkingSet(handle),
            MemoryTrimMethod::SetWorkingSet => {
                SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX)
            }
            MemoryTrimMethod::SetWorkingSetEx => {
                SetProcessWorkingSetSizeEx(
                    handle,
                    usize::MAX,
                    usize::MAX,
                    QUOTA_LIMITS_HARDWS_MAX_DISABLE,
                )
            }
        }
    };

    result != 0
}

#[cfg(windows)]
fn close_process_handle(handle: *mut c_void) {
    unsafe {
        CloseHandle(handle);
    }
}

#[cfg(windows)]
fn apply_to_process(pid: u32, method: MemoryTrimMethod) -> (bool, String) {
    match open_process_handle(pid) {
        Some(handle) => {
            let success = trim_working_set(handle, method);
            close_process_handle(handle);

            if success {
                (true, "success".to_string())
            } else {
                let error_code = get_last_error();
                (false, get_error_message(error_code))
            }
        }
        None => {
            let error_code = get_last_error();
            (false, get_error_message(error_code))
        }
    }
}

// ============================================================================
// Process Batch Management
// ============================================================================

#[cfg(windows)]
struct ProcessBatch {
    processes: RwLock<Vec<(u32, String)>>,
    counter: AtomicUsize,
    max_counter_digits: usize,
    index: Mutex<usize>,
}

#[cfg(windows)]
impl ProcessBatch {
    fn new(processes: Vec<(u32, String)>, max_counter_digits: usize) -> Self {
        ProcessBatch {
            processes: RwLock::new(processes),
            counter: AtomicUsize::new(0),
            max_counter_digits,
            index: Mutex::new(0),
        }
    }

    fn take_next(&self) -> Option<(u32, String)> {
        let mut idx = self.index.lock();
        let processes = self.processes.read();

        if *idx < processes.len() {
            let (pid, name) = processes[*idx].clone();
            *idx += 1;
            Some((pid, name))
        } else {
            None
        }
    }

    fn output(&self, pid: u32, display: &str, success: bool, message: &str) {
        let counter = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        let status = if success { "V" } else { "X" };
        println!(
            "{:0width$}.\t{}\t{}\t{}\t{}",
            counter,
            status,
            message,
            display,
            pid,
            width = self.max_counter_digits
        );
    }
}

// ============================================================================
// Threading Orchestration
// ============================================================================

#[cfg(windows)]
fn calculate_thread_count(num_processes: usize) -> usize {
    let cpu_count = num_cpus::get();
    let thread_count = cpu_count.min(num_processes);
    thread_count.max(1).min(16)
}

#[cfg(windows)]
fn spawn_worker_threads(
    batch: Arc<ProcessBatch>,
    num_threads: usize,
    method: MemoryTrimMethod,
) -> Vec<thread::JoinHandle<()>> {
    (0..num_threads)
        .map(|_| {
            let batch = Arc::clone(&batch);
            thread::spawn(move || {
                while let Some((pid, display)) = batch.take_next() {
                    let (success, message) = apply_to_process(pid, method);
                    batch.output(pid, &display, success, &message);
                }
            })
        })
        .collect()
}

#[cfg(windows)]
fn wait_for_threads(handles: Vec<thread::JoinHandle<()>>) {
    for handle in handles {
        let _ = handle.join();
    }
}

// ============================================================================
// CLI Arguments
// ============================================================================

#[cfg(windows)]
#[derive(Parser)]
#[command(name = "RAM Optimizer Windows")]
#[command(about = "Trim working sets of all processes", long_about = None)]
struct Cli {
    /// API method to use for trimming
    #[arg(
        short,
        long,
        value_enum,
        default_value = "empty",
        help = "Trimming method: empty (EmptyWorkingSet), set (SetProcessWorkingSetSize), or ex (SetProcessWorkingSetSizeEx)"
    )]
    method: TrimMethod,

    /// Number of worker threads (auto-calculated if not specified)
    #[arg(
        short = 't',
        long,
        help = "Number of worker threads (1-16, default: auto)"
    )]
    threads: Option<usize>,
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[cfg(windows)]
fn main() {
    let cli = Cli::parse();

    // Enumerate all processes once
    let processes = enumerate_processes();

    if processes.is_empty() {
        eprintln!("No processes found");
        return;
    }

    println!("Counter\tStatus\tMessage\tDisplay\tPID");

    let num_processes = processes.len();
    let max_counter_digits = num_processes.to_string().len();

    let num_threads = match cli.threads {
        Some(t) => t.max(1).min(16),
        None => calculate_thread_count(num_processes),
    };

    // Create process batch
    let batch = Arc::new(ProcessBatch::new(processes, max_counter_digits));

    // Spawn worker threads
    let method: MemoryTrimMethod = cli.method.into();
    let handles = spawn_worker_threads(batch, num_threads, method);

    // Wait for completion
    wait_for_threads(handles);
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This program only runs on Windows.");
}
