use std::fs::OpenOptions;
use std::io::Write;

pub fn debug_log(message: &str) {
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
