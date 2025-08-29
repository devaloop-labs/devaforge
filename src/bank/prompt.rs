use std::{thread, time::Duration};

use crate::{
    bank::scaffold::scaffold_bank,
    utils::{kebab_case::to_kebab_case, spinner::with_spinner},
};

pub async fn prompt_bank_addon(cwd: &str) -> Result<(), String> {
    println!();
    println!("⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯");
    println!("Devalang Bank Forge");
    println!("⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯");
    println!();
    let final_name = match inquire::Text::new("Enter the bank name:")
        .with_default("my-bank")
        .prompt() {
        Ok(name) => to_kebab_case(&name),
        Err(e) => return Err(format!("Failed to prompt for bank name: {}", e)),
    };

    let final_author = match inquire::Text::new("Enter the bank author:")
        .with_default("johndoe")
        .prompt() {
        Ok(author) => to_kebab_case(&author),
        Err(e) => return Err(format!("Failed to prompt for bank author: {}", e)),
    };

    let final_description = match inquire::Text::new("Enter the bank description:")
        .with_default("A description of my bank")
        .prompt() {
        Ok(description) => to_kebab_case(&description),
        Err(e) => return Err(format!("Failed to prompt for bank description: {}", e)),
    };

    let options = vec!["public", "private", "protected"];
    let final_access = match inquire::Select::new("Select the bank access level:", options)
        .with_help_message(
            "Select if the bank should be public (free), private (for you only), or protected (purchased by others).",
        )
        .prompt() {
        Ok(access) => to_kebab_case(access),
        Err(e) => return Err(format!("Failed to prompt for bank access level: {}", e)),
    };

    println!();
    println!("⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯");
    println!("Confirm Bank Details");
    println!("⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯⎯");
    println!();

    println!("Name: {}", final_name);
    println!("Author: {}", final_author);
    println!("Description: {}", final_description);
    println!("Access Level: {}", final_access);

    println!();

    let confirm_prompt = inquire::Confirm::new("Are these details correct ?")
        .with_default(true)
        .prompt();

    match confirm_prompt {
        Ok(true) => {
            let spinner = with_spinner("Generating bank...", || {
                thread::sleep(Duration::from_millis(800));
            });

            let res = scaffold_bank(
                cwd,
                final_name,
                final_author,
                final_description,
                final_access,
            )
            .await;
            spinner.finish_and_clear();
            res
        }
        _ => {
            println!("Aborting bank scaffolding.");
            Err("aborted by user".into())
        }
    }
}
