extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::sync::Mutex;
use syn::{parse, parse::Parser, parse_macro_input, ItemEnum, ItemStruct};

static COUNTER: Mutex<usize> = Mutex::new(0);

#[proc_macro_attribute]
pub fn component_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { pub entity: EntityWeak })
                .unwrap(),
        );
    }

    let name = &item_struct.ident;

    let mut locked = COUNTER.lock().unwrap();
    let index: usize = *locked;
    *locked = index + 1;

    return quote! {
        #item_struct

        impl Component for #name {
            const INDEX: usize = #index;

            fn get_entity_id(&self) -> Option<EntityId> {
                Some(self.entity.upgrade()?.get_id())
            }
        }
    }
    .into();
}
