#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::Path;
use tauri::{Emitter, Manager};

const SKIP_DIRS: [&str; 3] = ["node_modules", "target", "dist"];
const MAX_FILES: usize = 30_000;

/// 명령줄 인자를 절대경로("/" 구분)로 변환한다.
fn abs_arg(arg: &str, cwd: &Path) -> String {
    let p = Path::new(arg);
    let full = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
    let full = fs::canonicalize(&full).unwrap_or(full);
    let s = full.to_string_lossy().to_string();
    s.strip_prefix(r"\\?\").unwrap_or(&s).replace('\\', "/")
}

fn first_path(args: &[String], cwd: &Path) -> Option<String> {
    args.iter()
        .skip(1)
        .find(|a| !a.starts_with('-'))
        .map(|a| abs_arg(a, cwd))
}

#[tauri::command]
fn launch_path() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    first_path(&args, &std::env::current_dir().ok()?)
}

#[tauri::command]
fn path_kind(path: String) -> &'static str {
    let p = Path::new(&path);
    if p.is_dir() { "dir" } else if p.is_file() { "file" } else { "none" }
}

/// 폴더 안의 모든 파일을 root 기준 상대경로("/" 구분)로 반환한다.
#[tauri::command]
fn scan_dir(root: String) -> Result<Vec<String>, String> {
    let root_path = Path::new(&root);
    let mut out = Vec::new();
    let mut stack = vec![root_path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if dir == root_path => return Err(e.to_string()),
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            let path = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                stack.push(path);
            } else if ft.is_file() {
                if let Ok(rel) = path.strip_prefix(root_path) {
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
                if out.len() >= MAX_FILES {
                    return Ok(out);
                }
            }
        }
    }
    Ok(out)
}

#[tauri::command]
fn read_text(path: String) -> Result<String, String> {
    let s = fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
    Ok(s.trim_start_matches('\u{feff}').to_string())
}

#[tauri::command]
fn write_text(path: String, text: String) -> Result<(), String> {
    fs::write(&path, text).map_err(|e| format!("{path}: {e}"))
}

#[tauri::command]
fn read_bytes(path: String) -> Result<tauri::ipc::Response, String> {
    fs::read(&path)
        .map(tauri::ipc::Response::new)
        .map_err(|e| format!("{path}: {e}"))
}

/// 설치된 mdr.exe 의 폴더를 사용자 PATH 에 한 번 등록해 `mdr .` 가 동작하게 한다.
/// (개발 빌드가 만드는 target 폴더는 등록하지 않는다.)
#[cfg(windows)]
fn ensure_in_path() {
    use winreg::{enums::*, types::ToRegValue, RegKey};

    let Ok(exe) = std::env::current_exe() else { return };
    let Some(dir) = exe.parent().map(|d| d.to_string_lossy().to_string()) else { return };
    if dir.contains("\\target\\") {
        return;
    }
    let Ok(env) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
    else {
        return;
    };
    let cur: String = env.get_value("Path").unwrap_or_default();
    let norm = |s: &str| s.trim_end_matches('\\').to_lowercase();
    if cur.split(';').any(|p| norm(p) == norm(&dir)) {
        return;
    }
    let new = if cur.is_empty() {
        dir
    } else {
        format!("{};{}", cur.trim_end_matches(';'), dir)
    };
    let mut value = new.to_reg_value();
    value.vtype = REG_EXPAND_SZ;
    if env.set_raw_value("Path", &value).is_err() {
        return;
    }
    // 이미 떠 있는 탐색기/터미널이 변경을 알도록 알린다.
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    let param: Vec<u16> = "Environment\0".encode_utf16().collect();
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            param.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            2000,
            std::ptr::null_mut(),
        );
    }
}

fn main() {
    #[cfg(windows)]
    ensure_in_path();

    tauri::Builder::default()
        // 이미 실행 중이면 새 창 대신 기존 창에 경로를 전달한다.
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            if let Some(p) = first_path(&argv, Path::new(&cwd)) {
                let _ = app.emit("open-path", p);
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            launch_path, path_kind, scan_dir, read_text, write_text, read_bytes
        ])
        .run(tauri::generate_context!())
        .expect("error while running MdReader");
}
