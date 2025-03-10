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

#[allow(clippy::too_many_lines)]
#[proc_macro_derive(WebComponent, attributes(component, optionnal))]
pub fn derive_web_component(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let attrs = &input.attrs;

    let Some(component) = attrs.iter().find(|attr| attr.path().is_ident("component")) else {
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
            for field in &data_struct.fields {
                let Some(ident) = &field.ident else {
                    break;
                };
                let kind = &field.ty;
                let attribute_html = format!("{ident}").replace('_', "-");

                let attrs = &field.attrs;

                if attrs.iter().any(|a| a.path().is_ident("optionnal")) {
                    opt_struct.extend(quote! {
                        #ident : #kind,
                    });

                    check_none.extend(quote! {
                        let #ident = match ctx.props().#ident.clone(){
                            None => None,
                            Some(s) => {
                                if s.is_empty(){ None }
                                else{ Some(s) }
                            }
                        };
                    });
                } else {
                    opt_struct.extend(quote! {
                        #ident : Option<#kind>,
                    });

                    check_none.extend(quote! {
                        let Some(#ident) = ctx.props().#ident.clone() else{
                            return html!();
                        };
                        if #ident.is_empty() {
                            return html!();
                        }
                    });
                }
                default_props.extend(quote! {
                    #ident : this.get_attribute(#attribute_html),
                });
                observed.extend(quote! {
                    #attribute_html,
                });
                get_attributes.extend(quote! {
                    #ident: this.get_attribute(#attribute_html),
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

                fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
                    true
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
                    name: String,
                    old_value: Option<String>,
                    new_value: Option<String>,
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
