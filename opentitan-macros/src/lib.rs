use std::{env, path::PathBuf};

use proc_macro::TokenStream;

mod addresses;
mod entry;
mod hjson_sanitizer;
mod registers;

/// Attribute to declare the entry point of the program
///
/// **IMPORTANT**: This attribute must appear exactly *once* in the dependency graph.
///
/// The type of the specified function must be `[unsafe] fn() -> !`
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::entry(args, input)
}

#[proc_macro_attribute]
pub fn registers(args: TokenStream, item: TokenStream) -> TokenStream {
    registers::registers(args, item)
}

#[proc_macro]
pub fn addresses(args: TokenStream) -> TokenStream {
    addresses::addresses(args)
}

fn get_opentitan_path() -> PathBuf {
    if let Ok(ot_path) = env::var("OPENTITAN_PATH") {
        PathBuf::from(ot_path)
    } else {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let mut path = PathBuf::from(manifest_dir);
        path.pop();
        path.push("opentitan");
        path
    }
}
