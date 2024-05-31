extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod jni;

use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, Item};

#[proc_macro_attribute]
pub fn jni(attr: TokenStream, body: TokenStream) -> TokenStream {
    let ii = syn::parse::<syn::Item>(body.clone()).expect("转换 Item 失败");
    match ii {
        Item::Mod(mm) => jni::proc_mod(attr, mm),
        Item::Fn(ff) => jni::proc_fun(attr, ff),
        _ => quote_spanned! {
            ii.span() => compile_error!("过程宏 jni 仅作用于函数(fn)与模块(mod)")
        }
        .into(),
    }
}
