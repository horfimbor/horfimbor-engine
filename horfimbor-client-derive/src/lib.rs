use proc_macro::{self, TokenStream};

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::__private::quote::quote;
use syn::{Data, DeriveInput, Error, parse_macro_input};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

#[proc_macro_derive(WebComponent, attributes(component))]
pub fn derive_web_component(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let attrs = &input.attrs;

    let Some(component) = attrs.iter().find(|attr| attr.path().is_ident("component")) else{
        return derive_error!("WebComponent need a base component");
    };

    let component: syn::Ident = match component.parse_args() {
        Ok(s) => s,
        Err(_) => {
            return derive_error!("attribute 'state' cannot be parsed");
        }
    };

    let name = &input.ident;
    let data = &input.data;

    let mut opt_struct = TokenStream2::new();
    let mut default_props = TokenStream2::new();
    let mut observed = TokenStream2::new();
    let mut get_attributes = TokenStream2::new();
    let mut check_none = TokenStream2::new();
    let mut real_props = TokenStream2::new();

    match data {
        Data::Struct(data_struct) => {
            for attr in &data_struct.fields {
                let Some(ident) = &attr.ident else {
                    break;
                };
                let kind = &attr.ty;

                opt_struct.extend(quote! {
                    #ident : Option<#kind>,
                });

                default_props.extend(quote! {
                    #ident : None,
                });

                let s = format!("{ident}");
                observed.extend(quote! {
                    #s,
                });

                get_attributes.extend(quote! {
                    #ident: this.get_attribute(#s),
                });
                check_none.extend(quote! {
                    let Some(#ident) = ctx.props().#ident.clone() else{
                        return html!();
                    };
                });
                real_props.extend(quote! {
                    #ident={{#ident.clone()}}
                });
            }
        }
        _ => return derive_error!("WebComponent is only implemented for struct"),
    }

    let output = quote! {
        pub mod optional {

            use yew::AppHandle;
            use yew::prelude::*;
            use custom_elements::CustomElement;
            use web_sys::HtmlElement;
            use super::#component;

            #[derive(Default, Properties, PartialEq)]
            pub struct #name {
                #opt_struct
            }

            struct WrappedComponent{}

            impl Component for WrappedComponent{
                type Message = ();
                type Properties = #name;

                fn create(_ctx: &Context<Self>) -> Self {
                    Self {}
                }

                fn view(&self, ctx: &Context<Self>) -> Html {
                    #check_none
                    html! {
                        <#component #real_props>
                        </#component>
                    }
                }

            }

            #[derive(Default)]
            pub struct ComponentWrapper {
                content: Option<AppHandle<WrappedComponent>>,
            }

            impl CustomElement for ComponentWrapper {
                fn inject_children(&mut self, this: &HtmlElement) {
                    let props = #name {
                        #default_props
                    };

                    self.content = Some(
                        yew::Renderer::<WrappedComponent>::with_root_and_props(this.clone().into(), props).render(),
                    );
                }

                fn shadow() -> bool {
                    false
                }

                fn observed_attributes() -> &'static [&'static str] {
                    &[#observed]
                }

                fn attribute_changed_callback(
                    &mut self,
                    this: &HtmlElement,
                    _name: String,
                    _old_value: Option<String>,
                    _new_value: Option<String>,
                ) {
                    let props = #name {
                        #get_attributes
                    };

                    match &mut self.content {
                        None => {}
                        Some(handle) => {
                            handle.update(props);
                        }
                    }
                }
            }
        }
    };

    output.into()
}
