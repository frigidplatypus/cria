# CRIA - Quick Add Magic Usage Guide

## Setup

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Update your `.env` file with your Vikunja instance details:
   ```
   VIKUNJA_URL=https://your-vikunja-instance.com
   VIKUNJA_TOKEN=your-auth-token-here
   VIKUNJA_DEFAULT_PROJECT=1
   ```

## Running the Application

```bash
cargo run
```

## Quick Add Magic Modal

Press **`n`** in the main application to open the Quick Add Magic modal.

### Quick Add Magic Syntax

Create tasks using natural language with special syntax:

#### Labels
- `*label` - Simple label
- `*"label with spaces"` - Label with spaces

#### Assignees  
- `@username` - Assign to user
- `@"user name"` - User with spaces

#### Projects
- `+project` - Set project
- `+"project with spaces"` - Project with spaces

#### Priority
- `!1` to `!5` - Set priority (1=lowest, 5=highest)

#### Dates & Times
- **Natural language**: `today`, `tomorrow`, `next monday`, `this weekend`
- **Specific dates**: `15th`, `Feb 17th`, `17/02/2021`
- **Relative dates**: `in 3 days`, `in 2 weeks`
- **Times**: `at 2:30pm`, `at 10am`

#### Repeating Tasks
- `every day` - Daily repetition
- `every 2 weeks` - Every 2 weeks
- `every month` - Monthly

### Examples

```
Buy groceries *shopping @john +personal tomorrow !2
Review proposal *urgent *"high priority" @jane @"john doe" +"Client Work" next monday at 10am !4 every week
Call mom next friday at 2:30pm *personal
Pay rent 15th !3 +finances
Team meeting this weekend *important +work
```

### Keyboard Shortcuts in Modal

- **Enter** - Create task and close modal
- **Escape** - Cancel and close modal
- **Type** - Enter your task with Quick Add Magic syntax

### Main Application Shortcuts

- **`q`** - Quit application
- **`n`** - Open Quick Add Magic modal
- **`j`/`k`** or **Up/Down arrows** - Navigate tasks
- **`r`** - Refresh tasks

## Command Line Usage

You can also use the standalone command line tool:

```bash
cargo run --bin cria-quick "Buy groceries *shopping @john +personal tomorrow !2"
```
