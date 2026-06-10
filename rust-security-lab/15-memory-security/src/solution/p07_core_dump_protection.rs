//! # Lesson 07: Core Dump Protection (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Disable core dumps using setrlimit(RLIMIT_CORE, 0).
pub fn disable_core_dumps() -> Result<(), String> {
    let rlimit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let result = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlimit) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

/// Check if core dumps are currently disabled (RLIMIT_CORE == 0).
pub fn is_core_dump_disabled() -> bool {
    let mut rlimit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let result = unsafe { libc::getrlimit(libc::RLIMIT_CORE, &mut rlimit) };
    result == 0 && rlimit.rlim_cur == 0
}

/// Set the PR_SET_DUMPABLE flag (Linux only).
pub fn set_dumpable(dumpable: bool) -> Result<(), String> {
    let value = if dumpable { 1 } else { 0 } as libc::c_long;
    let result = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, value, 0, 0, 0) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

/// RAII guard that disables core dumps for its lifetime.
pub struct CoreDumpGuard {
    saved_limit: libc::rlimit,
}

impl CoreDumpGuard {
    pub fn new() -> Result<Self, String> {
        // Save current limit
        let mut saved = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        let result = unsafe { libc::getrlimit(libc::RLIMIT_CORE, &mut saved) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }

        // Disable core dumps
        let zero = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        let result = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &zero) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }

        Ok(Self { saved_limit: saved })
    }
}

impl Drop for CoreDumpGuard {
    fn drop(&mut self) {
        unsafe {
            libc::setrlimit(libc::RLIMIT_CORE, &self.saved_limit);
        }
    }
}

/// Read the coredump_filter setting from /proc/self/coredump_filter.
pub fn get_core_dump_filter() -> u32 {
    std::fs::read_to_string("/proc/self/coredump_filter")
        .ok()
        .and_then(|s| {
            let trimmed = s.trim();
            if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                u32::from_str_radix(&trimmed[2..], 16).ok()
            } else {
                u32::from_str_radix(trimmed, 16).ok()
            }
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_core_dumps() {
        let _ = disable_core_dumps();
    }

    #[test]
    fn test_is_core_dump_disabled() {
        let _disabled = is_core_dump_disabled();
    }

    #[test]
    fn test_core_dump_guard_creates() {
        if let Ok(guard) = CoreDumpGuard::new() {
            assert!(is_core_dump_disabled(), "Core dumps should be disabled while guard exists");
            drop(guard);
        }
    }

    #[test]
    fn test_core_dump_guard_restores() {
        let was_disabled = is_core_dump_disabled();
        {
            let _guard = CoreDumpGuard::new();
        }
        let _ = was_disabled;
    }

    #[test]
    fn test_set_dumpable_false() {
        let _ = set_dumpable(false);
    }

    #[test]
    fn test_set_dumpable_true() {
        let _ = set_dumpable(true);
    }

    #[test]
    fn test_get_core_dump_filter() {
        let filter = get_core_dump_filter();
        let _ = filter;
    }
}
