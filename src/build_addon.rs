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

pub fn build_addon(path: &str, _release: &bool, cwd: &str) -> Result<(), String> {
    // 1) Resolve the bank directory from input (relative path or alias bank.<id>)
    let bank_dir = resolve_bank_dir(cwd, path)?;

    // 2) Ensure bank.toml exists and load it
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

    // 3) Scan audio directory and compute triggers
    let audio_dir = bank_dir.join("audio");
    if !audio_dir.is_dir() {
        return Err(format!(
            "Audio directory not found: {}",
            audio_dir.to_string_lossy()
        ));
    }

    // Discover triggers from filesystem
    let discovered = discover_triggers(&audio_dir)?;
    // Merge with existing triggers: preserve custom names for same path, drop missing, add new.
    bank_doc.triggers = merge_triggers(bank_doc.triggers, discovered);

    // 4) Write triggers after bank
    write_triggers_after_bank(&bank_toml_path, &bank_doc.triggers)?;

    // 5) Build zip -> <root>/output/bank/<author>.<bankName>.devabank
    let author = bank_doc.bank.author.clone();
    let name = bank_doc.bank.name.clone();
    if author.trim().is_empty() || name.trim().is_empty() {
        return Err("Fields [bank].author and [bank].name are required in bank.toml".into());
    }

    let out_root = Path::new(cwd).join("output").join("bank");
    fs::create_dir_all(&out_root)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    let out_file = out_root.join(format!("{}.{}.devabank", author, name));

    create_bank_zip(&bank_dir, &bank_toml_path, &audio_dir, &out_file)?;

    println!("✅ Bank built: {}", out_file.to_string_lossy());

    Ok(())
}

