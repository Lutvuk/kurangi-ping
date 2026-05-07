#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;

fn main() {
    tauri::Builder::default()
        .manage(ipc::routing_commands::RoutingCommandState::default())
        .invoke_handler(tauri::generate_handler![
            ipc::routing_commands::routing_toggle_on,
            ipc::routing_commands::routing_toggle_off,
            ipc::detection_commands::detection_get_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
