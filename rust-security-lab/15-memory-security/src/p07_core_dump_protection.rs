//! # Lesson 07: Core Dump Protection — Prevent Secret Leakage in Crash Files
//!
//! ## The Problem
//!
//! When a process crashes (segfault, abort, etc.), the OS can write a "core dump" —
//! a snapshot of the process's entire memory space. This includes ALL secrets:
//! encryption keys, passwords, session tokens, etc.
//!
//! ```text
//! Process crashes → core dump file (core.12345)
//! ┌─────────────────────────────┐
//! │ All heap memory             │ ← includes secret keys
//! │ All stack memory            │ ← includes passwords
//! │ All mapped regions          │ ← includes TLS session keys
//! └─────────────────────────────┘
//!
//! Attacker: strings core.12345 | grep -i "BEGIN.*KEY"
//! → Found private key!
//! ```
//!
//! ## Defense Strategies
//!
//! 1. **Disable core dumps**: `setrlimit(RLIMIT_CORE, 0)` — most common
//! 2. **Set core pattern**: `/proc/sys/kernel/core_pattern` to pipe to a handler
//! 3. **Core dump filter**: `/proc/self/coredump_filter` to exclude memory regions
//! 4. **PR_SET_DUMPABLE**: `prctl(PR_SET_DUMPABLE, 0)` — disable for the process
//!
//! ## When Core Dumps ARE Useful
//!
//! Core dumps are valuable for debugging. The tradeoff:
//! - Development/debug: enable core dumps, no secrets in memory
//! - Production: disable core dumps, secrets in memory

/// Exercise 1: Implement `disable_core_dumps` — use `setrlimit` to set RLIMIT_CORE to 0.
///
/// Requirements:
/// - Use `libc::setrlimit` to set `RLIMIT_CORE` to 0
/// - Return `Ok(())` on success, `Err(String)` on failure
///
/// Hints:
/// - `libc::setrlimit(resource: libc::c_int, rlim: *const libc::rlimit)`
/// - Create `libc::rlimit { rlim_cur: 0, rlim_max: 0 }`
/// - `libc::RLIMIT_CORE` is the resource constant
pub fn disable_core_dumps() -> Result<(), String> {
    todo!("Disable core dumps using setrlimit")
}

/// Exercise 2: Implement `is_core_dump_disabled` — check if core dumps are disabled.
///
/// Requirements:
/// - Use `libc::getrlimit` to read `RLIMIT_CORE`
/// - Return `true` if the limit is 0
///
/// Hints:
/// - `libc::getrlimit(resource, &mut rlimit)` → returns 0 on success
/// - Check `rlim.rlim_cur == 0`
pub fn is_core_dump_disabled() -> bool {
    todo!("Check if core dumps are currently disabled")
}

/// Exercise 3: Implement `set_dumpable` — control the PR_SET_DUMPABLE flag.
///
/// Requirements:
/// - Use `libc::prctl(libc::PR_SET_DUMPABLE, value)` on Linux
/// - `value = 0` to disable, `value = 1` to enable
/// - Return `Ok(())` on success
///
/// Hints:
/// - `libc::prctl(libc::PR_SET_DUMPABLE, 0 as libc::c_long, 0, 0, 0)`
/// - Only works on Linux
pub fn set_dumpable(dumpable: bool) -> Result<(), String> {
    todo!("Set PR_SET_DUMPABLE flag")
}

/// Exercise 4: Implement `CoreDumpGuard` — RAII guard that disables core dumps
/// for the lifetime of the guard, re-enabling on drop.
///
/// Requirements:
/// - On creation: save current RLIMIT_CORE, set it to 0
/// - On drop: restore the saved RLIMIT_CORE
/// - `new()` returns `Result<Self, String>`
pub struct CoreDumpGuard {
    saved_limit: libc::rlimit,
}

impl CoreDumpGuard {
    pub fn new() -> Result<Self, String> {
        todo!("Save current limit and disable core dumps")
    }
}

impl Drop for CoreDumpGuard {
    fn drop(&mut self) {
        // Restore the saved core dump limit
        unsafe {
            libc::setrlimit(libc::RLIMIT_CORE, &self.saved_limit);
        }
    }
}

/// Exercise 5: Implement `get_core_dump_filter` — read the coredump_filter setting.
///
/// On Linux, `/proc/self/coredump_filter` is a bitmask:
/// - bit 0: anonymous private memory
/// - bit 1: anonymous shared memory
/// - bit 2: file-backed private memory
/// - bit 3: file-backed shared memory
/// - bit 4: ELF headers
/// - bit 5: hugetlb pages
///
/// Return the current filter value, or 0 if not available.
///
/// Hints:
/// - Use `std::fs::read_to_string("/proc/self/coredump_filter")`
/// - Parse the hex value
pub fn get_core_dump_filter() -> u32 {
    todo!("Read /proc/self/coredump_filter")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_core_dumps() {
        // This may fail without privileges, but should not panic
        let _ = disable_core_dumps();
    }

    #[test]
    fn test_is_core_dump_disabled() {
        // Just verify it runs without panic
        let _disabled = is_core_dump_disabled();
    }

    #[test]
    fn test_core_dump_guard_creates() {
        // Guard should disable core dumps for its lifetime
        if let Ok(guard) = CoreDumpGuard::new() {
            assert!(is_core_dump_disabled(), "Core dumps should be disabled while guard exists");
            drop(guard);
        }
        // After guard is dropped, the original setting is restored
    }

    #[test]
    fn test_core_dump_guard_restores() {
        let was_disabled = is_core_dump_disabled();
        {
            let _guard = CoreDumpGuard::new();
            // Core dumps are disabled here
        }
        // After guard drop, should be restored to previous state
        // (We can't assert exact equality because we don't know the initial state)
        let _ = was_disabled;
    }

    #[test]
    fn test_set_dumpable_false() {
        // This may fail without privileges
        let _ = set_dumpable(false);
    }

    #[test]
    fn test_set_dumpable_true() {
        let _ = set_dumpable(true);
    }

    #[test]
    fn test_get_core_dump_filter() {
        // On Linux, this should return a value; on other platforms, 0
        let filter = get_core_dump_filter();
        // Filter is a bitmask, so it's always >= 0 (u32)
        let _ = filter;
    }
}
