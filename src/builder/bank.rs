use crate::utils::fs as ufs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BankSection {
    name: String,
    author: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    access: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TriggerEntry {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BankToml {
    bank: BankSection,
    #[serde(default)]
    triggers: Vec<TriggerEntry>,
}

/// Builds a bank located at the given path.
///
/// ### Parameters
/// - `path`: The path of the bank
/// - `cwd`: The current working directory
///
pub fn build_bank(path: &str, cwd: &str) -> Result<(), String> {
    let bank_dir = resolve_bank_dir(cwd, path)?;

    let bank_toml_path = bank_dir.join("bank.toml");
    if !bank_toml_path.exists() {
        return Err(format!(
            "bank.toml not found in: {}",
            bank_dir.to_string_lossy()
        ));
    }

    let mut bank_doc: BankToml = {
        let txt = fs::read_to_string(&bank_toml_path)
            .map_err(|e| format!("Failed to read bank.toml: {}", e))?;
        toml::from_str(&txt).map_err(|e| format!("Invalid TOML: {}", e))?
    };

    let audio_dir = bank_dir.join("audio");
    if !audio_dir.is_dir() {
        return Err(format!(
            "Audio directory not found: {}",
            audio_dir.to_string_lossy()
        ));
    }

    let discovered = discover_triggers(&audio_dir)?;
    bank_doc.triggers = merge_triggers(bank_doc.triggers, discovered);

    write_triggers_after_bank(&bank_toml_path, &bank_doc.triggers)?;

    let author = bank_doc.bank.author.clone();
    let name = bank_doc.bank.name.clone();
    if author.trim().is_empty() || name.trim().is_empty() {
        return Err("Fields [bank].author and [bank].name are required in bank.toml".into());
    }

    let out_root = Path::new(cwd).join("output").join("bank");
    fs::create_dir_all(&out_root)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    let out_file = out_root.join(format!("{}.{}.devabank", author, name));

    create_bank_zip(
        &bank_dir,
        &bank_toml_path,
        &audio_dir,
        &out_file,
        &author,
        &name,
        bank_doc.bank.description.clone(),
    )?;
    println!("✅ Bank built: {}", out_file.to_string_lossy());

    Ok(())
}

/// Builds all banks in the generated directory.
///
/// ### Parameters
/// - `cwd`: The current working directory
///
pub fn build_all_banks(cwd: &str) -> Result<(), String> {
    let banks_root = Path::new(cwd).join("generated").join("banks");
    if !banks_root.exists() {
        return Err(format!(
            "Banks directory not found: {}",
            banks_root.to_string_lossy()
        ));
    }

    let mut bank_dirs: Vec<PathBuf> = Vec::new();
    let rd = fs::read_dir(&banks_root)
        .map_err(|e| format!("Failed to list {}: {}", banks_root.to_string_lossy(), e))?;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() && p.join("bank.toml").exists() {
            bank_dirs.push(p);
        }
    }

    if bank_dirs.is_empty() {
        return Err("No banks to build (generated/banks is empty)".into());
    }

    bank_dirs.sort();

    let mut errors: Vec<String> = Vec::new();
    let total = bank_dirs.len();
    for p in bank_dirs {
        let p_str = p.to_string_lossy().to_string();
        match build_bank(&p_str, cwd) {
            Ok(_) => {}
            Err(e) => errors.push(format!("{} -> {}", p_str, e)),
        }
    }

    if errors.is_empty() {
        println!("✅ Build complete: {} bank(s) built", total);
        Ok(())
    } else {
        let joined = errors.join("\n - ");
        Err(format!(
            "Some banks failed ({}/{}):\n - {}",
            errors.len(),
            total,
            joined
        ))
    }
}

/// Resolves the bank directory from the given input path or alias.
///
/// ### Parameters
/// - `cwd`: The current working directory
/// - `input`: The input path or alias
///
fn resolve_bank_dir(cwd: &str, input: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(cwd).join(input);
    if candidate.is_file()
        && candidate
            .file_name()
            .map(|f| f == "bank.toml")
            .unwrap_or(false)
    {
        return Ok(candidate.parent().unwrap().to_path_buf());
    }
    if candidate.is_dir() && candidate.join("bank.toml").exists() {
        return Ok(candidate);
    }

    if let Some(rest) = input.strip_prefix("bank.") {
        let banks_root = Path::new(cwd).join("generated").join("banks");
        let by_exact = banks_root.join(rest);
        if by_exact.join("bank.toml").exists() {
            return Ok(by_exact);
        }
        if !rest.contains('.') {
            if let Ok(read_dir) = fs::read_dir(&banks_root) {
                let mut matches: Vec<PathBuf> = Vec::new();
                for e in read_dir.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            if name.ends_with(&format!(".{rest}")) && p.join("bank.toml").exists() {
                                matches.push(p.clone());
                            }
                        }
                    }
                }
                return match matches.len() {
                    1 => Ok(matches.remove(0)),
                    0 => Err(format!(
                        "No bank matched alias bank.{} under {}",
                        rest,
                        banks_root.to_string_lossy()
                    )),
                    _ => Err(format!(
                        "Multiple banks matched bank.{}; use 'bank.<author>.<name>'",
                        rest
                    )),
                };
            }
        }
        return Err(format!(
            "Alias not found: {}; expected under {}",
            input,
            banks_root.to_string_lossy()
        ));
    }

    Err(format!(
        "Invalid path: {} (no bank.toml found)",
        candidate.to_string_lossy()
    ))
}

