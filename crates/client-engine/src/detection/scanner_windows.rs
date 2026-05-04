//! Windows process scanner for supported game detection.
//! The scanner is strictly allowlist-based and process-external.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedGame {
    pub game_id: String,
    pub executable_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessEntry {
    pub process_id: u32,
    pub executable_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionMatch {
    pub process_id: u32,
    pub game_id: String,
    pub executable_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanErrorCode {
    PermissionDenied,
    SnapshotUnavailable,
    EnumerationFailed,
    UnsupportedPlatform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanError {
    pub code: ScanErrorCode,
    pub message: String,
}

impl ScanError {
    fn new(code: ScanErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.message, self.code)
    }
}

impl std::error::Error for ScanError {}

pub trait ProcessEnumerator {
    fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsProcessEnumerator;

pub fn match_allowlist<'a>(
    process_executable: &str,
    allowlist: &'a [SupportedGame],
) -> Option<&'a SupportedGame> {
    let normalized_process_name = normalize_executable(process_executable);
    allowlist.iter().find(|entry| {
        entry.enabled && normalize_executable(&entry.executable_name) == normalized_process_name
    })
}

pub fn scan_processes(allowlist: &[SupportedGame]) -> Result<Vec<DetectionMatch>, ScanError> {
    scan_processes_with(&WindowsProcessEnumerator, allowlist)
}

pub fn scan_processes_with<E: ProcessEnumerator>(
    enumerator: &E,
    allowlist: &[SupportedGame],
) -> Result<Vec<DetectionMatch>, ScanError> {
    let processes = enumerator.enumerate()?;
    let mut matches = Vec::new();

    for process in processes {
        if let Some(game) = match_allowlist(&process.executable_name, allowlist) {
            matches.push(DetectionMatch {
                process_id: process.process_id,
                game_id: game.game_id.clone(),
                executable_name: game.executable_name.clone(),
            });
        }
    }

    Ok(matches)
}

fn normalize_executable(executable_name: &str) -> String {
    executable_name.trim().to_ascii_lowercase()
}

#[cfg(windows)]
impl ProcessEnumerator for WindowsProcessEnumerator {
    fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
        enumerate_windows_processes()
    }
}

#[cfg(not(windows))]
impl ProcessEnumerator for WindowsProcessEnumerator {
    fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
        Err(ScanError::new(
            ScanErrorCode::UnsupportedPlatform,
            "Windows process scanner is only available on Windows targets",
        ))
    }
}

#[cfg(windows)]
fn enumerate_windows_processes() -> Result<Vec<ProcessEntry>, ScanError> {
    use std::mem::size_of;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_NO_MORE_FILES, ERROR_PARTIAL_COPY,
        INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        let code = unsafe { GetLastError() };
        return Err(map_windows_error(
            ScanErrorCode::SnapshotUnavailable,
            code,
            "CreateToolhelp32Snapshot failed",
        ));
    }

    let mut entries = Vec::new();
    let mut process: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    process.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    let first_ok = unsafe { Process32FirstW(snapshot, &mut process) };
    if first_ok == 0 {
        let code = unsafe { GetLastError() };
        unsafe {
            CloseHandle(snapshot);
        }

        if code == ERROR_NO_MORE_FILES {
            return Ok(entries);
        }

        return Err(map_windows_error(
            ScanErrorCode::EnumerationFailed,
            code,
            "Process32FirstW failed",
        ));
    }

    loop {
        let executable_name = utf16z_to_string(&process.szExeFile);
        if !executable_name.is_empty() {
            entries.push(ProcessEntry {
                process_id: process.th32ProcessID,
                executable_name: normalize_executable(&executable_name),
            });
        }

        let next_ok = unsafe { Process32NextW(snapshot, &mut process) };
        if next_ok == 0 {
            let code = unsafe { GetLastError() };
            if code == ERROR_NO_MORE_FILES {
                break;
            }

            unsafe {
                CloseHandle(snapshot);
            }

            return Err(map_windows_error(
                ScanErrorCode::EnumerationFailed,
                code,
                "Process32NextW failed",
            ));
        }
    }

    unsafe {
        CloseHandle(snapshot);
    }

    fn map_windows_error(default: ScanErrorCode, code: u32, context: &str) -> ScanError {
        let mapped = match code {
            ERROR_ACCESS_DENIED | ERROR_PARTIAL_COPY => ScanErrorCode::PermissionDenied,
            _ => default,
        };
        ScanError::new(mapped, format!("{context}: Win32 error {code}"))
    }

    fn utf16z_to_string(raw: &[u16]) -> String {
        let end = raw
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(raw.len());
        String::from_utf16_lossy(&raw[..end])
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::{
        match_allowlist, scan_processes_with, ProcessEntry, ProcessEnumerator, ScanError,
        ScanErrorCode, SupportedGame,
    };

    struct StubEnumerator {
        processes: Vec<ProcessEntry>,
    }

    impl ProcessEnumerator for StubEnumerator {
        fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
            Ok(self.processes.clone())
        }
    }

    struct FailingEnumerator;

    impl ProcessEnumerator for FailingEnumerator {
        fn enumerate(&self) -> Result<Vec<ProcessEntry>, ScanError> {
            Err(ScanError {
                code: ScanErrorCode::PermissionDenied,
                message: "access denied while enumerating processes".to_string(),
            })
        }
    }

    fn allowlist() -> Vec<SupportedGame> {
        vec![
            SupportedGame {
                game_id: "ffxiv".to_string(),
                executable_name: "ffxiv_dx11.exe".to_string(),
                enabled: true,
            },
            SupportedGame {
                game_id: "swtor".to_string(),
                executable_name: "swtor.exe".to_string(),
                enabled: false,
            },
        ]
    }

    #[test]
    fn match_allowlist_is_exact_and_enabled_only() {
        let supported = allowlist();

        let matched = match_allowlist(" FFXIV_DX11.EXE ", &supported).expect("should match ffxiv");
        assert_eq!(matched.game_id, "ffxiv");

        let disabled = match_allowlist("swtor.exe", &supported);
        assert!(disabled.is_none(), "disabled title must not match");

        let unknown = match_allowlist("some-random-game.exe", &supported);
        assert!(
            unknown.is_none(),
            "unknown process must not produce false positive"
        );
    }

    #[test]
    fn scan_processes_with_ignores_unknown_processes() {
        let supported = allowlist();
        let enumerator = StubEnumerator {
            processes: vec![
                ProcessEntry {
                    process_id: 111,
                    executable_name: "explorer.exe".to_string(),
                },
                ProcessEntry {
                    process_id: 222,
                    executable_name: "ffxiv_dx11.exe".to_string(),
                },
            ],
        };

        let matches =
            scan_processes_with(&enumerator, &supported).expect("scan should succeed with stub");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].process_id, 222);
        assert_eq!(matches[0].game_id, "ffxiv");
    }

    #[test]
    fn scan_processes_with_surfaces_permission_errors() {
        let supported = allowlist();
        let error =
            scan_processes_with(&FailingEnumerator, &supported).expect_err("scan should fail");

        assert_eq!(error.code, ScanErrorCode::PermissionDenied);
    }
}
