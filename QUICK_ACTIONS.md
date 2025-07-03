# Quick Actions Configuration Guide

## Overview

Quick Actions allow you to rapidly modify tasks using keyboard shortcuts. After pressing the **Space** key, you can press a configured key to instantly apply an action to the currently selected task.

## Configuration Location

Quick actions are configured in a YAML configuration file. CRIA looks for config in this order:

1. **Custom config file** (if specified with `--config` option)
2. **XDG config directory**: `$XDG_CONFIG_HOME/cria/config.yaml`
3. **Default location**: `~/.config/cria/config.yaml`

### Specifying Custom Config File

```bash
# Use a custom config file
cria --config /path/to/my-config.yaml

# Use a config file in the current directory  
cria --config ./project-config.yaml

# Use the default location
cria
```

### Structure

Each quick action has three properties:
- `key`: The keyboard key to trigger the action (e.g., "w", "u", "p")
- `action`: The type of action ("project", "priority", or "label")
- `target`: The specific target for the action

### Example Configuration

```yaml
quick_actions:
  # Project shortcuts
  - key: "w"
    action: "project"
    target: "Work"
  - key: "p"
    action: "project"
    target: "Personal"
  - key: "h"
    action: "project"
    target: "Home"
  
  # Priority shortcuts
  - key: "u"
    action: "priority"
    target: "5"    # u for urgent (highest priority)
  - key: "l"
    action: "priority"
    target: "1"    # l for low priority
  - key: "m"
    action: "priority"
    target: "3"    # m for medium priority
  
  # Label shortcuts
  - key: "i"
    action: "label"
    target: "Important"
  - key: "t"
    action: "label"
    target: "Today"
```

## Usage

1. **Navigate** to a task using `j`/`k` or arrow keys
2. **Press Space** to enter quick action mode
   - You'll see: "Quick Action Mode: Press a key for quick action or Space to cancel"
3. **Press a configured key** (e.g., `w` for Work project)
4. The action is applied instantly and you'll see a flash on the modified task

### Example Workflow

```
# Move current task to Work project
Space + w

# Set current task priority to urgent (5)
Space + u

# Set current task priority to low (1)
Space + l
```

## Action Types

### Project Actions
- **Action**: `"project"`
- **Target**: Name of an existing project (case-insensitive)
- **Effect**: Moves the task to the specified project

### Priority Actions
- **Action**: `"priority"`
- **Target**: Priority level from "1" (lowest) to "5" (highest)
- **Effect**: Sets the task priority

### Label Actions
- **Action**: `"label"`
- **Target**: Name of a label
- **Effect**: Adds the label to the task (functionality in development)

## Error Handling

- If a project doesn't exist, you'll see an error message
- Invalid priority values (not 1-5) will show an error
- Unknown keys will display "No quick action configured for key: X"
- Quick action mode automatically exits after 2 seconds of inactivity

## Help

Press `?` in the application to see all configured quick actions in the help modal under the "Quick Actions (Space + key)" section.
