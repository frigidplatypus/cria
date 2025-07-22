use std::fs::OpenOptions;
use std::io::Write;

pub fn clear_debug_log() {
    // Only clear log if CRIA_DEBUG is set
    if std::env::var("CRIA_DEBUG").is_err() {
        return;
    }
    
    // Truncate the log file to clear it
    if let Ok(_) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("cria_debug.log")
    {
        // File has been cleared
    }
}

pub fn debug_log(message: &str) {
    // Only log to file if CRIA_DEBUG is set
    if std::env::var("CRIA_DEBUG").is_err() {
        return;
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_message = format!("[{}] {}\n", timestamp, message);
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("cria_debug.log")
    {
        let _ = file.write_all(log_message.as_bytes());
        let _ = file.flush();
    }
}
