use std::{fs, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::{parse, parse_macro_input, LitStr};

use crate::get_opentitan_path;

pub fn addresses(args: TokenStream) -> TokenStream {
    // Compute path of Opentitan repo
    let mut path = get_opentitan_path();
    let ip_path = PathBuf::from(parse_macro_input!(args as LitStr).value());
    path.push(ip_path);

    // Read & Parse .hjson file of Top description
    let Ok(file_content) = fs::read_to_string(&path) else {
        return parse::Error::new(Span::call_site(), format!("Can not read file {:?}", &path))
            .to_compile_error()
            .into();
    };
    let Ok(top) = deser_hjson::from_str::<TopDescription>(&file_content) else {
        return parse::Error::new(Span::call_site(), format!("Can not parse file {:?} as TopDescription in HJSON format", &path))
            .to_compile_error()
            .into();
    };

    let name = top.module.iter().filter_map(|m| {
        parse_addr(&m.base_addr).and(Some(format_ident!("{}", &m.name.to_uppercase())))
    });

    let address = top.module.iter().filter_map(|m| parse_addr(&m.base_addr));

    let docs = top
        .module
        .iter()
        .filter_map(|m| m.base_addr.clone().map(|d| format!("Address: {d}")));

    quote!(
        pub mod addresses {
            #(
                #[doc = #docs]
                pub const #name: *const u8 = #address as *const u8;
            )*
        }
    )
    .into()
}

fn parse_addr(s: &Option<String>) -> Option<usize> {
    if let Some(s) = s {
        let s = s.trim_start_matches("0x");
        usize::from_str_radix(s, 16).ok()
    } else {
        None
    }
}

#[derive(Deserialize)]
struct TopDescription {
    module: Vec<ModuleDescription>,
}

#[derive(Deserialize)]
struct ModuleDescription {
    name: String,
    base_addr: Option<String>,
}
