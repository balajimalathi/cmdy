# cmdy - Enhanced Command Manager CLI

cmdy is a powerful and flexible command execution manager that allows you to store, manage, and execute predefined command sets across different directories. It simplifies repetitive tasks and enhances automation for developers and power users.

## Features
- Store and execute predefined command sets
- Manage multiple directories for command execution
- Interactive selection of command sets and directories
- Progress bars and status indicators for better UX
- Execution logging for tracking command runs

## Installation
To install `cmdy`, clone the repository and build it using Cargo:
```sh
cargo build --release
```
Move the binary to a directory in your PATH:
```sh
mv target/release/cmdy /usr/local/bin/
```

## Usage
Run `cmdy` followed by a command:
```sh
cmdy <command>
```

### Available Commands
| Command          | Description |
|-----------------|-------------|
| `cmdy run`      | Run a stored command set |
| `cmdy list`     | List all stored command sets |
| `cmdy logs`     | View execution logs |
| `cmdy delete <name>` | Delete a command set |
| `cmdy help`     | Show help menu |

### Running a Command Set
```sh
cmdy run
```
This will prompt you to select a directory and a stored command set to execute.

### Listing Available Command Sets
```sh
cmdy list
```
Displays all stored command sets along with their associated commands.

### Viewing Execution Logs
```sh
cmdy logs
```
Shows a history of executed command sets with timestamps.

### Deleting a Command Set
```sh
cmdy delete <name>
```
Removes a specified command set from storage.

### Help Menu
```sh
cmdy help
```
Displays available commands and their descriptions.

## Configuration
cmdy stores its configurations in `config.json`.
- Directories: Stores directories used for executing commands.
- Command Sets: Stores predefined command sets.

Example `config.json`:
```json
{
  "directories": ["/path/to/project"],
  "command_sets": [
    {
      "name": "Build Project",
      "commands": ["cargo build", "cargo test"]
    }
  ]
}
```

## Logging
Execution logs are stored in `cmdy.log` with timestamps for tracking past executions.

## License
MIT License

## Contributions
Feel free to submit issues and pull requests to improve `cmdy`!

## Author
Developed by [Your Name]

