// platform/inventory/winget.rs — Lista de apps detectadas por winget.
//
// Solo se ejecuta si winget está disponible en PATH.
// Parsea la salida tabular de `winget list --source winget`.

pub struct WingetApp {
    pub id: String,
    pub display_name: String,
    pub version: Option<String>,
}

pub fn list_winget_apps() -> Vec<WingetApp> {
    let Ok(which) = std::process::Command::new("where").arg("winget").output() else {
        return Vec::new();
    };
    if !which.status.success() {
        return Vec::new();
    }

    let Ok(out) = std::process::Command::new("winget")
        .args(["list", "--source", "winget", "--accept-source-agreements"])
        .output()
    else {
        return Vec::new();
    };

    if !out.status.success() {
        return Vec::new();
    }

    parse_winget_output(&String::from_utf8_lossy(&out.stdout))
}

fn parse_winget_output(text: &str) -> Vec<WingetApp> {
    let mut apps = Vec::new();
    let mut lines = text.lines();

    // Find the header line (contains "Name", "Id", "Version")
    let mut header_line: Option<&str> = None;
    let mut separator_line: Option<&str> = None;

    for line in lines.by_ref() {
        let stripped = strip_ansi(line);
        if stripped.contains("Id") && stripped.contains("Name") && stripped.contains("Version") {
            header_line = Some(line);
        } else if header_line.is_some() && stripped.trim_start_matches('-').is_empty() {
            separator_line = Some(line);
            break;
        }
    }

    let (Some(header), Some(sep)) = (header_line, separator_line) else {
        return apps;
    };

    // Compute column widths from separator dashes
    let col_ranges = compute_column_ranges(&strip_ansi(sep));
    if col_ranges.len() < 3 {
        return apps;
    }

    let header_stripped = strip_ansi(header);
    // Find which column index is Name, Id, Version
    let name_col = find_col(&header_stripped, &col_ranges, "Name");
    let id_col = find_col(&header_stripped, &col_ranges, "Id");
    let ver_col = find_col(&header_stripped, &col_ranges, "Version");

    for line in lines {
        let stripped = strip_ansi(line);
        if stripped.trim().is_empty() {
            continue;
        }
        let chars: Vec<char> = stripped.chars().collect();
        let name = extract_col_chars(&chars, &col_ranges, name_col).trim().to_string();
        let id = extract_col_chars(&chars, &col_ranges, id_col).trim().to_string();
        let version = extract_col_chars(&chars, &col_ranges, ver_col).trim().to_string();

        if id.is_empty() {
            continue;
        }

        let display_name = if name.is_empty() { id.clone() } else { name.clone() };
        apps.push(WingetApp {
            id,
            display_name,
            version: if version.is_empty() { None } else { Some(version) },
        });
    }

    apps
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut iter = s.chars().peekable();
    while let Some(c) = iter.next() {
        if c == '\x1b' {
            if iter.peek() == Some(&'[') {
                iter.next();
                for nc in iter.by_ref() {
                    if nc.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn compute_column_ranges(sep: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let chars: Vec<char> = sep.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '-' {
            let start = i;
            while i < chars.len() && chars[i] == '-' {
                i += 1;
            }
            ranges.push((start, i));
            // skip space between columns
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    ranges
}

fn find_col(header: &str, ranges: &[(usize, usize)], name: &str) -> usize {
    let chars: Vec<char> = header.chars().collect();
    for (idx, &(start, end)) in ranges.iter().enumerate() {
        let slice: String = chars[start..end.min(chars.len())].iter().collect();
        if slice.trim().to_lowercase().contains(&name.to_lowercase()) {
            return idx;
        }
    }
    0
}

fn extract_col_chars(chars: &[char], ranges: &[(usize, usize)], col: usize) -> String {
    let Some(&(start, end)) = ranges.get(col) else {
        return String::new();
    };
    chars[start..end.min(chars.len())].iter().collect()
}
