extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

mod jni;

use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, Item};

/// 通过 jni属性设置函数命名格式
/// - 转换语法树
/// - 若是 mod则提取属性给内部函数修改属性
/// - 若是 fn则修改函数签名
/// ### JNI标准：Java_包名_包装类_函数名
/// - 前缀 `Java_`
/// - 包名 `my_package_name_`
/// - 包装类 `JniWrapClass_`
/// - 函数名 `fname`
#[proc_macro_attribute]
pub fn jni(attr: TokenStream, body: TokenStream) -> TokenStream {
    let ii = syn::parse::<syn::Item>(body.clone()).expect("转换 Item 失败");
    match ii {
        Item::Fn(ff) => jni::proc_fun(attr, ff),
        Item::Mod(mm) => match jni::proc_mod(attr, mm) {
            Some(ts) => ts,
            _ => body,
        }
        // 目前只处理模块和函数
        _ => quote_spanned! {
            ii.span() => compile_error!("过程宏 jni 仅作用于函数(fn)与模块(mod)")
        }
        .into(),
    }
}
