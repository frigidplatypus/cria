use clap::{Arg, Command};
use cria::vikunja_client::create_quick_task;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("cria-quick")
        .about("Quick task creation for Vikunja using Quick Add Magic syntax")
        .version("0.1.0")
        .arg(
            Arg::new("task")
                .help("Task description with Quick Add Magic syntax")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("url")
                .long("url")
                .short('u')
                .help("Vikunja instance URL")
                .env("VIKUNJA_URL")
                .required(true)
        )
        .arg(
            Arg::new("token")
                .long("token")
                .short('t')
                .help("Authentication token")
                .env("VIKUNJA_TOKEN")
                .required(true)
        )
        .arg(
            Arg::new("project")
                .long("project")
                .short('p')
                .help("Default project ID")
                .env("VIKUNJA_DEFAULT_PROJECT")
                .default_value("1")
        )
        .get_matches();

    let task_text = matches.get_one::<String>("task").unwrap();
    let vikunja_url = matches.get_one::<String>("url").unwrap();
    let auth_token = matches.get_one::<String>("token").unwrap();
    let project_id: u64 = matches.get_one::<String>("project").unwrap().parse()?;

    println!("Creating task with Quick Add Magic: {}", task_text);

    match create_quick_task(vikunja_url.to_string(), auth_token.to_string(), task_text.to_string(), project_id as i64).await {
        Ok(_) => println!("Task created successfully!"),
        Err(e) => eprintln!("Failed to create task: {}", e),
    }

    Ok(())
}
