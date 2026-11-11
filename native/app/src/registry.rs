use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE, REG_SZ,
    REG_VALUE_TYPE, RegCloseKey, RegCreateKeyExW, RegDeleteKeyW, RegDeleteTreeW, RegOpenKeyExW,
    RegQueryInfoKeyW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::{HSTRING, PCWSTR};

pub struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

fn ok(status: WIN32_ERROR) -> bool {
    status == ERROR_SUCCESS
}

impl Key {
    pub fn create(path: &str) -> Option<Key> {
        let mut key = HKEY::default();
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                &HSTRING::from(path),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_READ | KEY_WRITE,
                None,
                &mut key,
                None,
            )
        };
        ok(status).then_some(Key(key))
    }

    pub fn open(path: &str) -> Option<Key> {
        let mut key = HKEY::default();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                &HSTRING::from(path),
                None,
                KEY_READ,
                &mut key,
            )
        };
        ok(status).then_some(Key(key))
    }

    pub fn set_dword(&self, name: &str, value: u32) -> bool {
        ok(unsafe {
            RegSetValueExW(
                self.0,
                &HSTRING::from(name),
                None,
                REG_DWORD,
                Some(&value.to_le_bytes()),
            )
        })
    }

    pub fn set_string(&self, name: &str, value: &str) -> bool {
        let bytes: Vec<u8> = value
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect();
        ok(unsafe { RegSetValueExW(self.0, &HSTRING::from(name), None, REG_SZ, Some(&bytes)) })
    }

    pub fn dword(&self, name: &str) -> Option<u32> {
        let mut kind = REG_VALUE_TYPE::default();
        let mut data = [0u8; 4];
        let mut size = data.len() as u32;
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                &HSTRING::from(name),
                None,
                Some(&mut kind),
                Some(data.as_mut_ptr()),
                Some(&mut size),
            )
        };
        (ok(status) && kind == REG_DWORD && size == 4).then(|| u32::from_le_bytes(data))
    }

    pub fn string(&self, name: &str) -> Option<String> {
        let wide_name = HSTRING::from(name);
        let mut kind = REG_VALUE_TYPE::default();
        let mut size = 0u32;
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                &wide_name,
                None,
                Some(&mut kind),
                None,
                Some(&mut size),
            )
        };
        if !ok(status) || kind != REG_SZ || size > 65_536 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        let status = unsafe {
            RegQueryValueExW(
                self.0,
                &wide_name,
                None,
                Some(&mut kind),
                Some(data.as_mut_ptr()),
                Some(&mut size),
            )
        };
        if !ok(status) || kind != REG_SZ {
            return None;
        }
        data.truncate(size as usize);
        let units: Vec<u16> = data
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        let end = units
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(units.len());
        Some(String::from_utf16_lossy(&units[..end]))
    }

    pub fn counts(&self) -> Option<(u32, u32)> {
        let mut subkeys = 0u32;
        let mut values = 0u32;
        let status = unsafe {
            RegQueryInfoKeyW(
                self.0,
                None,
                None,
                None,
                Some(&mut subkeys),
                None,
                None,
                Some(&mut values),
                None,
                None,
                None,
                None,
            )
        };
        ok(status).then_some((subkeys, values))
    }
}

pub fn exists(path: &str) -> bool {
    Key::open(path).is_some()
}

pub fn delete_tree(path: &str) -> bool {
    let status = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, &HSTRING::from(path)) };
    if status == ERROR_FILE_NOT_FOUND {
        return true;
    }
    if !ok(status) {
        return false;
    }
    let status = unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, &HSTRING::from(path)) };
    ok(status) || status == ERROR_FILE_NOT_FOUND
}

pub fn delete_if_empty(path: &str) -> bool {
    let empty = match Key::open(path) {
        Some(key) => key.counts() == Some((0, 0)),
        None => return true,
    };
    empty && ok(unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, &HSTRING::from(path)) })
}

pub fn parent(path: &str) -> Option<&str> {
    path.rfind('\\')
        .map(|index| &path[..index])
        .filter(|parent| !parent.is_empty())
}
