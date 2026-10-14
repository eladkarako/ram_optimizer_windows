#![cfg(target_os = "windows")]
//! Program designed to only compile for Windows OS due to exclusive use of Windows API.

use clap::{Parser, ValueEnum};
use parking_lot::RwLock;
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use sysinfo::{Process, System};

// ============================================================================
// Windows API Bindings
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub enum MemoryTrimMethod {
    EmptyWorkingSet = 0,
    SetWorkingSet = 1,
    SetWorkingSetEx = 2,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TrimMethod {
    #[value(name = "empty")]
    Empty,
    #[value(name = "set")]
    Set,
    #[value(name = "ex")]
    Ex,
}

impl From<TrimMethod> for MemoryTrimMethod {
    fn from(method: TrimMethod) -> Self {
        match method {
            TrimMethod::Empty => MemoryTrimMethod::EmptyWorkingSet,
            TrimMethod::Set => MemoryTrimMethod::SetWorkingSet,
            TrimMethod::Ex => MemoryTrimMethod::SetWorkingSetEx,
        }
    }
}

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const PROCESS_SET_QUOTA: u32 = 0x0100;
const QUOTA_LIMITS_HARDWS_MAX_DISABLE: u32 = 0x00000008;
const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x00001000;

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

    fn FormatMessageW(
        dwFlags: u32,
        lpSource: *mut c_void,
        dwMessageId: u32,
        dwLanguageId: u32,
        lpBuffer: *mut u16,
        nSize: u32,
        Arguments: *mut c_void,
    ) -> u32;
}

// ============================================================================
// Error Handling
// ============================================================================

fn get_last_error_message() -> String {
    let error_code = unsafe { GetLastError() };

    if error_code == 0 {
        return "success".to_string();
    }

    unsafe {
        let mut buffer: [u16; 512] = [0; 512];
        let msg_len = FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            std::ptr::null_mut(),
            error_code,
            0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            std::ptr::null_mut(),
        );

        if msg_len > 0 {
            String::from_utf16_lossy(&buffer[..msg_len as usize])
                .trim_end()
                .to_string()
        } else {
            format!("Error code: {}", error_code)
        }
    }
}

// ============================================================================
// Process Display Formatting
// ============================================================================

fn build_command_string(
    cmd: &[impl AsRef<std::ffi::OsStr>]
) -> String {
    cmd.iter()
        .map(|s| s.as_ref().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_process_display(process: &Process) -> String {
    let cmd = process.cmd();
    if !cmd.is_empty() {
        build_command_string(cmd)
    } else {
        process.name().to_string_lossy().to_string()
    }
}

// ============================================================================
// Process Enumeration
// ============================================================================

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
// Windows API Process Operations
// ============================================================================

fn open_process_handle(pid: u32) -> Option<*mut c_void> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA,
            false,
            pid,
        );
        if handle.is_null() { None } else { Some(handle) }
    }
}

fn trim_working_set(
    handle: *mut c_void,
    method: MemoryTrimMethod,
) -> bool {
    let result = unsafe {
        match method {
            MemoryTrimMethod::EmptyWorkingSet => {
                EmptyWorkingSet(handle)
            }
            MemoryTrimMethod::SetWorkingSet => {
                SetProcessWorkingSetSize(
                    handle,
                    usize::MAX,
                    usize::MAX,
                )
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

fn close_process_handle(handle: *mut c_void) {
    unsafe {
        CloseHandle(handle);
    }
}

fn trim_process_memory(
    pid: u32,
    method: MemoryTrimMethod,
) -> (bool, String) {
    let handle = match open_process_handle(pid) {
        Some(h) => h,
        None => return (false, get_last_error_message()),
    };

    let success = trim_working_set(handle, method);
    close_process_handle(handle);

    if success {
        (true, "success".to_string())
    } else {
        (false, get_last_error_message())
    }
}

// ============================================================================
// Process Batch Management
// ============================================================================

struct ProcessBatch {
    processes: RwLock<Vec<(u32, String)>>,
    counter: AtomicUsize,
    index: AtomicUsize,
    max_counter_digits: usize,
}

impl ProcessBatch {
    fn new(
        processes: Vec<(u32, String)>,
        max_counter_digits: usize,
    ) -> Self {
        ProcessBatch {
            processes: RwLock::new(processes),
            counter: AtomicUsize::new(0),
            index: AtomicUsize::new(0),
            max_counter_digits,
        }
    }

    fn take_next(&self) -> Option<(u32, String)> {
        let processes = self.processes.read();
        let idx = self.index.fetch_add(1, Ordering::SeqCst);

        if idx < processes.len() {
            Some(processes[idx].clone())
        } else {
            None
        }
    }

    fn record_result(
        &self,
        pid: u32,
        display: &str,
        success: bool,
        message: &str,
    ) {
        let counter =
            self.counter.fetch_add(1, Ordering::SeqCst) + 1;
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
// Worker Thread Logic
// ============================================================================

fn process_batch_worker(
    batch: Arc<ProcessBatch>,
    method: MemoryTrimMethod,
) {
    while let Some((pid, display)) = batch.take_next() {
        let (success, message) = trim_process_memory(pid, method);
        batch.record_result(pid, &display, success, &message);
    }
}

// ============================================================================
// Thread Orchestration
// ============================================================================

fn calculate_thread_count(num_processes: usize) -> usize {
    let cpu_count = num_cpus::get();
    let thread_count = cpu_count.min(num_processes);
    thread_count.max(1).min(16)
}

fn spawn_worker_threads(
    batch: Arc<ProcessBatch>,
    num_threads: usize,
    method: MemoryTrimMethod,
) -> Vec<thread::JoinHandle<()>> {
    (0..num_threads)
        .map(|_| {
            let batch = Arc::clone(&batch);
            thread::spawn(move || {
                process_batch_worker(batch, method);
            })
        })
        .collect()
}

fn wait_for_threads(handles: Vec<thread::JoinHandle<()>>) {
    for handle in handles {
        let _ = handle.join();
    }
}

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug, Clone)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), long_about = None)]
#[command(after_help = concat!(
"Repository:    ", env!("CARGO_PKG_REPOSITORY"), "\n",
))]
struct Cli {
    #[arg(
        short,
        long,
        value_enum,
        default_value = "empty",
        help = "Trimming method: empty (EmptyWorkingSet), set (SetProcessWorkingSetSize), or ex (SetProcessWorkingSetSizeEx)"
    )]
    method: TrimMethod,

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

fn main() {
    let cli = Cli::parse();
    let processes = enumerate_processes();

    if processes.is_empty() {
        eprintln!("No processes found");
        return;
    }

    println!("Counter\tStatus\tMessage\tDisplay\tPID");

    let num_processes = processes.len();
    let max_counter_digits = num_processes.to_string().len();
    let num_threads = cli
        .threads
        .map(|t| t.max(1).min(16))
        .unwrap_or_else(|| calculate_thread_count(num_processes));

    let batch =
        Arc::new(ProcessBatch::new(processes, max_counter_digits));
    let method: MemoryTrimMethod = cli.method.into();
    let handles = spawn_worker_threads(batch, num_threads, method);

    wait_for_threads(handles);
}
