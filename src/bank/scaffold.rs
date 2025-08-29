use std::path::Path;

pub async fn scaffold_bank(
    cwd: &str,
    name: String,
    author: String,
    description: String,
    access: String,
) -> Result<(), String> {
    let banks_root = Path::new(cwd).join("generated").join("banks");
    let bank_name = format!("{}.{}", author, name);

    let bank_path = banks_root.join(bank_name);
    if bank_path.exists() {
        eprintln!("bank already exists, aborting");
        return Err("bank already exists, aborting".into());
    }

    if let Err(e) = std::fs::create_dir_all(&bank_path) {
        eprintln!("Error creating bank directory: {}", e);
        return Err(format!("Failed to create bank directory: {}", e));
    }

    if let Err(e) = create_bank_toml(&bank_path, &name, &author, &description, &access).await {
        eprintln!("Error creating bank toml: {}", e);
        return Err(format!("Failed to create bank toml: {}", e));
    }

    if let Err(e) = create_bank_audio_dir(&bank_path).await {
        eprintln!("Error creating bank audio directory: {}", e);
        return Err(format!("Failed to create bank audio directory: {}", e));
    }

    Ok(())
}

pub async fn create_bank_toml(
    bank_path: &Path,
    name: &str,
    author: &str,
    description: &str,
    access: &str,
) -> Result<(), String> {
    let version = "0.0.1";
    let bank_toml_content = format!(
        "[bank]\nname = \"{name}\"\nauthor = \"{author}\"\ndescription = \"{description}\"\nversion = \"{version}\"\naccess = \"{access}\"\n",
        name = name,
        author = author,
        description = description,
        version = version,
        access = access
    );

    if let Err(e) = std::fs::write(bank_path.join("bank.toml"), bank_toml_content) {
        eprintln!("Error creating bank.toml file: {}", e);
        return Err(format!("Failed to create bank.toml file: {}", e));
    }

    Ok(())
}

pub async fn create_bank_audio_dir(bank_path: &Path) -> Result<(), String> {
    let audio_dir = bank_path.join("audio");
    if let Err(e) = std::fs::create_dir_all(&audio_dir) {
        eprintln!("Error creating bank audio directory: {}", e);
        return Err(format!("Failed to create bank audio directory: {}", e));
    }

    Ok(())
}
