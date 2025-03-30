use chrono::Local;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{env, fs, io::Write, process::Command, sync::mpsc, thread, time::Duration};

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
const LOG_FILE: &str = "cmdy.log";

/// Load the configuration from the JSON file
fn load_config() -> Config {
    let config_data = fs::read_to_string(CONFIG_FILE)
        .unwrap_or_else(|_| "{\"directories\": [], \"command_sets\": []}".to_string());
    serde_json::from_str(&config_data).expect("Failed to parse config file")
}

/// Save the current configuration to the JSON file
fn save_config(config: &Config) {
    let config_data = serde_json::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(CONFIG_FILE, config_data).expect("Failed to save config file");
}

/// Execute the selected command set in the specified directory

fn execute_commands(commands: &[String], directory: &String, set: &CommandSet) {
    println!("\n🚀 Executing command set: {}", set.name);
    let pb = ProgressBar::new(commands.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({elapsed_precise})")
        .unwrap()
        .progress_chars("#>-"));

    for (_, cmd) in commands.iter().enumerate() {
        println!("🔹 Running: {}", cmd);
        let mut command = Command::new("sh");
        command.arg("-c").arg(cmd).current_dir(directory);

        if cmd == "npm run dev" {
            let mut child = command.spawn().expect("Failed to start process");
            let (tx, rx) = mpsc::channel();
            let spinner_thread = thread::spawn(move || {
                let spinner = ["|", "/", "-", "\\"];
                let mut i = 0;
                while rx.try_recv().is_err() {
                    print!("\rSpinning Deployment Server... {}", spinner[i % 4]);
                    std::io::stdout().flush().unwrap();
                    i += 1;
                    thread::sleep(Duration::from_millis(200));
                }
                println!("\rComplete!            ");
            });
            thread::sleep(Duration::from_secs(10));
            child.kill().expect("Failed to stop npm run dev");
            tx.send(()).unwrap();
            spinner_thread.join().unwrap();
            println!("npm run dev stopped after pulling the vault.");
        } else {
            let status = command.status().expect("Failed to execute command");
            if !status.success() {
                eprintln!("❌ Command failed: {}", cmd);
                return;
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("✅ All commands executed successfully!");
    log_execution(set);
}

/// Log executed command sets
fn log_execution(command_set: &CommandSet) {
    let log_entry = format!(
        "{} - Executed: {}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        command_set.name
    );
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .expect("Failed to open log file")
        .write_all(log_entry.as_bytes())
        .expect("Failed to write log");
}

/// List available command sets
fn list_commands() {
    let config = load_config();
    println!("\n📌 Stored Command Sets:");
    for (i, set) in config.command_sets.iter().enumerate() {
        println!("{}. {} - Commands: {:?}", i + 1, set.name, set.commands);
    }
}

/// View execution logs
fn view_logs() {
    let logs = fs::read_to_string(LOG_FILE).unwrap_or_else(|_| "No logs found.".to_string());
    println!(
        "\n📜 Execution Logs:\n{}
",
        logs
    );
}

/// Delete a specific command set
fn delete_command(command_name: &str) {
    let mut config = load_config();
    config.command_sets.retain(|set| set.name != command_name);
    save_config(&config);
    println!("🗑️ Deleted command set: {}", command_name);
}

/// Run the CLI workflow
fn run() {
    let mut config = load_config();
    let selection = Select::new()
        .with_prompt("Select a directory")
        .item("Current Directory")
        .items(&config.directories)
        .item("Enter New Directory")
        .interact()
        .unwrap();

    let directory = if selection == 0 {
        env::current_dir().unwrap().to_string_lossy().to_string()
    } else if selection == config.directories.len() + 1 {
        let new_dir: String = Input::new()
            .with_prompt("Enter directory path")
            .interact_text()
            .unwrap();
        config.directories.push(new_dir.clone());
        save_config(&config);
        new_dir
    } else {
        config.directories[selection - 1].clone()
    };

    let command_set_names: Vec<String> = config
        .command_sets
        .iter()
        .map(|set| set.name.clone())
        .collect();
    let command_set_name = Select::new()
        .with_prompt("Select a command set")
        .items(&command_set_names)
        .item("Create new command set")
        .interact()
        .unwrap();

    let command_set_name = if command_set_name == command_set_names.len() {
        Input::new()
            .with_prompt("Enter new command set name")
            .interact_text()
            .unwrap()
    } else {
        command_set_names[command_set_name].clone()
    };

    if !config
        .command_sets
        .iter()
        .any(|set| set.name == command_set_name)
    {
        let commands_input: String = Input::new()
            .with_prompt("Enter commands (comma-separated)")
            .interact_text()
            .unwrap();
        let commands = commands_input
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        config.command_sets.push(CommandSet {
            name: command_set_name.clone(),
            commands,
        });
        save_config(&config);
    }

    if let Some(set) = config
        .command_sets
        .iter()
        .find(|set| set.name == command_set_name)
    {
        execute_commands(&set.commands, &directory, set);
    }
}

/// Display help menu
fn help() {
    println!("\n📌 cmdy CLI - Enhanced Command Manager");
    println!("------------------------------------");
    println!("cmdy run         - Run a command set");
    println!("cmdy list        - List all command sets");
    println!("cmdy logs        - View execution logs");
    println!("cmdy delete <name> - Delete a command set");
    println!("cmdy help        - Show this help message\n");
}

/// Entry point of the CLI application
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") => run(),
        Some("list") => list_commands(),
        Some("logs") => view_logs(),
        Some("delete") if args.len() > 2 => delete_command(&args[2]),
        Some("help") | Some("--help") => help(),
        _ => println!("Usage: cmdy <command>. Use 'cmdy help' for details."),
    }
}
