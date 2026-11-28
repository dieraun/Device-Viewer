//! Windows 控制台处理。
//!
//! 发布版以 windows_subsystem = "windows" 链接：双击启动只有窗口，没有黑框。
//! 批处理模式（带参数）在启动时附着到父控制台，没有父控制台时才自己开一个，
//! 并把标准输出/错误重定向到 CONOUT，让 println! 照常可见。
//!
//! 注意：这些调用必须发生在任何标准输出使用之前——Rust 的 stdout() 在首次
//! 使用时缓存 GetStdHandle 的结果，之后再改就来不及了。

/// 带参数启动（批处理）时准备控制台，纯 GUI 启动保持无控制台。
///
/// 只附着已有的父控制台，绝不新建：双击关联文件也会带一个路径参数，那种
/// 情况下不该凭空弹出黑框。父进程没有控制台时输出交给继承来的管道。
pub fn prepare_for_args() {
    #[cfg(windows)]
    {
        if std::env::args_os().len() > 1 {
            // SAFETY: 只调用 Win32 附着控制台，不借用 Rust 内存。
            unsafe { attach_to_parent() }
        }
    }
}

#[cfg(windows)]
unsafe fn attach_to_parent() {
    use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};

    // SAFETY: 纯 Win32 调用，失败以 Result 返回。
    if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.is_ok() {
        // SAFETY: 父控制台已经附着，只重绑定缺失的标准句柄。
        unsafe { rebind_standard_streams() }
    }
}

/// 出错路径：确保能把错误信息输出给用户。
pub fn prepare_for_error() {
    #[cfg(windows)]
    {
        // SAFETY: 见 attach_or_alloc 的前置条件说明。
        unsafe { attach_or_alloc() }
    }
}

#[cfg(windows)]
unsafe fn attach_or_alloc() {
    use windows::Win32::System::Console::{
        ATTACH_PARENT_PROCESS, AllocConsole, AttachConsole, GetConsoleWindow,
    };

    // SAFETY: 无参数调用，不借用 Rust 内存；窗口句柄为空表示当前没有控制台。
    let window = unsafe { GetConsoleWindow() };
    if window.0.is_null() {
        // SAFETY: 纯 Win32 调用，失败以 Result 返回。
        if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.is_err() {
            // SAFETY: 父进程没有控制台（例如资源管理器双击），新建一个。
            let _ = unsafe { AllocConsole() };
        }
    }
    // SAFETY: 控制台此时已经存在，只写标准句柄。
    unsafe { rebind_standard_streams() }
}

#[cfg(windows)]
unsafe fn handle_is_missing(slot: windows::Win32::System::Console::STD_HANDLE) -> bool {
    // SAFETY: GetStdHandle 只返回已有句柄，不转移所有权。
    match unsafe { windows::Win32::System::Console::GetStdHandle(slot) } {
        Ok(handle) => handle.0.is_null(),
        Err(_) => true,
    }
}

#[cfg(windows)]
unsafe fn rebind_standard_streams() {
    use std::os::windows::io::AsRawHandle;

    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Console::{
        STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle,
    };

    // SAFETY: 只查询标准句柄，不释放任何资源。
    let output_missing = unsafe { handle_is_missing(STD_OUTPUT_HANDLE) };
    // SAFETY: 同上。
    let error_missing = unsafe { handle_is_missing(STD_ERROR_HANDLE) };

    if output_missing || error_missing {
        // 用 std 打开控制台设备，避免为 CreateFileW 再开一堆 windows feature。
        if let Ok(console) = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open("CONOUT$")
        {
            let handle = HANDLE(console.as_raw_handle());
            // 句柄必须活到进程结束：交给句柄表持有，不做 drop。
            std::mem::forget(console);
            // SAFETY: 句柄来自 CreateFile，有效且进程生命周期内不关闭。
            let _ = unsafe { SetStdHandle(STD_OUTPUT_HANDLE, handle) };
            // SAFETY: 同上。
            let _ = unsafe { SetStdHandle(STD_ERROR_HANDLE, handle) };
        }
    }

    // SAFETY: 只查询标准句柄，不释放任何资源。
    if unsafe { handle_is_missing(STD_INPUT_HANDLE) }
        && let Ok(console) = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONIN$")
    {
        let handle = HANDLE(console.as_raw_handle());
        // 句柄必须活到进程结束：交给句柄表持有，不做 drop。
        std::mem::forget(console);
        // SAFETY: 句柄来自 CreateFile，有效且进程生命周期内不关闭。
        let _ = unsafe { SetStdHandle(STD_INPUT_HANDLE, handle) };
    }
}
