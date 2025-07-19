# CRIA

cria is a fast, keyboard-driven terminal client for managing tasks with the Vikunja API. It features a highly customizable interface, powerful filtering, and flexible column layouts for productivity-focused workflows.

## Getting Started

1. **Install CRIA**
   - Build from source with Cargo:
     ```fish
     cargo build --release
     ```
   - Run the binary:
     ```fish
     ./target/release/cria
     ```

2. **Configuration**
   - CRIA uses a YAML config file for all settings. The config file is typically located at:
     - `$XDG_CONFIG_HOME/cria/config.yaml` (if XDG is set)
     - Otherwise: `~/.config/cria/config.yaml`
   - You can specify a custom config file with `--config /path/to/config.yaml`.
   - An example config is provided as `config.example.yaml`. Copy and edit as needed:
     ```fish
     cp config.example.yaml ~/.config/cria/config.yaml
     ```

### Example `config.yaml`
```yaml
api_url: "https://vikunja.example.com/api/v1"
api_key: "your-api-key-here"
default_project: "Inbox"
default_filter: "Work"

# Optional: Automatically load a specific filter on startup
# default_filter: "Daily Tasks"

# Auto-refresh settings
auto_refresh: true
refresh_interval_seconds: 300

# Quick actions for fast task modification
quick_actions:
  - key: "w"
    action: "project"
    target: "Work"
  - key: "u"
    action: "priority"
    target: "5"
  - key: "i"
    action: "label"
    target: "Important"

# Column layouts for table view
column_layouts:
  - name: "default"
    description: "Standard task view"
    columns:
      - name: "Task"
        column_type: "Title"
        min_width: 25
        wrap_text: true
        enabled: true
      - name: "Due"
        column_type: "DueDate"
        min_width: 10
        max_width: 12
        enabled: true
      - name: "Project"
        column_type: "Project"
        min_width: 10
        max_width: 15
        enabled: true
active_layout: "default"
```

## Filters and `cria_project` Option

CRIA supports powerful task filtering, including custom filters defined in Vikunja. You can add a special `cria_project` override to a filter's description to change the active project context when the filter is applied.

### How to Use `cria_project` in Filters
- In Vikunja, create or edit a filter.
- Add a line to the filter's description:
  ```
  <code>cria_project: ProjectName</code>
  ```
- When this filter is selected in CRIA all tasks will default to being created in this project.

## More Information
- See `config.example.yaml` for all available configuration options.
- See `COLUMN_LAYOUTS.md` for details on customizing table columns and layouts.
- See `QUICK_ACTIONS.md` for quick action setup and usage.

## License
CRIA is open source. See `LICENSE` for details.
