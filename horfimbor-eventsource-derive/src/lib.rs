#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use proc_macro::{self, TokenStream};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

/// `derive_command` generate the boilerplate to get the `CommandName` from the command enum
/// the attribute `state` give the prefix for the name
#[proc_macro_derive(Command, attributes(state))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let state_name = match get_state_name(&input) {
        Ok(value) => value,
        Err(value) => return value,
    };

    // get enum name
    let name = &input.ident;
    let data = &input.data;
    let mut fn_core;

    match data {
        Data::Enum(data_enum) => {
            fn_core = TokenStream2::new();

            // Iterate over enum variants
            // `variants` if of type `Punctuated` which implements IntoIterator
            for variant in &data_enum.variants {
                // Variant's name
                let variant_name = &variant.ident;

                // Variant can have unnamed fields like `Variant(i32, i64)`
                // Variant can have named fields like `Variant {x: i32, y: i32}`
                // Variant can be named Unit like `Variant`
                let fields_in_variant = match &variant.fields {
                    Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
                    Fields::Unit => quote_spanned! { variant.span()=> },
                    Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
                };

                // Here we construct the function for the current variant
                let result = format!(".CMD.{variant_name}");
                fn_core.extend(quote! {
                    #name::#variant_name #fields_in_variant => {

                        const SUFFIX: &str = #result;

                        const LEN: usize = #state_name.len() + SUFFIX.len();
                        const BYTES: [u8; LEN] = {
                            let mut bytes = [0; LEN];

                            let mut i = 0;
                            while i < #state_name.len() {
                                bytes[i] = #state_name.as_bytes()[i];
                                i += 1;
                            }

                            let mut j = 0;
                            while j < SUFFIX.len() {
                                bytes[#state_name.len() + j] = SUFFIX.as_bytes()[j];
                                j += 1;
                            }

                            bytes
                        };

                        match std::str::from_utf8(&BYTES) {
                            Ok(s) => s,
                            Err(_) => unreachable!(),
                        }
                    },
                });
            }
        }
        _ => return derive_error!("Command is only implemented for enums"),
    }

    let output = quote! {
        impl Command for #name {
            fn command_name(&self) -> CommandName {

                match self {
                    #fn_core
                }
            }
        }
    };
    output.into()
}

/// `derive_event` generate the boilerplate to get the `EventName`
///
/// it generates it from the event enum :
/// the attribute `state` give the prefix for the name
/// unless the attribute `composite_state` in which case the current enum level is skip
#[proc_macro_derive(Event, attributes(state, composite_state))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let mut state_name = None;

    let is_composite_state = &input
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("composite_state"));

    if !is_composite_state {
        state_name = match get_state_name(&input) {
            Ok(value) => Some(value),
            Err(value) => return value,
        };
    }

    // get enum name
    let name = &input.ident;
    let data = &input.data;
    let mut fn_core;

    match data {
        Data::Enum(data_enum) => {
            fn_core = TokenStream2::new();

            // Iterate over enum variants
            // `variants` if of type `Punctuated` which implements IntoIterator
            for variant in &data_enum.variants {
                // Variant's name
                let variant_name = &variant.ident;

                if *is_composite_state {
                    // Variant can have unnamed fields like `Variant(i32, i64)`
                    // Variant can have named fields like `Variant {x: i32, y: i32}`
                    // Variant can be named Unit like `Variant`
                    let fields_in_variant = match &variant.fields {
                        Fields::Unnamed(_) => quote_spanned! {variant.span()=> (event) },
                        _ => {
                            return derive_error!(
                                "composite variants can only have one unnamed fields "
                            );
                        }
                    };

                    fn_core.extend(quote! {
                        #name::#variant_name #fields_in_variant => {

                            event.event_name()
                        },
                    });
                } else {
                    // Variant can have unnamed fields like `Variant(i32, i64)`
                    // Variant can have named fields like `Variant {x: i32, y: i32}`
                    // Variant can be named Unit like `Variant`
                    let fields_in_variant = match &variant.fields {
                        Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
                        Fields::Unit => quote_spanned! { variant.span()=> },
                        Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
                    };

                    // Here we construct the function for the current variant
                    let result = format!(".evt.{}", variant_name.to_string().to_case(Case::Snake));
                    fn_core.extend(quote! {
                        #name::#variant_name #fields_in_variant => {

                            const SUFFIX: &str = #result;

                            const LEN: usize = #state_name.len() + SUFFIX.len();
                            const BYTES: [u8; LEN] = {
                                let mut bytes = [0; LEN];

                                let mut i = 0;
                                while i < #state_name.len() {
                                    bytes[i] = #state_name.as_bytes()[i];
                                    i += 1;
                                }

                                let mut j = 0;
                                while j < SUFFIX.len() {
                                    bytes[#state_name.len() + j] = SUFFIX.as_bytes()[j];
                                    j += 1;
                                }

                                bytes
                            };

                            match std::str::from_utf8(&BYTES) {
                                Ok(s) => s,
                                Err(_) => unreachable!(),
                            }
                        },
                    });
                }
            }
        }
        _ => return derive_error!("Event is only implemented for enums"),
    }

    let output = quote! {
        impl Event for #name {
            fn event_name(&self) -> EventName {
                match self {
                    #fn_core
                }
            }
        }
    };
    output.into()
}

/// # Panics
///
/// Will panic if attribute "state" is not parsable
#[proc_macro_derive(StateNamed, attributes(state))]
pub fn derive_state(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let attrs = &input.attrs;
    let name = &input.ident;

    let state = attrs.iter().find(|attr| attr.path().is_ident("state"));

    let output = match state {
        Some(s) => {
            let state_name: syn::Ident = match s.parse_args() {
                Ok(s) => s,
                Err(_) => {
                    return derive_error!("attribute 'state' cannot be parsed");
                }
            };
            quote! {
                impl StateNamed for #name {
                    fn state_name() -> StateName {
                        #state_name
                    }
                }
            }
        }
        None => {
            return derive_error!("attribute 'state' is mandatory");
        }
    };

    output.into()
}

fn get_state_name(input: &DeriveInput) -> Result<Ident, TokenStream> {
    let attrs = &input.attrs;

    let state = attrs.iter().find(|attr| attr.path().is_ident("state"));

    let Some(state) = state else {
        return Err(derive_error!("attribute 'state' is mandatory"));
    };

    let state_name: syn::Ident = match state.parse_args() {
        Ok(s) => s,
        Err(_) => {
            return Err(derive_error!("attribute 'state' cannot be parsed"));
        }
    };
    Ok(state_name)
}
