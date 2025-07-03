# CRIA Column Layouts Documentation

## Overview

CRIA features a flexible column system that allows you to customize how task information is displayed in the table. You can create multiple named layouts and switch between them on the fly using keyboard shortcuts.

## Key Features

- 🔄 **Multiple Named Layouts** - Create different column configurations for different workflows
- 📏 **Auto-Width Calculation** - Columns automatically size based on content and available space
- 📝 **Text Wrapping** - Long task titles and labels can wrap to multiple lines
- ⌨️ **Quick Switching** - Use `Shift+H`/`Shift+L` to cycle between layouts
- ⚙️ **Flexible Configuration** - Fine-tune min/max widths and wrapping behavior
- 🔙 **Backward Compatible** - Old percentage-based configs still work

## Available Column Types

| Column Type | Description | Example Content | Wrapping Recommended |
|------------|-------------|-----------------|---------------------|
| `Title` | Task title/name | "Fix bug in authentication system" | ✅ Yes |
| `Project` | Project name | "Work", "Personal", "Home" | ❌ No |
| `Labels` | Task labels/tags | "urgent, bug, backend" | ✅ Yes |
| `DueDate` | Task due date | "12/25/24", "Today", "Overdue" | ❌ No |
| `StartDate` | Task start date | "12/20/24", "Next Week" | ❌ No |
| `Priority` | Task priority level | "P1", "P3", "P5" | ❌ No |
| `Status` | Task completion status | "Open", "Done" | ❌ No |
| `Assignees` | Assigned team members | "alice, bob, charlie" | ✅ Yes |
| `Created` | Task creation date | "12/15/24" | ❌ No |
| `Updated` | Last modified date | "12/22/24" | ❌ No |

## Configuration Options

### Column Properties

Each column in a layout can be configured with the following properties:

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | String | ✅ Yes | Display name for the column header |
| `column_type` | String | ✅ Yes | Column type (see table above) |
| `min_width` | Number | ❌ No | Minimum width in characters |
| `max_width` | Number | ❌ No | Maximum width in characters (unlimited if not set) |
| `wrap_text` | Boolean | ❌ No | Whether to wrap long text (default: false) |
| `enabled` | Boolean | ❌ No | Whether to show this column (default: true) |

### Layout Properties

Each layout has these properties:

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | String | ✅ Yes | Unique layout identifier |
| `description` | String | ❌ No | Human-readable description |
| `columns` | Array | ✅ Yes | List of column configurations |

## Configuration Examples

### Basic Configuration

Add this to your `~/.config/cria/config.yaml`:

```yaml
# Named column layouts
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

# Set active layout (optional)
active_layout: "default"
```

### Complete Multi-Layout Setup

```yaml
column_layouts:
  # Standard layout with all essential columns
  - name: "default"
    description: "Standard task view with all essential columns"
    columns:
      - name: "Task"
        column_type: "Title"
        min_width: 25
        wrap_text: true
        enabled: true
      - name: "Project"
        column_type: "Project"
        min_width: 10
        max_width: 18
        enabled: true
      - name: "Due Date"
        column_type: "DueDate"
        min_width: 10
        max_width: 12
        enabled: true
      - name: "Start Date"
        column_type: "StartDate"
        min_width: 10
        max_width: 12
        enabled: true
      - name: "Labels"
        column_type: "Labels"
        min_width: 8
        max_width: 25
        wrap_text: true
        enabled: true

  # Minimal layout for focused work
  - name: "minimal"
    description: "Clean, minimal view"
    columns:
      - name: "Task"
        column_type: "Title"
        min_width: 40
        wrap_text: true
        enabled: true
      - name: "Due"
        column_type: "DueDate"
        min_width: 10
        max_width: 12
        enabled: true

  # Project-focused layout for team collaboration
  - name: "project-focused"
    description: "Project-centric view for team work"
    columns:
      - name: "Project"
        column_type: "Project"
        min_width: 15
        max_width: 20
        enabled: true
      - name: "Task"
        column_type: "Title"
        min_width: 25
        wrap_text: true
        enabled: true
      - name: "Assignees"
        column_type: "Assignees"
        min_width: 10
        max_width: 20
        wrap_text: true
        enabled: true
      - name: "Priority"
        column_type: "Priority"
        min_width: 8
        max_width: 10
        enabled: true
      - name: "Status"
        column_type: "Status"
        min_width: 8
        max_width: 10
        enabled: true

  # Time management layout for scheduling
  - name: "time-management"
    description: "Time-focused view for scheduling"
    columns:
      - name: "Task"
        column_type: "Title"
        min_width: 25
        wrap_text: true
        enabled: true
      - name: "Start"
        column_type: "StartDate"
        min_width: 10
        max_width: 12
        enabled: true
      - name: "Due"
        column_type: "DueDate"
        min_width: 10
        max_width: 12
        enabled: true
      - name: "Created"
        column_type: "Created"
        min_width: 10
        max_width: 12
        enabled: true
      - name: "Updated"
        column_type: "Updated"
        min_width: 10
        max_width: 12
        enabled: true

active_layout: "default"
```

