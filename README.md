# 🔎 Port Detective

**What's running on this port, and how do I safely kill it?**

Port Detective is a tiny, fast Rust CLI that replaces the `lsof`/`netstat`/`ps` incantation soup with one clear command.

## Installation

```bash
cargo install --path .
```

## Usage

### Inspect a port

```bash
portdetective 3000
```

```
🔎 Port 3000 (tcp) is in use

Process:    node
PID:        42193
User:       makafui
Command:    node server.js --port=3000
CWD:        /Users/makafui/projects/my-app
Parent:     zsh (PID 41200)
Started:    2025-11-18 14:32:10

Suggested kill:
  kill 42193
  # or force if needed:
  kill -9 42193
```

### Check if a port is free

```bash
portdetective 55555
```

```
✅ Port 55555 is free (no listening process found)
```

### List all listening ports

```bash
portdetective list
```

```
PORT    PROTO  PID      PROCESS      USER       COMMAND
3000    tcp    42193    node         makafui    node server.js --port=3000
5432    tcp    550      postgres     postgres   /usr/local/bin/postgres -D ...
8000    tcp    43011    python       makafui    uvicorn main:app --port 8000

📊 3 listening port(s) found
```

### Kill process on a port

```bash
portdetective kill 3000
```

Interactive confirmation:
```
🔎 Port 3000 (tcp) is in use by:
  node (PID 42193)
  Command: node server.js --port=3000
  CWD:     /Users/makafui/projects/my-app

Are you sure you want to kill PID 42193? [y/N]:
```

### Commands & Aliases

| Command | Aliases | Description |
|---------|---------|-------------|
| `portdetective <PORT>` | — | Inspect a port (shorthand) |
| `portdetective inspect <PORT>` | `i` | Inspect what's on a port |
| `portdetective list` | `l`, `ls` | List all listening ports |
| `portdetective kill <PORT>` | `k` | Kill process on a port |

### Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--json` | `-j` | Output as JSON |
| `--tcp` | — | Only show TCP connections |
| `--udp` | — | Only show UDP connections |
| `--force` | `-f` | Send SIGKILL instead of SIGTERM (kill) |
| `--no-prompt` | `-y` | Skip confirmation prompt (kill) |

### JSON output

```bash
portdetective 3000 --json
```

```json
{
  "port": 3000,
  "protocol": "tcp",
  "status": "in_use",
  "processes": [
    {
      "pid": 42193,
      "name": "node",
      "user": "makafui",
      "command": ["node", "server.js", "--port=3000"],
      "cwd": "/Users/makafui/projects/my-app",
      "parent_pid": 41200,
      "parent_name": "zsh",
      "started": "2025-11-18T14:32:10+02:00",
      "protocol": "tcp"
    }
  ]
}
```

## Philosophy

- **Sharp, boring, dependable**: Does one thing well
- **Instant startup**: No async runtime, fast binary
- **Human-readable by default**: JSON when you need it
- **Safe by default**: Kill requires confirmation unless scripted

## License

MIT
