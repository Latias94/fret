use std::io::{IsTerminal as _, Write as _};
use std::path::PathBuf;

use crate::cli::workspace_root;

use super::fs::sanitize_package_name;
use super::{IconPack, NewTemplate, ScaffoldOptions, init_empty_at, init_hello_at, init_todo_at};

pub(super) fn new_wizard() -> Result<(), String> {
    if !std::io::stdin().is_terminal() {
        return Err(
            "interactive wizard requires a TTY (try: `fretboard new todo --name my-todo`)"
                .to_string(),
        );
    }

    let root = workspace_root()?;

    println!("Fretboard new (interactive)");
    println!();

    let template = prompt_choice(
        "Template",
        &[
            ("empty", NewTemplate::Empty),
            ("hello", NewTemplate::Hello),
            ("todo", NewTemplate::Todo),
        ],
        1,
    )?;

    let default_name = match template {
        NewTemplate::Empty => "my-app",
        NewTemplate::Hello => "hello-world",
        NewTemplate::Todo => "todo-app",
    };

    let name_raw = prompt_line("Package name", Some(default_name))?;
    let package_name = sanitize_package_name(&name_raw)?;

    let default_out = root.join("local").join(&package_name);
    let out_raw = prompt_line(
        "Output path (blank = default)",
        Some(default_out.to_string_lossy().as_ref()),
    )?;
    let out_dir = PathBuf::from(out_raw);

    let icon_pack = match template {
        NewTemplate::Empty => IconPack::None,
        _ => prompt_choice(
            "Icons",
            &[
                ("lucide", IconPack::Lucide),
                ("radix", IconPack::Radix),
                ("none", IconPack::None),
            ],
            0,
        )?,
    };

    let command_palette = match template {
        NewTemplate::Empty => false,
        _ => prompt_yes_no("Enable command palette? (--command-palette)", false)?,
    };

    let ui_assets = match template {
        NewTemplate::Todo => prompt_yes_no("Enable UI assets cache? (--ui-assets)", false)?,
        _ => false,
    };

    let opts = ScaffoldOptions {
        icon_pack,
        command_palette,
        ui_assets,
    };

    println!();
    println!("Summary:");
    println!("  template: {:?}", template);
    println!("  name:     {package_name}");
    println!("  path:     {}", out_dir.display());
    if !matches!(template, NewTemplate::Empty) {
        println!("  icons:    {}", opts.icon_pack.as_str());
        println!("  palette:  {}", opts.command_palette);
    }
    if matches!(template, NewTemplate::Todo) {
        println!("  ui-assets: {}", opts.ui_assets);
    }
    println!();

    if !prompt_yes_no("Proceed?", true)? {
        return Err("aborted".to_string());
    }

    match template {
        NewTemplate::Empty => init_empty_at(&out_dir, &package_name),
        NewTemplate::Hello => init_hello_at(&root, &out_dir, &package_name, opts),
        NewTemplate::Todo => init_todo_at(&root, &out_dir, &package_name, opts),
    }
}

fn prompt_line(prompt: &str, default: Option<&str>) -> Result<String, String> {
    let mut stdout = std::io::stdout();
    match default {
        Some(default) => {
            write!(&mut stdout, "{prompt} [{default}]: ").map_err(|e| e.to_string())?;
        }
        None => {
            write!(&mut stdout, "{prompt}: ").map_err(|e| e.to_string())?;
        }
    }
    stdout.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    let line = line.trim().to_string();
    if line.is_empty() {
        Ok(default.unwrap_or_default().to_string())
    } else {
        Ok(line)
    }
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool, String> {
    let hint = if default { "Y/n" } else { "y/N" };
    loop {
        let v = prompt_line(prompt, Some(hint))?;
        let v = v.trim().to_ascii_lowercase();
        match v.as_str() {
            "" => return Ok(default),
            "y" | "yes" | "true" | "1" => return Ok(true),
            "n" | "no" | "false" | "0" => return Ok(false),
            _ => {
                println!("Please enter y/n.");
            }
        }
    }
}

fn prompt_choice<T: Copy>(
    prompt: &str,
    items: &[(&str, T)],
    default_index: usize,
) -> Result<T, String> {
    if items.is_empty() {
        return Err("prompt_choice requires at least one item".to_string());
    }
    let default_index = default_index.min(items.len().saturating_sub(1));

    println!("{prompt}:");
    for (i, (label, _)) in items.iter().enumerate() {
        if i == default_index {
            println!("  {}) {} (default)", i + 1, label);
        } else {
            println!("  {}) {}", i + 1, label);
        }
    }

    loop {
        let raw = prompt_line("Select", Some(&(default_index + 1).to_string()))?;
        let raw = raw.trim();
        if raw.is_empty() {
            return Ok(items[default_index].1);
        }
        let Ok(n) = raw.parse::<usize>() else {
            println!("Please enter a number.");
            continue;
        };
        if (1..=items.len()).contains(&n) {
            return Ok(items[n - 1].1);
        }
        println!("Please enter a number between 1 and {}.", items.len());
    }
}