## Usage

### Switching Layouts

- **`Shift+H`** - Switch to previous layout
- **`Shift+L`** - Switch to next layout

When you switch layouts, CRIA will show a debug message with the layout name and description.

### Layout Order

Layouts are cycled in the order they appear in your configuration file. For example, with the config above:
1. default → minimal → project-focused → time-management → default...
2. Or reverse: default → time-management → project-focused → minimal → default...

## Width Calculation Logic

CRIA uses intelligent width calculation to make the best use of available space:

1. **Minimum Widths First** - All columns get their minimum width
2. **Respect Maximum Widths** - Columns with max_width won't exceed it
3. **Distribute Extra Space** - Remaining space goes to flexible columns (those without max_width)
4. **Priority to Title** - Title columns typically get the most extra space

### Example Width Distribution

For a terminal width of 120 characters with this layout:
```yaml
columns:
  - name: "Task"          # min: 25, max: none
  - name: "Project"       # min: 10, max: 15  
  - name: "Due"           # min: 10, max: 12
```

Result: Task gets ~93 chars, Project gets 15 chars, Due gets 12 chars.

## Text Wrapping

### When to Use Wrapping

- ✅ **Task Titles** - Long task names need full visibility
- ✅ **Labels** - Multiple labels can be lengthy
- ✅ **Assignees** - Team member lists can be long
- ❌ **Dates** - Fixed format, wrapping not useful
- ❌ **Priority** - Short values (P1, P2, etc.)
- ❌ **Status** - Short values (Open, Done)

### Wrapping Behavior

When `wrap_text: true`:
- Text wraps at word boundaries when possible
- Maintains proper spacing and readability
- Table row height adjusts automatically

When `wrap_text: false` (default):
- Text is truncated with "…" if too long
- Single-line display for consistent table height

## Best Practices

### Layout Design

1. **Start with defaults** - Use the provided layouts as templates
2. **Consider your workflow** - Design layouts for specific use cases
3. **Test different terminal sizes** - Ensure layouts work on small screens
4. **Use descriptive names** - Make layout purposes clear

### Column Configuration

1. **Set appropriate minimums** - Ensure critical info is always visible
2. **Use maximums for fixed-width data** - Dates, priorities, status
3. **Enable wrapping selectively** - Only for columns that benefit from it
4. **Consider column order** - Put most important info first

### Performance Tips

1. **Limit total columns** - Too many columns hurt readability
2. **Use minimal layouts** - For better performance with large task lists
3. **Disable unused columns** - Set `enabled: false` instead of removing

## Troubleshooting

### Common Issues

**Q: Layout switching doesn't work**
A: Ensure your config file is valid YAML and the layout names are unique.

**Q: Columns are too narrow**
A: Increase `min_width` values or reduce the number of columns.

**Q: Text is cut off**
A: Enable `wrap_text: true` for that column or increase `max_width`.

**Q: Layout changes aren't applied**
A: Restart CRIA after modifying the config file.

### Validation

CRIA will fall back to default layouts if:
- Config file has syntax errors
- Required fields are missing
- Column types are invalid

Check the debug pane (press `i`) for configuration error messages.

## Legacy Configuration

The old percentage-based format is still supported:

```yaml
table_columns:
  - name: "Title"
    column_type: "Title"
    width_percentage: 50
    enabled: true
  - name: "Due Date"
    column_type: "DueDate"
    width_percentage: 30
    enabled: true
```

However, the new layout system is recommended for better flexibility and auto-width calculation.

## Future Enhancements

Planned features for future versions:
- 🎨 Column reordering via drag & drop
- 💾 Per-project layout preferences
- 🎯 Quick layout customization modal
- 📊 More column types (subtasks, time tracking, etc.)
- 🎨 Custom column colors and styling

---

*For more CRIA documentation, see the main README.md file.*
