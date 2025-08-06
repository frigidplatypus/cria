use clap::{Arg, Command};
use cria::config::CriaConfig;
use cria::vikunja_client::VikunjaClient;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("cria-quick")
        .about("Quick task creation for Vikunja using Quick Add Magic syntax")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("task")
                .help("Task description with Quick Add Magic syntax")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Path to config file")
                .env("CRIA_CONFIG")
        )
        .arg(
            Arg::new("url")
                .long("url")
                .short('u')
                .help("Vikunja instance URL (overrides config)")
                .env("VIKUNJA_URL")
        )
        .arg(
            Arg::new("token")
                .long("token")
                .short('t')
                .help("Authentication token (overrides config)")
                .env("VIKUNJA_TOKEN")
        )
        .arg(
            Arg::new("project")
                .long("project")
                .short('p')
                .help("Default project ID/name (overrides config)")
                .env("VIKUNJA_DEFAULT_PROJECT")
        )
        .get_matches();

    let task_text = matches.get_one::<String>("task").unwrap();
    
    // Load config from file or use defaults
    let config_path = matches.get_one::<String>("config").map(|s| s.as_str());
    let config = CriaConfig::load_from_path(config_path);
    
    // Get URL from command line or config
    let vikunja_url = if let Some(url) = matches.get_one::<String>("url") {
        url.clone()
    } else if let Some(ref config) = config {
        config.api_url.clone()
    } else {
        return Err("No Vikunja URL specified. Use --url or set in config file.".into());
    };

    // Get auth token from command line or config
    let auth_token = if let Some(token) = matches.get_one::<String>("token") {
        token.clone()
    } else if let Some(ref config) = config {
        config.get_api_key().map_err(|e| format!("Config error: {}", e))?
    } else {
        return Err("No auth token specified. Use --token or set in config file.".into());
    };

    // Get default project from command line or config
    let default_project = if let Some(project) = matches.get_one::<String>("project") {
        project.clone()
    } else if let Some(ref config) = config {
        config.default_project.clone().unwrap_or_else(|| "1".to_string())
    } else {
        "1".to_string()
    };

    println!("Creating task with Quick Add Magic: {}", task_text);

    // Create Vikunja client
    let client = VikunjaClient::new(vikunja_url, auth_token);
    
    // Parse default project as either ID or name
    let default_project_id = if let Ok(id) = default_project.parse::<i64>() {
        id
    } else {
        // Try to find project by name
        match client.find_or_get_project_id(&default_project).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                eprintln!("Project '{}' not found, using project ID 1", default_project);
                1
            }
            Err(e) => {
                eprintln!("Error looking up project '{}': {}, using project ID 1", default_project, e);
                1
            }
        }
    };

    // Create task with magic syntax
    match client.create_task_with_magic(task_text, default_project_id.max(0) as u64).await {
        Ok(task) => {
            println!("Task created successfully!");
            println!("  ID: {:?}", task.id);
            println!("  Title: {}", task.title);
            println!("  Project: {}", task.project_id);
        }
        Err(e) => eprintln!("Failed to create task: {}", e),
    }

    Ok(())
}
