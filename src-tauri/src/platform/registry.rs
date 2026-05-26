// platform/registry.rs — adaptador winreg para HKLM/HKCU/HKU/HKCR.

use crate::core::{AppError, AppResult};

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use serde_json::Value;
    use winreg::enums::*;
    use winreg::types::FromRegValue;
    use winreg::RegKey;

    const ALLOWED_HIVES: &[&str] = &["HKLM", "HKCU", "HKU", "HKCR"];

    fn hkey_for(hive: &str) -> AppResult<RegKey> {
        if !ALLOWED_HIVES.contains(&hive) {
            return Err(AppError::Permission(format!(
                "hive no permitido: {}",
                hive
            )));
        }
        Ok(match hive {
            "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
            "HKU" => RegKey::predef(HKEY_USERS),
            "HKCR" => RegKey::predef(HKEY_CLASSES_ROOT),
            _ => unreachable!(),
        })
    }

    pub fn read_value(hive: &str, key: &str, name: &str) -> AppResult<Option<Value>> {
        let root = hkey_for(hive)?;
        let subkey = match root.open_subkey(key) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(AppError::Registry(format!(
                    "open {}\\{}: {}",
                    hive, key, e
                )))
            }
        };
        let v = match subkey.get_raw_value(name) {
            Ok(rv) => rv,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(AppError::Registry(format!("get {}: {}", name, e)))
            }
        };
        Ok(Some(reg_value_to_json(&v)))
    }

    fn reg_value_to_json(v: &winreg::RegValue) -> Value {
        match v.vtype {
            REG_DWORD => {
                let n = u32::from_le_bytes(v.bytes[..4].try_into().unwrap_or([0; 4]));
                Value::from(n)
            }
            REG_QWORD => {
                let n = u64::from_le_bytes(v.bytes[..8].try_into().unwrap_or([0; 8]));
                Value::from(n)
            }
            REG_SZ | REG_EXPAND_SZ => {
                let s = String::from_reg_value(v).unwrap_or_default();
                Value::from(s)
            }
            REG_MULTI_SZ => {
                let s: Vec<String> =
                    winreg::types::FromRegValue::from_reg_value(v).unwrap_or_default();
                Value::from(s)
            }
            REG_BINARY => Value::from(v.bytes.clone()),
            _ => Value::Null,
        }
    }

    pub fn write_value(
        hive: &str,
        key: &str,
        name: &str,
        kind: &str,
        value: &Value,
        create_if_missing: bool,
    ) -> AppResult<()> {
        let root = hkey_for(hive)?;
        let subkey = if create_if_missing {
            root.create_subkey(key)
                .map_err(|e| {
                    AppError::Registry(format!("create {}\\{}: {}", hive, key, e))
                })?
                .0
        } else {
            root.open_subkey_with_flags(key, KEY_SET_VALUE)
                .map_err(|e| {
                    AppError::Registry(format!("open {}\\{}: {}", hive, key, e))
                })?
        };

        match kind {
            "dword" => {
                let n = value.as_u64().unwrap_or(0) as u32;
                subkey
                    .set_value(name, &n)
                    .map_err(|e| AppError::Registry(format!("set dword: {}", e)))
            }
            "qword" => {
                let n = value.as_u64().unwrap_or(0);
                subkey
                    .set_value(name, &n)
                    .map_err(|e| AppError::Registry(format!("set qword: {}", e)))
            }
            "string" => {
                let s = value.as_str().unwrap_or("").to_string();
                subkey
                    .set_value(name, &s)
                    .map_err(|e| AppError::Registry(format!("set string: {}", e)))
            }
            "expand-string" => {
                let s = value.as_str().unwrap_or("").to_string();
                let rv = winreg::RegValue {
                    bytes: encode_wide_string(&s),
                    vtype: REG_EXPAND_SZ,
                };
                subkey
                    .set_raw_value(name, &rv)
                    .map_err(|e| AppError::Registry(format!("set expand: {}", e)))
            }
            "multi-string" => {
                let arr = value.as_array().cloned().unwrap_or_default();
                let v: Vec<String> = arr
                    .into_iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect();
                subkey
                    .set_value(name, &v)
                    .map_err(|e| AppError::Registry(format!("set multi: {}", e)))
            }
            _ => Err(AppError::Registry(format!(
                "kind no soportado: {}",
                kind
            ))),
        }
    }

    fn encode_wide_string(s: &str) -> Vec<u8> {
        let mut buf: Vec<u16> = s.encode_utf16().collect();
        buf.push(0);
        buf.into_iter().flat_map(|c| c.to_le_bytes()).collect()
    }

    pub fn delete_value(hive: &str, key: &str, name: &str) -> AppResult<()> {
        let root = hkey_for(hive)?;
        let subkey = match root.open_subkey_with_flags(key, KEY_SET_VALUE) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                return Err(AppError::Registry(format!(
                    "open for delete: {}",
                    e
                )))
            }
        };
        match subkey.delete_value(name) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::Registry(format!("delete: {}", e))),
        }
    }

    pub fn write_value_or_delete(
        hive: &str,
        key: &str,
        name: &str,
        previous_value: Option<&Value>,
    ) -> AppResult<()> {
        match previous_value {
            Some(v) => {
                let kind = infer_kind(v);
                write_value(hive, key, name, kind, v, true)
            }
            None => delete_value(hive, key, name),
        }
    }

    fn infer_kind(v: &Value) -> &'static str {
        match v {
            Value::Number(n)
                if n.is_u64() && n.as_u64().unwrap() <= u32::MAX as u64 =>
            {
                "dword"
            }
            Value::Number(_) => "qword",
            Value::String(_) => "string",
            Value::Array(_) => "multi-string",
            _ => "string",
        }
    }
}

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
pub fn read_value(_hive: &str, _path: &str, _value: &str) -> AppResult<Option<serde_json::Value>> {
    Err(AppError::External(
        "Registro de Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn write_value(
    _hive: &str,
    _key: &str,
    _name: &str,
    _kind: &str,
    _value: &serde_json::Value,
    _create_if_missing: bool,
) -> AppResult<()> {
    Err(AppError::External(
        "Registro de Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn delete_value(_hive: &str, _path: &str, _value: &str) -> AppResult<()> {
    Err(AppError::External(
        "Registro de Windows no disponible en esta plataforma".to_string(),
    ))
}

#[cfg(not(windows))]
pub fn write_value_or_delete(
    _hive: &str,
    _key: &str,
    _name: &str,
    _previous_value: Option<&serde_json::Value>,
) -> AppResult<()> {
    Err(AppError::External(
        "Registro de Windows no disponible en esta plataforma".to_string(),
    ))
}
