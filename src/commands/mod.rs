use crate::types::{command_request, command_response};

pub mod copy_item;
pub mod create_directory;
pub mod decrypt;
pub mod encrypt;
pub mod find_items;
pub mod generate_password;
pub mod initialize;
pub mod list_items;
pub mod move_item;
pub mod read_file;
pub mod remove_directory;
pub mod remove_file;
pub mod search_content;
pub mod write_file;

pub use copy_item::copy_item;
pub use create_directory::create_directory;
pub use decrypt::decrypt_string;
pub use encrypt::encrypt_string;
pub use find_items::find_items;
pub use generate_password::generate_password;
pub use initialize::initialize;
pub use list_items::list_items;
pub use move_item::move_item;
pub use read_file::read_file;
pub use remove_directory::remove_directory;
pub use remove_file::remove_file;
pub use search_content::filter_lines;
pub use write_file::write_file;

pub fn handler(request: &command_request::Request) -> Option<command_response::Response> {
    match &request.command as &str {
        "initialize" => initialize::interface(&request.parameters),
        "mkdir" => create_directory::interface(&request.parameters),
        "rmdir" => remove_directory::interface(&request.parameters),
        "read" => read_file::interface(&request.parameters),
        "write" => write_file::interface(&request.parameters),
        _ => None,
    }
}
