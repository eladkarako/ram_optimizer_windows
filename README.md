#### Windows OS only!

## ram_optimizer_windows

releases RAM from other programs. just like `FreeRAM-XP-Pro`.  

you can try it with the RAM monitoring tool (works in any OS) - https://github.com/eladkarako/ram_report

<hr/>

<details><summary><h3>👇  download</h3></summary>

### download (direct URLs - latest release)

#### Windows
- [![](resources/logos/64.png) ![](resources/logos/windows.png) x86_64-pc-windows-msvc.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/x86_64-pc-windows-msvc.zip)
- [![](resources/logos/64.png) ![](resources/logos/windows.png) aarch64-pc-windows-msvc.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/aarch64-pc-windows-msvc.zip)
- [![](resources/logos/32.png) ![](resources/logos/windows.png) i686-pc-windows-msvc.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/i686-pc-windows-msvc.zip)

#### Android (NDK `v30.0.16248370`, SDK `v21`)
- [![](resources/logos/64.png) ![](resources/logos/android.png) aarch64-linux-android.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/aarch64-linux-android.zip)
- [![](resources/logos/32.png) ![](resources/logos/android.png) armv7-linux-androideabi.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/armv7-linux-androideabi.zip)
- [![](resources/logos/64.png) ![](resources/logos/android.png) x86_64-linux-android.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/x86_64-linux-android.zip)
- [![](resources/logos/32.png) ![](resources/logos/android.png) i686-linux-android.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/i686-linux-android.zip)

#### Linux (gnu)
- [![](resources/logos/64.png) ![](resources/logos/linux.png) x86_64-unknown-linux-gnu.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/x86_64-unknown-linux-gnu.zip)
- [![](resources/logos/32.png) ![](resources/logos/linux.png) i686-unknown-linux-gnu.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/i686-unknown-linux-gnu.zip)
- [![](resources/logos/64.png) ![](resources/logos/linux.png) aarch64-unknown-linux-gnu.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/aarch64-unknown-linux-gnu.zip)
- [![](resources/logos/32.png) ![](resources/logos/linux.png) armv7-unknown-linux-gnueabi.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/armv7-unknown-linux-gnueabi.zip)
- [![](resources/logos/32.png) ![](resources/logos/linux.png) armv7-unknown-linux-gnueabihf.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/armv7-unknown-linux-gnueabihf.zip)

#### Linux (musl)
- [![](resources/logos/64.png) ![](resources/logos/linux.png) x86_64-unknown-linux-musl.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/x86_64-unknown-linux-musl.zip)
- [![](resources/logos/32.png) ![](resources/logos/linux.png) i686-unknown-linux-musl.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/i686-unknown-linux-musl.zip)
- [![](resources/logos/64.png) ![](resources/logos/linux.png) aarch64-unknown-linux-musl.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/aarch64-unknown-linux-musl.zip)
- [![](resources/logos/32.png) ![](resources/logos/linux.png) armv7-unknown-linux-musleabi.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/armv7-unknown-linux-musleabi.zip)
- [![](resources/logos/32.png) ![](resources/logos/linux.png) armv7-unknown-linux-musleabihf.zip](https://github.com/eladkarako/ram_optimizer_windows/releases/latest/download/armv7-unknown-linux-musleabihf.zip)

### other
- [changelog.txt](https://github.com/eladkarako/ram_report/releases/latest/download/changelog.txt)
- [version.txt](https://github.com/eladkarako/ram_report/releases/latest/download/version.txt)

</details>

<hr/>

### `--help`

```txt
Trim working sets of all processes

Usage: ram_optimizer_windows.exe [OPTIONS]

Options:
  -m, --method <METHOD>
          Trimming method: empty (EmptyWorkingSet), set (SetProcessWorkingSetSize), or ex (SetProcessWorkingSetSizeEx)

          Possible values:
          - empty: EmptyWorkingSet API
          - set:   SetProcessWorkingSetSize API
          - ex:    SetProcessWorkingSetSizeEx API

          [default: empty]

  -t, --threads <THREADS>
          Number of worker threads (1-16, default: auto)

  -h, --help
          Print help (see a summary with '-h')
```

<hr/>

### the output format is TSV like

your list would be full, I just took few to let you know how it looks like.
- i extract full CMD whenever possible (when not possible it might be in `PATH`)
- the actual error is included (but no error number), usually you'll solve it by running as an admin, or as NT_SYSTEM (see below)

```
Counter Status  Message Display PID
.....
121.    X       Access is denied.       svchost.exe     2556
.....
125.    X       Access is denied.       svchost.exe     2100
.....
140.    V       success C:\WINDOWS\system32\DllHost.exe /Processid:{3EB3C877-1F16-487C-9050-104DBCD66683}     9136
.....
175.    V       success ram_optimizer_windows.exe       4400
.....
183.    V       success C:\WINDOWS\Explorer.EXE 11104
.....
240.    V       success C:\Windows\System32\ShellHost.exe       18284
.....
```

as the calls are run by a thread, and those run in parallel,  
the order would change. that's fine you can always trim the line number and sort by `V`/`X`, errors, path, or even PID.  

<hr/>

![](resources/logos/app.png)

### for Windows OS only.

allows you to run either  

- EmptyWorkingSet - https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-emptyworkingset
- SetProcessWorkingSetSize - https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-setprocessworkingsetsize
- SetProcessWorkingSetSizeEx - https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-setprocessworkingsetsizeex

on all the currently running applications.

the description of those Windows API functions is  
"Removes as many pages as possible from the working set of the specified process."  
either of those functions will ask a process to reduce its RAM usage,  
this is usually result with being ignored or dropped few MB of memory,  
the most I've got is from Firefox browser, about 1/7 of the tatal amount of physical RAM.  

the applications will restore their RAM eventually,  
but it normally gives you several minute to an hour,  
until it happens. these are the functions used by various "memory optimization" programs,  
such as the good old FreeRAM-XP-Pro.

I've tries to improve the overall mechanism by parallel executing the API,  
by default it will use same amount of parallelism as the amount of CPUs you have.  

<hr/>

## Tip:

those Windows API calls would work on application started by same credentials as you probably,  
if you are an administrator it will work on more applications,  
try to run the program as `NT AUTHORITY\SYSTEM` user - the System account,  
you only need to be a normal administrator and download https://live.sysinternals.com/Files/PsExec.zip  
place `PsExec64.exe` in same directory as `ram_optimizer_windows.exe`,  
and run (or write a batch file that does that for you):  

```
::@echo off
chcp 65001 1>nul 2>nul
pushd "%~sdp0"

call "PsExec64.exe" -accepteula -nobanner -w "%CD%" -s "%CD%\ram_optimizer_windows.exe"

pause
exit /b 0
```

<hr/>

this is a free and open-source program that works across all OS,  
built with the assist of `GitHub's Copilot` and `Claude Haiku 4.5`,  
logos and infographics generated with `GPT Image 2`.  
written in Rust, with JetBrains RustRover IDE Community.  

enjoy!

<br/>
<hr/>

<a href="https://paypal.me/31adkarak0" target="_blank" rel="noopener noreferrer"><img src="resources/logos/paypal1.svg"><br /><img src="resources/logos/paypal2.png"></a>