/// Discover audio triggers in the given directory.
///
/// ### Parameters
/// - `audio_dir`: The directory to search for audio files
///
fn discover_triggers(audio_dir: &Path) -> Result<Vec<TriggerEntry>, String> {
    let mut out: Vec<TriggerEntry> = Vec::new();
    let allowed = ["wav", "mp3", "ogg", "aif", "aiff", "flac"];
    let files = ufs::walk_files(audio_dir)?;
    for p in files {
        let ext_ok = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| allowed.iter().any(|a| a.eq_ignore_ascii_case(e)))
            .unwrap_or(false);
        if !ext_ok {
            continue;
        }
        let rel = ufs::path_relative_to(&p, audio_dir).unwrap_or_else(|| {
            p.file_name()
                .map(PathBuf::from)
                .unwrap_or_else(PathBuf::new)
        });
        let rel_str = format!("./{}", ufs::to_unix_string(&rel));
        let name = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        out.push(TriggerEntry {
            name,
            path: rel_str,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Creates a ZIP archive of the bank directory.
///
/// ### Parameters
/// - `bank_dir`: The path to the bank directory.
/// - `bank_toml_path`: The path to the bank.toml file.
/// - `audio_dir`: The path to the audio directory.
/// - `out_file`: The output ZIP file path.
/// - `author`: The author of the bank.
/// - `name`: The name of the bank.
/// - `description`: An optional description of the bank.
///
fn create_bank_zip(
    bank_dir: &Path,
    bank_toml_path: &Path,
    audio_dir: &Path,
    out_file: &Path,
    author: &str,
    name: &str,
    description: Option<String>,
) -> Result<(), String> {
    let file =
        fs::File::create(out_file).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("bank.toml", options)
        .map_err(|e| format!("Failed to zip bank.toml: {}", e))?;
    let mut toml_bytes = Vec::new();
    fs::File::open(bank_toml_path)
        .and_then(|mut f| f.read_to_end(&mut toml_bytes))
        .map_err(|e| format!("Failed to read bank.toml: {}", e))?;
    zip.write_all(&toml_bytes)
        .map_err(|e| format!("Failed to write bank.toml to zip: {}", e))?;

    // Add README.md (from bank dir if present, else default)
    zip.start_file("README.md", options)
        .map_err(|e| format!("Failed to add README.md: {}", e))?;
    let readme_path = bank_dir.join("README.md");
    if readme_path.exists() {
        let mut buf = Vec::new();
        fs::File::open(&readme_path)
            .and_then(|mut f| f.read_to_end(&mut buf))
            .map_err(|e| format!("Failed to read README.md: {}", e))?;
        zip.write_all(&buf)
            .map_err(|e| format!("Failed to write README.md: {}", e))?;
    } else {
        let readme = default_readme_bank(author, name, description.as_deref());
        zip.write_all(readme.as_bytes())
            .map_err(|e| format!("Failed to write README.md: {}", e))?;
    }

    // Add LICENSE (from bank dir if present, else default MIT)
    zip.start_file("LICENSE", options)
        .map_err(|e| format!("Failed to add LICENSE: {}", e))?;
    let license_path = bank_dir.join("LICENSE");
    if license_path.exists() {
        let mut buf = Vec::new();
        fs::File::open(&license_path)
            .and_then(|mut f| f.read_to_end(&mut buf))
            .map_err(|e| format!("Failed to read LICENSE: {}", e))?;
        zip.write_all(&buf)
            .map_err(|e| format!("Failed to write LICENSE: {}", e))?;
    } else {
        let license = default_mit_license(author);
        zip.write_all(license.as_bytes())
            .map_err(|e| format!("Failed to write LICENSE: {}", e))?;
    }

    zip.add_directory("audio/", options)
        .map_err(|e| format!("Failed to add audio directory to zip: {}", e))?;

    let files = ufs::walk_files(audio_dir)?;
    for p in files {
        if !p.is_file() {
            continue;
        }
        let rel_os = ufs::path_relative_to(&p, audio_dir).unwrap_or_else(|| {
            p.file_name()
                .map(PathBuf::from)
                .unwrap_or_else(PathBuf::new)
        });
        let rel = ufs::to_unix_string(&rel_os);
        let mut data = Vec::new();
        fs::File::open(&p)
            .and_then(|mut f| f.read_to_end(&mut data))
            .map_err(|e| format!("Failed to read audio file: {}", e))?;
        let zip_path = format!("audio/{}", rel);
        zip.start_file(zip_path.clone(), options)
            .map_err(|e| format!("Failed to add {}: {}", zip_path, e))?;
        zip.write_all(&data)
            .map_err(|e| format!("Failed to write {}: {}", zip_path, e))?;
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?;
    let _ = fs::metadata(out_file).map_err(|e| format!("Failed to stat zip: {}", e))?;
    let _ = bank_dir;
    Ok(())
}

/// Gets the default README.md for a bank.
///
/// ### Parameters
/// - `author`: The author of the bank.
/// - `name`: The name of the bank.
/// - `description`: The description of the bank.
///
fn default_readme_bank(author: &str, name: &str, description: Option<&str>) -> String {
    let desc = description.unwrap_or("Sample bank for Devalang.");
    format!(
        "# {}.{} Bank\n\n{}\n\nContents:\n- bank.toml\n- audio/ (assets)\n- LICENSE\n\nBuilt with devaforge.\n",
        author, name, desc
    )
}

/// Gets the default LICENSE for a bank.
///
/// ### Parameters
/// - `author`: The author of the bank.
///
fn default_mit_license(author: &str) -> String {
    format!(
        "MIT License\n\nCopyright (c) {}\n\nPermission is hereby granted, free of charge, to any person obtaining a copy\n of this software and associated documentation files (the \"Software\"), to deal\n in the Software without restriction, including without limitation the rights\n to use, copy, modify, merge, publish, distribute, sublicense, and/or sell\n copies of the Software, and to permit persons to whom the Software is\n furnished to do so, subject to the following conditions:\n\nThe above copyright notice and this permission notice shall be included in all\n copies or substantial portions of the Software.\n\nTHE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\n IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE\n AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER\n LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,\n OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE\n SOFTWARE.\n",
        author
    )
}

/// Writes bank's triggers after the `[bank]` section.
///
/// ### Parameters
/// - `bank_toml_path`: The path to the bank.toml file.
/// - `triggers`: The triggers to add to the bank.toml file.
///
fn write_triggers_after_bank(
    bank_toml_path: &Path,
    triggers: &[TriggerEntry],
) -> Result<(), String> {
    let original = fs::read_to_string(bank_toml_path)
        .map_err(|e| format!("Failed to read bank.toml: {}", e))?;

    let mut cleaned: Vec<String> = Vec::new();
    let mut skipping_triggers = false;
    for line in original.lines() {
        let trimmed = line.trim();
        if !skipping_triggers {
            if trimmed == "[[triggers]]" {
                skipping_triggers = true;
                continue;
            }
            cleaned.push(line.to_string());
        } else {
            if trimmed.starts_with('[') && trimmed != "[[triggers]]" {
                skipping_triggers = false;
                cleaned.push(line.to_string());
            } else {
                continue;
            }
        }
    }

    let mut insert_idx = cleaned.len();
    let mut in_bank = false;
    for (i, line) in cleaned.iter().enumerate() {
        let t = line.trim();
        if t == "[bank]" {
            in_bank = true;
            insert_idx = i + 1;
            continue;
        }
        if in_bank && t.starts_with('[') && t != "[bank]" {
            insert_idx = i;
            break;
        }
        if in_bank && !t.is_empty() {
            insert_idx = i + 1;
        }
    }

    let head = cleaned[..insert_idx].to_vec();
    let mut tail = cleaned[insert_idx..].to_vec();
    while !tail.is_empty() && tail[0].trim().is_empty() {
        tail.remove(0);
    }

    let mut trig_lines: Vec<String> = Vec::new();
    if !triggers.is_empty() {
        for (i, t) in triggers.iter().enumerate() {
            trig_lines.push("[[triggers]]".to_string());
            trig_lines.push(format!("name = \"{}\"", t.name));
            trig_lines.push(format!("path = \"{}\"", t.path));
            if i + 1 < triggers.len() {
                trig_lines.push(String::new());
            }
        }
    }

    let mut result_lines: Vec<String> = Vec::new();
    result_lines.extend(head);
    if !trig_lines.is_empty() {
        result_lines.push(String::new());
        result_lines.extend(trig_lines);
        if !tail.is_empty() {
            result_lines.push(String::new());
        }
    }
    result_lines.extend(tail);

    let mut result = result_lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    fs::write(bank_toml_path, result).map_err(|e| format!("Failed to write bank.toml: {}", e))?;
    Ok(())
}

/// Merges the existing and discovered triggers.
///
/// ### Parameters
/// - `existing`: The existing triggers.
/// - `discovered`: The discovered triggers.
///
fn merge_triggers(existing: Vec<TriggerEntry>, discovered: Vec<TriggerEntry>) -> Vec<TriggerEntry> {
    use std::collections::{HashMap, HashSet};
    let mut by_path: HashMap<String, String> = HashMap::new();
    for t in existing {
        by_path.insert(t.path.clone(), t.name.clone());
    }

    let mut used_names: HashSet<String> = by_path.values().cloned().collect();
    let mut final_triggers: Vec<TriggerEntry> = Vec::new();
    for d in discovered {
        let path = d.path.clone();
        if let Some(existing_name) = by_path.get(&path) {
            final_triggers.push(TriggerEntry {
                name: existing_name.clone(),
                path,
            });
        } else {
            let base = d.name;
            let unique = disambiguate_name(&base, &path, &mut used_names);
            final_triggers.push(TriggerEntry { name: unique, path });
        }
    }
    final_triggers.sort_by(|a, b| a.path.cmp(&b.path));
    final_triggers
}

/// Disambiguates a name to ensure uniqueness within the used set.
///
/// ### Parameters
/// - `base`: The base name to disambiguate.
/// - `rel_path_with_dot`: The relative path with a dot prefix.
/// - `used`: The set of already used names.
///
fn disambiguate_name(
    base: &str,
    rel_path_with_dot: &str,
    used: &mut std::collections::HashSet<String>,
) -> String {
    if !base.is_empty() && !used.contains(base) {
        used.insert(base.to_string());
        return base.to_string();
    }
    let rel = rel_path_with_dot.trim_start_matches("./");
    let mut parts: Vec<&str> = rel.split('/').collect();
    if parts.len() > 1 {
        parts.pop();
        let joined = format!("{}.{}", parts.join("."), base);
        if !used.contains(&joined) {
            used.insert(joined.clone());
            return joined;
        }
        let mut acc: Vec<&str> = Vec::new();
        for comp in parts.iter().rev() {
            acc.push(comp);
            let name = format!(
                "{}.{}",
                acc.iter().rev().cloned().collect::<Vec<&str>>().join("."),
                base
            );
            if !used.contains(&name) {
                used.insert(name.clone());
                return name;
            }
        }
    }
    let mut i = 2usize;
    loop {
        let cand = format!("{}_{}", base, i);
        if !used.contains(&cand) {
            used.insert(cand.clone());
            return cand;
        }
        i += 1;
    }
}
