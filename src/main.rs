use std::{fs, process::Command, env};
use serde::{Serialize, Deserialize};
use dialoguer::{Input, Confirm, Select};
use std::env::args;

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

fn list_commands() {
    let config = load_config();
    println!("Stored Command Sets:");
    for (i, set) in config.command_sets.iter().enumerate() {
        println!("{}. {} - Commands: {:?}", i + 1, set.name, set.commands);
    }
}

fn delete_command(command_name: &str) {
    let mut config = load_config();
    config.command_sets.retain(|set| set.name != command_name);
    save_config(&config);
    println!("Deleted command set: {}", command_name);
}

fn export_config() {
    let config = load_config();
    let file_path: String = Input::new()
        .with_prompt("Enter the export file name (e.g., backup.json)")
        .default("backup.json".to_string())
        .interact_text()
        .unwrap();

    let data = serde_json::to_string_pretty(&config).expect("Failed to serialize");
    fs::write(&file_path, data).expect("Failed to write file");
    println!("Configuration exported to {}", file_path);
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "run" => {
                run();
                return;
            }
            "list" => {
                list_commands();
                return;
            }
            "delete" if args.len() > 2 => {
                delete_command(&args[2]);
                return;
            }
            "export" => {
                export_config();
                return;
            }
            "--help" | "help" => {
                help();
                return;
            }
            _ => {
                println!("Unknown command. Use 'cmdy --help' for available commands.");
                return;
            }
        }
    }
    println!("Usage: cmdy <command>. Use 'cmdy --help' for more details.");
}


fn run() {
    let mut config = load_config();
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

fn help() {
    println!("cmdy CLI - Command Manager");
    println!("Usage:");
    println!("  cmdy run    - Run a saved command set");
    println!("  cmdy list   - List all saved command sets");
    println!("  cmdy delete <command_name> - Delete a saved command set");
    println!("  cmdy --help | help - Show this help message");
}
