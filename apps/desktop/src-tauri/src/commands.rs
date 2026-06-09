use tauri::command;

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! WorkGrid Memory is ready.", name)
}

#[command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
