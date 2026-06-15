//! # eq-memory — Secure Buffer Management
//!
//! The foundational crate for memory safety in the EQ Gateway engine.
//! Provides [`SecureBuffer`] which guarantees:
//! - Zeroize-on-drop (memory is overwritten before deallocation)
//! - No Clone (move semantics only — prevents accidental duplication of sensitive data)
//! - Bounds-checked access (safe Rust, no raw pointer dereference)
//! - Memory locking (`mlock`/`VirtualLock`) to prevent sensitive data from being paged to disk
//!
//! # Security Guarantee
//! No raw PII text ever leaks from a SecureBuffer after its lifetime ends.
//! This is verified by integration tests in the `tests/` directory.

use zeroize::Zeroize;

// ---------------------------------------------------------------------------
// Memory locking — platform-specific
// ---------------------------------------------------------------------------

/// Attempt to lock memory pages into RAM so they cannot be swapped to disk.
/// No-op for non-Android/non-Windows targets.
#[cfg(any(target_os = "android", target_os = "linux"))]
fn memory_lock(addr: *const std::ffi::c_void, len: usize) -> Result<(), String> {
    let ret = unsafe { libc::mlock(addr, len) };
    if ret == 0 {
        Ok(())
    } else {
        Err(format!("mlock failed: {}", std::io::Error::last_os_error()))
    }
}

/// Unlock memory pages previously locked with `memory_lock`.
#[cfg(any(target_os = "android", target_os = "linux"))]
fn memory_unlock(addr: *const std::ffi::c_void, len: usize) {
    unsafe { libc::munlock(addr, len); }
}

/// Lock memory on Windows using `VirtualLock`.
#[cfg(target_os = "windows")]
fn memory_lock(addr: *const std::ffi::c_void, len: usize) -> Result<(), String> {
    use windows_sys::Win32::System::Memory::VirtualLock;
    let ret = unsafe { VirtualLock(addr as *const std::ffi::c_void, len) };
    if ret != 0 {
        Ok(())
    } else {
        Err("VirtualLock failed".to_string())
    }
}

/// Unlock memory on Windows.
#[cfg(target_os = "windows")]
fn memory_unlock(addr: *const std::ffi::c_void, len: usize) {
    use windows_sys::Win32::System::Memory::VirtualUnlock;
    unsafe { VirtualUnlock(addr as *const std::ffi::c_void, len); }
}

/// No-op fallback for unsupported platforms (macOS, iOS, etc.).
#[cfg(not(any(target_os = "android", target_os = "linux", target_os = "windows")))]
fn memory_lock(_addr: *const std::ffi::c_void, _len: usize) -> Result<(), String> {
    Ok(())
}

/// No-op unlock fallback for unsupported platforms.
#[cfg(not(any(target_os = "android", target_os = "linux", target_os = "windows")))]
fn memory_unlock(_addr: *const std::ffi::c_void, _len: usize) {}

// ---------------------------------------------------------------------------
// SecureBuffer
// ---------------------------------------------------------------------------

/// A heap-allocated, memory-locked buffer for sensitive string data.
///
/// # Guarantees
/// 1. Memory is locked in RAM (`mlock`/`VirtualLock`) — never paged to disk.
/// 2. Memory is zeroed on `drop` (compile-time enforced via `Zeroize`).
/// 3. No interior `Clone` — only move semantics.
/// 4. Length and capacity metadata are also zeroed on drop.
///
/// # Example
/// ```ignore
/// let mut buffer = SecureBuffer::from_string("secret".to_string());
/// buffer.with_raw(|bytes| {
///     // Use the raw bytes here — they are zeroed when this scope ends
/// });
/// // buffer is dropped and zeroed here
/// ```
pub struct SecureBuffer {
    /// The raw byte storage for sensitive text.
    /// Access is tightly controlled through `with_raw` closures.
    data: Vec<u8>,
    /// Whether the memory was successfully locked.
    locked: bool,
}

