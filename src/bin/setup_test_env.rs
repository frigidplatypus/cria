// src/bin/setup_test_env.rs
// Usage: cargo run --bin setup_test_env
// Seeds Vikunja with demo projects, labels, and tasks for testing.

use std::env;
use chrono::{Utc, Duration};
use tokio;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    let whatif = args.iter().any(|a| a == "-whatif" || a == "--whatif");
    let api_url = env::var("VIKUNJA_API_URL").expect("VIKUNJA_API_URL not set");
    let api_token = env::var("VIKUNJA_API_TOKEN").expect("VIKUNJA_API_TOKEN not set");
    let client = cria::vikunja_client::VikunjaClient::new(api_url, api_token);

    // Labels
    let label_defs = vec![
        ("urgent", "#ff0000"),
        ("home", "#00bfff"),
        ("work", "#228b22"),
        ("personal", "#ff69b4"),
        ("feature", "#ffa500"),
        ("bug", "#8b0000"),
        ("review", "#9370db"),
        ("meeting", "#ffd700"),
        ("low", "#808080"),
        ("high", "#00ff00"),
    ];
    let mut label_ids = std::collections::HashMap::new();
    let existing_labels = client.get_all_labels().await.expect("Failed to get labels");
    let mut new_labels = 0;
    for (name, color) in &label_defs {
        let found = existing_labels.iter().find(|l| l.title == *name);
        let id = if let Some(label) = found {
            label.id.unwrap()
        } else {
            if whatif {
                new_labels += 1;
                0 // dummy id
            } else {
                let new_label = client.create_label(name).await.expect("Failed to create label");
                new_label.id.unwrap()
            }
        };
        label_ids.insert(*name, id);
    }

    // Projects and their tasks
    let projects = vec![
        ("Planning a Roadtrip", "#00bfff", vec![
            ("Book hotels", Some(1), true, Some(1)),
            ("Plan route", Some(2), false, Some(2)),
            ("Pack bags", None, false, None),
            ("Check car maintenance", Some(3), false, Some(3)),
            ("Create playlist", None, false, None),
        ]),
        ("Family", "#ff69b4", vec![
            ("Plan family dinner", Some(1), false, Some(2)),
            ("Call parents", None, false, None),
            ("Organize photo album", None, false, None),
            ("Schedule game night", Some(2), true, Some(1)),
            ("Help with homework", None, false, None),
        ]),
        ("Home Maintenance", "#ffa500", vec![
            ("Clean gutters", Some(2), false, Some(2)),
            ("Change HVAC filter", None, false, None),
            ("Test smoke detectors", None, false, None),
            ("Fix leaky faucet", Some(3), false, Some(3)),
            ("Organize garage", None, false, None),
        ]),
        ("Personal Improvement", "#00ff00", vec![
            ("Read a book", None, false, None),
            ("Exercise", Some(1), true, Some(1)),
            ("Meditate", None, false, None),
            ("Learn a new recipe", None, false, None),
            ("Practice a hobby", None, false, None),
        ]),
        ("Homelab", "#9370db", vec![
            ("Update server firmware", Some(1), false, Some(2)),
            ("Backup configs", None, false, None),
            ("Test UPS", None, false, None),
            ("Document network", None, false, None),
            ("Set up monitoring", Some(2), true, Some(3)),
        ]),
    ];
    let mut project_ids = std::collections::HashMap::new();
    let existing_projects = client.get_all_projects().await.expect("Failed to get projects");
    let mut new_projects = 0;
    let mut new_tasks = 0;
    for (project, color, tasks) in &projects {
        let found = existing_projects.iter().find(|p| p.title == *project);
        let project_id = if let Some(p) = found {
            p.id as u64
        } else {
            if whatif {
                new_projects += 1;
                0 // dummy id
            } else {
                let new_project = client.create_project(project, color).await.expect("Failed to create project");
                new_project.id as u64
            }
        };
        project_ids.insert(*project, project_id);
        // Add tasks
        for (i, (title, due_offset, starred, priority)) in tasks.iter().enumerate() {
            let due_date = due_offset.map(|days| (Utc::now() + Duration::days(days)).to_rfc3339());
            let mut label_set = vec![];
            if i % 2 == 0 { label_set.push(label_ids["urgent"]); }
            if i % 3 == 0 { label_set.push(label_ids["feature"]); }
            if whatif {
                new_tasks += 1;
                continue;
            }
            let label_objs: Vec<_> = label_set.iter().map(|id| serde_json::json!({"id": id})).collect();
            let mut payload = serde_json::json!({
                "title": title,
                "project_id": project_id,
                "starred": *starred,
                "priority": priority.unwrap_or(1),
                "labels": label_objs,
            });
            if let Some(due) = due_date {
                payload["due_date"] = serde_json::json!(due);
            }
            let url = format!("{}/api/v1/projects/{}/tasks", client.base_url(), project_id);
            let resp = client.client().put(&url)
                .header("Authorization", format!("Bearer {}", client.auth_token()))
                .json(&payload)
                .send().await;
            match resp {
                Ok(r) if r.status().is_success() => println!("Created task '{}' in project '{}'", title, project),
                Ok(r) => {
                    let status = r.status();
                    let err_text = r.text().await.unwrap_or_else(|_| "<no body>".to_string());
                    println!("Failed to create task '{}': {}\nResponse body: {}", title, status, err_text);
                },
                Err(e) => println!("Error creating task '{}': {}", title, e),
            }
        }
    }
    if whatif {
        println!("\nWHATIF SUMMARY:");
        println!("Would create {} new labels, {} new projects, {} new tasks.", new_labels, new_projects, new_tasks);
    } else {
        println!("Test environment setup complete.");
    }
}
