use std::{fs, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::{parse, parse_macro_input, ItemStruct, LitStr};

use crate::get_opentitan_path;

pub fn registers(args: TokenStream, item: TokenStream) -> TokenStream {
    // Compute path of Opentitan repo
    let mut path = get_opentitan_path();
    let ip_path = PathBuf::from(parse_macro_input!(args as LitStr).value());
    path.push(ip_path);

    // Read & Parse .hjson file of IP description
    let Ok(file_content) = fs::read_to_string(&path) else {
        return parse::Error::new(Span::call_site(), format!("Can not read file {:?}", &path))
            .to_compile_error()
            .into();
    };
    let Ok(ip) = deser_hjson::from_str::<IPDescription>(&file_content) else {
        return parse::Error::new(Span::call_site(), format!("Can not parse file {:?} as IPDescription in HJSON format", &path))
            .to_compile_error()
            .into();
    };

    // Static info for generation
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = input_struct.ident;
    let struct_vis = input_struct.vis;

    // Generate Register descriptions for tock-registers crate
    let mut register_counter = 0;
    let mut registers = vec![gen_interrupt_alert_registers(&mut register_counter)];
    registers.extend(
        ip.registers
            .iter()
            .map(|reg_desc| gen_register(&mut register_counter, reg_desc)),
    );
    registers.push(quote!( (#register_counter => @END), ));

    // Generate Bitfield descriptions for tock-registers crate
    let mut bitfields = vec![
        gen_interrupt_alert_bitfields(format_ident!("intr"), ip.interrupt_list),
        gen_interrupt_alert_bitfields(format_ident!("alert"), ip.alert_list),
    ];
    bitfields.extend(
        ip.registers
            .iter()
            .filter_map(|reg_desc| gen_bitfield(reg_desc)),
    );

    quote!(
        tock_registers::register_structs! {
            #struct_vis #struct_ident {
                #( #registers )*
            }
        }

        tock_registers::register_bitfields! [
            u32,
            #( #bitfields ),*
        ];
    )
    .into()
}

// TODO: FIX DOCUMENTATION GENERATION

/// Generates the interrupt & alert register descriptions
fn gen_interrupt_alert_registers(register_counter: &mut usize) -> proc_macro2::TokenStream {
    let docs = vec![
        "Interrupt State register",
        "Interrupt Enable register",
        "Interrupt Test register",
        "Alert Test register",
    ];
    let offsets = vec![
        *register_counter,
        *register_counter + 4,
        *register_counter + 8,
        *register_counter + 12,
    ];
    *register_counter += 16;
    let names = vec![
        format_ident!("intr_state"),
        format_ident!("intr_enable"),
        format_ident!("intr_test"),
        format_ident!("alert_test"),
    ];
    let types = vec![
        format_ident!("intr"),
        format_ident!("intr"),
        format_ident!("intr"),
        format_ident!("alert"),
    ];

    quote!(
        #(
            #[doc = #docs]
            (#offsets => #names: tock_registers::registers::ReadWrite<u32, #types::Register>),
        )*
    )
}

/// Generates bitfield descriptions for interrupts or alerts
fn gen_interrupt_alert_bitfields(
    name: Ident,
    list: Vec<InterruptDescription>,
) -> proc_macro2::TokenStream {
    // Compute names & offsets
    let names = list
        .iter()
        .map(|x| format_ident!("{}", x.name.to_lowercase()));
    let offsets = 0..list.len();
    let docs = list.iter().map(|x| x.desc.clone());

    quote!(
        #name [
            #(
                #[doc = #docs]
                #names OFFSET(#offsets) NUMBITS(1) []
            ),*
        ]
    )
}

// TODO: support multi register & skipto

/// Generates standard register descriptions
fn gen_register(
    register_counter: &mut usize,
    register_description: &RegisterDescription,
) -> proc_macro2::TokenStream {
    let address = *register_counter;
    *register_counter += 4;
    let name = format_ident!("{}", register_description.name.to_lowercase());
    let reg_type = match register_description.swaccess.as_str() {
        "ro" => format_ident!("ReadOnly"),
        "wo" => format_ident!("WriteOnly"),
        "rw" | "rw0c" | "rw1c" => format_ident!("ReadWrite"),
        x => {
            return parse::Error::new(
                Span::call_site(),
                format!("Unknown swaccess type in file {}", x),
            )
            .to_compile_error()
            .into();
        }
    };
    let doc = &register_description.desc;

    if register_description.fields.is_empty() {
        quote!(
            #[doc = #doc]
            (#address => #name: tock_registers::registers::#reg_type<u32>),
        )
    } else {
        quote!(
            #[doc = #doc]
            (#address => #name: tock_registers::registers::#reg_type<u32, #name::Register>),
        )
    }
}

/// Generates optional standard register bitfields
fn gen_bitfield(register_description: &RegisterDescription) -> Option<proc_macro2::TokenStream> {
    let reg_name = format_ident!("{}", register_description.name.to_lowercase());
    let names = register_description
        .fields
        .iter()
        .map(|f_desc| f_desc.name.clone().unwrap_or("data".to_owned()))
        .map(|name| format_ident!("{}", name.to_lowercase()));
    let offsets = register_description
        .fields
        .iter()
        .map(|f_desc| extract_start(&f_desc.bits));
    let numbits = register_description
        .fields
        .iter()
        .map(|f_desc| extract_end(&f_desc.bits) + 1 - extract_start(&f_desc.bits));
    let docs = register_description.fields.iter().map(|f_desc| {
        f_desc
            .desc
            .clone()
            .unwrap_or("<no documentation>".to_owned())
    });

    if register_description.fields.is_empty() {
        None
    } else {
        Some(quote!(
            #reg_name [
                #(
                    #[doc = #docs]
                    #names OFFSET(#offsets) NUMBITS(#numbits) []
                ),*
            ]
        ))
    }
}

fn extract_start(s: &str) -> usize {
    if let Some((_, s)) = s.split_once(':') {
        s.parse().expect("Start parsing failed")
    } else {
        s.parse().expect("Start parsing failed")
    }
}
fn extract_end(s: &str) -> usize {
    if let Some((e, _)) = s.split_once(':') {
        e.parse().expect("Start parsing failed")
    } else {
        s.parse().expect("Start parsing failed")
    }
}

#[derive(Deserialize)]
struct IPDescription {
    interrupt_list: Vec<InterruptDescription>,
    alert_list: Vec<InterruptDescription>,
    registers: Vec<RegisterDescription>,
}

#[derive(Deserialize)]
struct InterruptDescription {
    name: String,
    desc: String,
}

#[derive(Deserialize)]
struct RegisterDescription {
    name: String,
    desc: String,
    swaccess: String,
    fields: Vec<FieldDescription>,
}

#[derive(Deserialize)]
struct FieldDescription {
    bits: String,
    name: Option<String>,
    desc: Option<String>,
}
