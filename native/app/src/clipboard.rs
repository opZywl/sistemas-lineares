use windows::Win32::Foundation::{GlobalFree, HANDLE, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::System::Threading::Sleep;

const ATTEMPTS: u32 = 10;
const DELAY_MS: u32 = 30;

pub fn set_text(owner: HWND, text: &str) -> bool {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            unsafe { Sleep(DELAY_MS) };
        }
        if unsafe { OpenClipboard(Some(owner)) }.is_err() {
            continue;
        }
        let stored = unsafe { store(&wide) };
        unsafe {
            let _ = CloseClipboard();
        }
        if stored {
            return true;
        }
    }
    false
}

unsafe fn store(wide: &[u16]) -> bool {
    unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        let bytes = std::mem::size_of_val(wide);
        let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, bytes) else {
            return false;
        };
        let target = GlobalLock(memory).cast::<u16>();
        if target.is_null() {
            let _ = GlobalFree(Some(memory));
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
        let _ = GlobalUnlock(memory);
        if SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))).is_err() {
            let _ = GlobalFree(Some(memory));
            return false;
        }
        true
    }
}
