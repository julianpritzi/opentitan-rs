use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse, parse_macro_input, spanned::Spanned, ItemFn, ReturnType, Type, Visibility};

pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    // check the function has no arguments
    if f.sig.inputs.len() != 0 {
        return parse::Error::new(
            f.sig.inputs.last().unwrap().span(),
            "`#[entry]` function has too many arguments",
        )
        .to_compile_error()
        .into();
    }

    // check the function signature
    let valid_signature = f.sig.constness.is_none()
        && f.sig.asyncness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
        };

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[entry]` function must have signature `[unsafe] fn() -> !`",
        )
        .to_compile_error()
        .into();
    }

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let attrs = f.attrs;
    let unsafety = f.sig.unsafety;
    let stmts = f.block.stmts;

    quote!(
        #[allow(non_snake_case)]
        #[export_name = "main"]
        #(#attrs)*
        pub extern "C" #unsafety fn __opentitan_lib__main() -> ! {
            #(#stmts)*
        }
    )
    .into()
}
