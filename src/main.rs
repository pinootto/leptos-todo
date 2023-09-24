use leptos::*;
use todo_webui::show_todos;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(show_todos)
}