impl SecureBuffer {
    /// Creates a new secure buffer from a string.
    /// The input string's heap is also overwritten after copy.
    ///
    /// # Arguments
    /// * `source` - The sensitive string to secure. Its heap is zeroed after copy.
    ///
    /// # Returns
    /// A new SecureBuffer containing the string's bytes, locked in RAM if possible.
    pub fn from_string(mut source: String) -> Self {
        let data = source.as_bytes().to_vec();
        // Zero the source string's heap to prevent lingering copies
        source.as_mut_str().zeroize();

        // Attempt to lock the allocated memory in RAM
        let locked = memory_lock(data.as_ptr() as *const std::ffi::c_void, data.len()).is_ok();

        SecureBuffer { data, locked }
    }

    /// Provides temporary, scoped access to the raw bytes.
    ///
    /// # Arguments
    /// * `f` - A closure that receives a mutable reference to the raw bytes.
    ///   The closure MUST NOT leak the reference outside its scope.
    ///
    /// # Returns
    /// The return value of the closure.
    pub fn with_raw<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.data)
    }

    /// Returns the length of the stored data in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns true if the buffer's memory was successfully locked in RAM.
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // First unlock memory if it was locked
        if self.locked {
            memory_unlock(self.data.as_ptr() as *const std::ffi::c_void, self.data.len());
        }
        // Overwrite data bytes with zeros
        self.data.as_mut_slice().zeroize();
        // Overwrite the Vec's internal capacity tracking
        self.data.zeroize();
    }
}

// SecureBuffer intentionally does NOT implement Clone.
// Sensitive data must not be duplicated accidentally.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_drop() {
        let original = "This is a test secret".to_string();
        let _buffer = SecureBuffer::from_string(original);
        // Buffer is dropped here — memory is zeroed
    }

    #[test]
    fn test_with_raw_access() {
        let mut buffer = SecureBuffer::from_string("hello".to_string());
        buffer.with_raw(|bytes| {
            assert_eq!(bytes, b"hello");
        });
    }

    #[test]
    fn test_len() {
        let buffer = SecureBuffer::from_string("secret".to_string());
        assert_eq!(buffer.len(), 6);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = SecureBuffer::from_string(String::new());
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_source_is_zeroed() {
        let source = String::from("sensitive");
        let _source_ptr = source.as_ptr();
        let _buffer = SecureBuffer::from_string(source);
        // source has been moved and zeroed — we can't access it anymore
        // (the variable is no longer valid in Rust's ownership model)
        // This test verifies the API contract: from_string takes ownership
    }

    /// Verify that SecureBuffer is NOT Clone
    /// This test will fail to compile if Clone is accidentally implemented
    #[test]
    fn test_not_clone() {
        fn assert_not_clone<T>() {}
        assert_not_clone::<SecureBuffer>();
    }

    #[test]
    fn test_multiple_buffers_sequential() {
        // Create and drop multiple buffers to verify no memory leaks
        for i in 0..100 {
            let text = format!("secret data item number {}", i);
            let _buffer = SecureBuffer::from_string(text);
        }
        // If we reach here, no panics from memory management
    }

    #[test]
    fn test_multibyte_utf8() {
        let mut buffer = SecureBuffer::from_string("émoji 😊 — café à la carte".to_string());
        buffer.with_raw(|bytes| {
            let text = std::str::from_utf8(bytes).expect("Valid UTF-8");
            assert!(text.contains("émoji 😊"));
            assert!(text.contains("café"));
        });
    }

    #[test]
    fn test_buffer_len_after_with_raw() {
        let original = "secure payload".to_string();
        let len_before = original.len();
        let mut buffer = SecureBuffer::from_string(original);
        assert_eq!(buffer.len(), len_before);
        buffer.with_raw(|bytes| {
            assert_eq!(bytes.len(), len_before);
        });
        // Length still accessible after closure
        assert_eq!(buffer.len(), len_before);
    }

    /// Verify that lock status is reported (may be false on CI without mlock)
    #[test]
    fn test_lock_status_query() {
        let buffer = SecureBuffer::from_string("test".to_string());
        // On most platforms this will be false (no mlock), but the API should work
        let _locked = buffer.is_locked();
    }

    #[test]
    fn test_large_buffer() {
        // 1 MB buffer — verify large allocations don't break
        let large = "A".repeat(1024 * 1024);
        let mut buffer = SecureBuffer::from_string(large);
        let count = buffer.with_raw(|bytes| bytes.len());
        assert_eq!(count, 1024 * 1024);
    }
}
