use std::{fs, process::Command, env};
use serde::{Serialize, Deserialize};
use dialoguer::{Input, Confirm, Select};

// Structure to hold named command sets and working directory
#[derive(Serialize, Deserialize)]
struct Config {
    directories: Vec<String>,
    command_sets: Vec<CommandSet>,
}

#[derive(Serialize, Deserialize)]
struct CommandSet {
    name: String,
    commands: Vec<String>,
}

const CONFIG_FILE: &str = "config.json";

// Load configuration from the config file
fn load_config() -> Config {
    let config_data = fs::read_to_string(CONFIG_FILE).unwrap_or_else(|_| "{\"directories\": [], \"command_sets\": []}".to_string());
    serde_json::from_str(&config_data).expect("Failed to parse config file")
}

// Save configuration to the config file
fn save_config(config: &Config) {
    let config_data = serde_json::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(CONFIG_FILE, config_data).expect("Failed to save config file");
}

// Execute the list of commands sequentially
fn execute_commands(commands: &[String], directory: &String) {
    for cmd in commands {
        println!("Executing: {}", cmd);
        let mut command = Command::new("sh");
        command.arg("-c").arg(cmd).current_dir(directory);
        
        let status = command.status().expect("Failed to execute command");
        if !status.success() {
            eprintln!("Command failed: {}", cmd);
            return;
        }
    }
}

fn main() {
    let mut config = load_config();

    // Ask if user wants to run in the current directory
    let selection = Select::new()
        .with_prompt("Do you want to run in the current directory or select a saved one?")
        .item("Current Directory")
        .items(&config.directories)
        .item("Enter New Directory")
        .interact()
        .unwrap();

    let directory = if selection == 0 {
        env::current_dir().unwrap().to_string_lossy().to_string()
    } else if selection == config.directories.len() + 1 {
        let new_dir: String = Input::new()
            .with_prompt("Enter the directory path")
            .interact_text()
            .unwrap();
        config.directories.push(new_dir.clone());
        save_config(&config);
        new_dir
    } else {
        config.directories[selection - 1].clone()
    };

    // Select or create a command set
    let command_set_names: Vec<String> = config.command_sets.iter().map(|set| set.name.clone()).collect();
    let command_set_name = if command_set_names.is_empty() {
        Input::new()
            .with_prompt("Enter a new command set name")
            .interact_text()
            .unwrap()
    } else {
        let selection = Select::new()
            .with_prompt("Select an existing command set or create a new one")
            .items(&command_set_names)
            .item("Create new command set")
            .interact()
            .unwrap();
        
        if selection == command_set_names.len() {
            Input::new()
                .with_prompt("Enter a new command set name")
                .interact_text()
                .unwrap()
        } else {
            command_set_names[selection].clone()
        }
    };

    if !config.command_sets.iter().any(|set| set.name == command_set_name) {
        println!("Creating a new command set.");
        let commands_input: String = Input::new()
            .with_prompt("Enter the commands to execute (comma-separated)")
            .interact_text()
            .unwrap();
        let commands = commands_input.split(',').map(|s| s.trim().to_string()).collect();
        
        config.command_sets.push(CommandSet {
            name: command_set_name.clone(),
            commands,
        });
        save_config(&config);
    }

    if let Some(set) = config.command_sets.iter().find(|set| set.name == command_set_name) {
        execute_commands(&set.commands, &directory);
    }
}

// Help command to display available paths and saved command sets
fn help() {
    let config = load_config();
    println!("Stored Directories:");
    for (i, dir) in config.directories.iter().enumerate() {
        println!("{}. {}", i + 1, dir);
    }
    println!("\nStored Command Sets:");
    for (i, set) in config.command_sets.iter().enumerate() {
        println!("{}. {} - Commands: {:?}", i + 1, set.name, set.commands);
    }
}