pub fn build_all_addons(release: &bool, cwd: &str) -> Result<(), String> {
    // For now: build banks only
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
        match build_addon(&p_str, release, cwd) {
            Ok(_) => {
                // continue
            }
            Err(e) => {
                errors.push(format!("{} -> {}", p_str, e));
            }
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

fn resolve_bank_dir(cwd: &str, input: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(cwd).join(input);
    // If input points directly to a file bank.toml
    if candidate.is_file()
        && candidate
            .file_name()
            .map(|f| f == "bank.toml")
            .unwrap_or(false)
    {
        return Ok(candidate.parent().unwrap().to_path_buf());
    }
    // If input is a directory containing bank.toml
    if candidate.is_dir() && candidate.join("bank.toml").exists() {
        return Ok(candidate);
    }

    // Handle alias: bank.<id>
    if let Some(rest) = input.strip_prefix("bank.") {
        let banks_root = Path::new(cwd).join("generated").join("banks");
        let by_exact = banks_root.join(rest);
        if by_exact.join("bank.toml").exists() {
            return Ok(by_exact);
        }
        // If no dot provided, try to find a single match that ends with .<rest>
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

fn discover_triggers(audio_dir: &Path) -> Result<Vec<TriggerEntry>, String> {
    let mut out: Vec<TriggerEntry> = Vec::new();
    let allowed = ["wav", "mp3", "ogg", "aif", "aiff", "flac"];

    // Load existing triggers if present to preserve names when paths match
    // (Caller will merge with bank_doc, but this function focuses on discovery only.)

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
        // Compute path relative to audio_dir with forward slashes and leading ./
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

    // Keep a stable order (by path) for deterministic outputs
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

// moved common fs utilities to utils::fs

fn create_bank_zip(
    bank_dir: &Path,
    bank_toml_path: &Path,
    audio_dir: &Path,
    out_file: &Path,
) -> Result<(), String> {
    // Use zip crate to create the archive containing bank.toml and audio/*
    let file =
        fs::File::create(out_file).map_err(|e| format!("Failed to create output file: {}", e))?;

    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Add bank.toml at root of zip
    zip.start_file("bank.toml", options)
        .map_err(|e| format!("Failed to zip bank.toml: {}", e))?;
    let mut toml_bytes = Vec::new();
    fs::File::open(bank_toml_path)
        .and_then(|mut f| f.read_to_end(&mut toml_bytes))
        .map_err(|e| format!("Failed to read bank.toml: {}", e))?;
    zip.write_all(&toml_bytes)
        .map_err(|e| format!("Failed to write bank.toml to zip: {}", e))?;

    // Add audio directory entries recursively as audio/<relative_path>
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
    // Touch to ensure proper timestamp (on some platforms)
    let _ = fs::metadata(out_file).map_err(|e| format!("Failed to stat zip: {}", e))?;

    // sanity: bank_dir is unused beyond; keep param for future when including more files
    let _ = bank_dir;
    Ok(())
}

fn write_triggers_after_bank(
    bank_toml_path: &Path,
    triggers: &[TriggerEntry],
) -> Result<(), String> {
    let original = fs::read_to_string(bank_toml_path)
        .map_err(|e| format!("Failed to read bank.toml: {}", e))?;

    // 1) Remove existing [[triggers]] blocks
    let mut cleaned: Vec<String> = Vec::new();
    let mut skipping_triggers = false;
    for line in original.lines() {
        let trimmed = line.trim();
        if !skipping_triggers {
            if trimmed == "[[triggers]]" {
                skipping_triggers = true;
                continue; // skip header line
            }
            cleaned.push(line.to_string());
        } else {
            // currently skipping; stop skipping when we hit a non-triggers header
            if trimmed.starts_with('[') && trimmed != "[[triggers]]" {
                skipping_triggers = false;
                cleaned.push(line.to_string()); // keep this new section header
            } else {
                // still in triggers block -> skip
                continue;
            }
        }
    }

    // 2) Find insertion point: right after [bank] section block
    let mut insert_idx = cleaned.len();
    let mut in_bank = false;
    for (i, line) in cleaned.iter().enumerate() {
        let t = line.trim();
        if t == "[bank]" {
            in_bank = true;
            insert_idx = i + 1; // default right after header if no fields
            continue;
        }
        if in_bank && t.starts_with('[') && t != "[bank]" {
            // Next section header -> insert before this
            insert_idx = i;
            break;
        }
        if in_bank {
            insert_idx = i + 1; // keep advancing until end of bank block
        }
    }

    // 3) Recompose file with exactly one blank line between sections
    let (mut head, mut tail): (Vec<String>, Vec<String>) = {
        let (h, t) = cleaned.split_at(insert_idx);
        (h.to_vec(), t.to_vec())
    };

    // Trim trailing blank lines from head
    while head.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        head.pop();
    }

    // Trim leading blank lines from tail
    while tail.first().map(|l| l.trim().is_empty()).unwrap_or(false) {
        tail.remove(0);
    }

    // Build triggers blocks with a single blank line between each
    let mut trig_lines: Vec<String> = Vec::new();
    if !triggers.is_empty() {
        for (i, t) in triggers.iter().enumerate() {
            trig_lines.push("[[triggers]]".to_string());
            trig_lines.push(format!("name = \"{}\"", t.name));
            trig_lines.push(format!("path = \"{}\"", t.path));
            if i + 1 < triggers.len() {
                trig_lines.push(String::new()); // blank between triggers
            }
        }
    }

    // Assembler: head + 1 blank + triggers + 1 blank + tail
    let mut result_lines: Vec<String> = Vec::new();
    result_lines.extend(head);
    if !trig_lines.is_empty() {
        result_lines.push(String::new()); // exactly one blank between [bank] and triggers
        result_lines.extend(trig_lines);
        if !tail.is_empty() {
            result_lines.push(String::new()); // exactly one blank before next section
        }
    }
    result_lines.extend(tail);

    // Join with newlines and ensure single trailing newline
    let mut result = result_lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }

    fs::write(bank_toml_path, result).map_err(|e| format!("Failed to write bank.toml: {}", e))?;
    Ok(())
}

fn merge_triggers(existing: Vec<TriggerEntry>, discovered: Vec<TriggerEntry>) -> Vec<TriggerEntry> {
    use std::collections::{HashMap, HashSet};
    let mut by_path: HashMap<String, String> = HashMap::new(); // path -> name
    for t in existing {
        by_path.insert(t.path.clone(), t.name.clone());
    }

    let mut used_names: HashSet<String> = by_path.values().cloned().collect();
    let mut discovered_paths: HashSet<String> = HashSet::new();

    // Build final list, preserving names where possible
    let mut final_triggers: Vec<TriggerEntry> = Vec::new();
    for d in discovered {
        let path = d.path.clone();
        discovered_paths.insert(path.clone());
        if let Some(existing_name) = by_path.get(&path) {
            // Keep existing name
            final_triggers.push(TriggerEntry {
                name: existing_name.clone(),
                path: path.clone(),
            });
        } else {
            // Assign a unique name derived from filename and possibly folder
            let base = d.name;
            let unique = disambiguate_name(&base, &path, &mut used_names);
            final_triggers.push(TriggerEntry {
                name: unique,
                path: path.clone(),
            });
        }
    }

    // Sort deterministically by path
    final_triggers.sort_by(|a, b| a.path.cmp(&b.path));
    final_triggers
}

fn disambiguate_name(
    base: &str,
    rel_path_with_dot: &str,
    used: &mut std::collections::HashSet<String>,
) -> String {
    // Try base name first
    if !base.is_empty() && !used.contains(base) {
        used.insert(base.to_string());
        return base.to_string();
    }

    // Remove leading "./" then split on '/'
    let rel = rel_path_with_dot.trim_start_matches("./");
    let mut parts: Vec<&str> = rel.split('/').collect();
    // Last part is filename; take stem from base, others are directories
    if parts.len() > 1 {
        parts.pop(); // remove filename
        let joined = format!("{}.{}", parts.join("."), base);
        if !used.contains(&joined) {
            used.insert(joined.clone());
            return joined;
        }
        // Try from the deepest folder backward to progressively disambiguate
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

    // Fallback to numeric suffixes
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
