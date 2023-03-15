use std::{fs, path::PathBuf, vec};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::{parse, parse_macro_input, ItemStruct, LitStr};

use crate::{get_opentitan_path, hjson_sanitizer};

// TODO: Refactor and make more robust by using reggen from opentitan

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
    // Parse file content
    let ip = match deser_hjson::from_str::<IPDescription>(&hjson_sanitizer::sanitize(file_content))
    {
        Ok(ip) => ip,
        Err(error) => {
            return parse::Error::new(
                Span::call_site(),
                format!(
                    "Can not parse file {:?} as IPDescription in HJSON format | \n {}",
                    &path, error
                ),
            )
            .to_compile_error()
            .into()
        }
    };

    // Static info for generation
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_ident = input_struct.ident;
    let struct_vis = input_struct.vis;

    // Generate Register descriptions for tock-registers crate
    let mut register_counter = 0;
    let mut registers = vec![gen_interrupt_alert_registers(
        &mut register_counter,
        ip.interrupt_list.is_some(),
    )];
    registers.extend(
        ip.registers
            .iter()
            .map(|reg_desc| gen_register(&mut register_counter, reg_desc, &ip.param_list)),
    );
    registers.push(quote!( (#register_counter => @END), ));

    // Generate Bitfield descriptions for tock-registers crate
    let mut bitfields = vec![];
    if let Some(intr_list) = ip.interrupt_list {
        bitfields.push(gen_interrupt_alert_bitfields(
            format_ident!("intr"),
            intr_list,
        ));
    }
    bitfields.push(gen_interrupt_alert_bitfields(
        format_ident!("alert"),
        ip.alert_list,
    ));
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

/// Generates the interrupt & alert register descriptions
fn gen_interrupt_alert_registers(
    register_counter: &mut usize,
    gen_intr: bool,
) -> proc_macro2::TokenStream {
    let mut docs = vec![];
    let mut offsets = vec![];
    let mut names = vec![];
    let mut types = vec![];
    if gen_intr {
        docs.extend(vec![
            "([`intr`]) Interrupt State register",
            "([`intr`]) Interrupt Enable register",
            "([`intr`]) Interrupt Test register",
        ]);
        offsets.extend(vec![
            *register_counter,
            *register_counter + 4,
            *register_counter + 8,
        ]);
        names.extend(vec![
            format_ident!("intr_state"),
            format_ident!("intr_enable"),
            format_ident!("intr_test"),
        ]);
        types.extend(vec![
            format_ident!("intr"),
            format_ident!("intr"),
            format_ident!("intr"),
        ]);
        *register_counter += 12;
    }
    docs.push("([`alert`]) Alert Test register");
    offsets.push(*register_counter);
    *register_counter += 4;
    names.push(format_ident!("alert_test"));
    types.push(format_ident!("alert"));

    quote!(
        #(
            #[doc = #docs]
            (#offsets => pub #names: tock_registers::registers::ReadWrite<u32, #types::Register>),
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
        /// All the submodules/constants represent parts of the content of this register
        pub #name [
            #(
                #[doc = #docs]
                #names OFFSET(#offsets) NUMBITS(1) []
            ),*
        ]
    )
}

// TODO: support multi register & skipto
//
// for multireg: look at count & cross reference with paramaters
// for skipto direct

/// Generates standard register descriptions
fn gen_register(
    register_counter: &mut usize,
    register_description: &RegisterDescription,
    params: &Option<Vec<ParamDescription>>,
) -> proc_macro2::TokenStream {
    let address = *register_counter;

    let (str_name, swaccess, desc, fields, items) = match register_description {
        RegisterDescription {
            name: Some(name),
            swaccess: Some(swaccess),
            desc,
            fields,
            ..
        } => (name, swaccess, desc, fields, None),
        RegisterDescription {
            window:
                Some(WindowDescription {
                    name,
                    swaccess,
                    desc,
                    items,
                    fields,
                }),
            ..
        } => (
            name,
            swaccess,
            desc,
            fields,
            Some(items.parse::<usize>().expect("Invalid item number")),
        ),
        RegisterDescription {
            multireg:
                Some(MultiregDescription {
                    name,
                    desc,
                    swaccess,
                    count,
                    field,
                }),
            ..
        } => (
            name,
            swaccess,
            desc,
            field,
            Some(parse_count(count, params)),
        ),
        RegisterDescription {
            skipto: Some(skipto),
            ..
        } => match usize::from_str_radix(&skipto.trim_start_matches("0x"), 16) {
            Ok(val) => {
                *register_counter = val;
                let name = format_ident!("_ignored{}", address);
                return quote!(
                    (#address => #name),
                );
            }
            Err(_) => {
                panic!("Invalid skipto:({}) value in file", skipto);
            }
        },
        _ => {
            panic!("unknown register description in file")
        }
    };

    let name = format_ident!("{}", str_name.to_lowercase());
    let reg_type = match swaccess.as_str() {
        "ro" => format_ident!("ReadOnly"),
        "wo" => format_ident!("WriteOnly"),
        "rw" | "rw0c" | "rw1c" | "r0w1c" => format_ident!("ReadWrite"),
        x => {
            panic!("Unknown swaccess type in file {}", x);
        }
    };

    if fields.is_none() {
        let doc = &desc.clone().unwrap_or("<no description>".to_owned());
        if let Some(item_num) = items {
            *register_counter += 4 * item_num;
            quote!(
                #[doc = #doc]
                (#address => pub #name: [tock_registers::registers::#reg_type<u32>; #item_num]),
            )
        } else {
            *register_counter += 4;
            quote!(
                #[doc = #doc]
                (#address => pub #name: tock_registers::registers::#reg_type<u32>),
            )
        }
    } else {
        let doc = format!(
            "([`{}`]) {}",
            str_name.to_lowercase(),
            &desc.clone().unwrap_or("<no description>".to_owned())
        );
        if let Some(item_num) = items {
            *register_counter += 4 * item_num;
            quote!(
                #[doc = #doc]
                (#address => pub #name: [tock_registers::registers::#reg_type<u32, #name::Register>; #item_num]),
            )
        } else {
            *register_counter += 4;
            quote!(
                #[doc = #doc]
                (#address => pub #name: tock_registers::registers::#reg_type<u32, #name::Register>),
            )
        }
    }
}

/// Generates optional standard register bitfields
fn gen_bitfield(register_description: &RegisterDescription) -> Option<proc_macro2::TokenStream> {
    let (reg_name, fields) = match register_description {
        RegisterDescription {
            name: Some(name),
            fields: Some(fields),
            ..
        } => (name, fields),
        RegisterDescription {
            window:
                Some(WindowDescription {
                    name,
                    fields: Some(fields),
                    ..
                }),
            ..
        } => (name, fields),
        _ => {
            return None;
        }
    };

    let reg_name = format_ident!("{}", reg_name.to_lowercase());
    let names = fields
        .iter()
        .map(|f_desc| f_desc.name.clone().unwrap_or("data".to_owned()))
        .map(|name| format_ident!("{}", name.to_lowercase()));
    let offsets = fields.iter().map(|f_desc| extract_start(&f_desc.bits));
    let numbits = fields
        .iter()
        .map(|f_desc| extract_end(&f_desc.bits) + 1 - extract_start(&f_desc.bits));
    let docs = fields.iter().map(|f_desc| {
        f_desc
            .desc
            .clone()
            .unwrap_or("<no documentation>".to_owned())
    });

    if fields.is_empty() {
        None
    } else {
        Some(quote!(
            /// All the submodules/constants represent parts of the content of this register
            pub #reg_name [
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
        s.parse().expect("Bitfield bits start parsing failed")
    } else {
        s.parse().expect("Bitfield bits start parsing failed")
    }
}
fn extract_end(s: &str) -> usize {
    if let Some((e, _)) = s.split_once(':') {
        e.parse().expect("Bitfield bits end parsing failed")
    } else {
        s.parse().expect("Bitfield bits end parsing failed")
    }
}
fn parse_count(s: &str, params: &Option<Vec<ParamDescription>>) -> usize {
    match s.parse::<usize>() {
        Ok(val) => val,
        Err(_) => {
            let params = params
                .as_ref()
                .expect("Count reference with no paramater list in file");
            let val = params
                .iter()
                .find(|x| x.name == s)
                .expect("Invalid count reference in file")
                .default
                .as_ref()
                .expect("Referenced count value is not set");
            val.parse().expect("Invalid value of refrenced param")
        }
    }
}

#[derive(Deserialize, Debug)]
struct IPDescription {
    interrupt_list: Option<Vec<InterruptDescription>>,
    alert_list: Vec<InterruptDescription>,
    registers: Vec<RegisterDescription>,
    param_list: Option<Vec<ParamDescription>>,
}

#[derive(Deserialize, Debug)]
struct InterruptDescription {
    name: String,
    desc: String,
}

#[derive(Deserialize, Debug)]
struct RegisterDescription {
    skipto: Option<String>,
    window: Option<WindowDescription>,
    name: Option<String>,
    desc: Option<String>,
    swaccess: Option<String>,
    fields: Option<Vec<FieldDescription>>,
    multireg: Option<MultiregDescription>,
}

#[derive(Deserialize, Debug)]
struct ParamDescription {
    name: String,
    default: Option<String>,
}

#[derive(Deserialize, Debug)]
struct WindowDescription {
    name: String,
    desc: Option<String>,
    swaccess: String,
    items: String,
    fields: Option<Vec<FieldDescription>>,
}

#[derive(Deserialize, Debug)]
struct MultiregDescription {
    name: String,
    desc: Option<String>,
    swaccess: String,
    count: String,
    field: Option<Vec<FieldDescription>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct FieldDescription {
    bits: String,
    name: Option<String>,
    desc: Option<String>,
    resval: Option<String>,
}
