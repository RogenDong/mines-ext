// #![allow(unused)]
use std::{convert::TryFrom, sync::Mutex};

use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse, parse_macro_input, spanned::Spanned, Abi, Attribute, Item, ItemFn, ItemMod, LitStr,
    Token, Visibility,
};

const TEMPLATE: &str = "Java_[package]_<class>_name";
const JAVA: &str = "Java";

static J_PKG: Mutex<Option<String>> = Mutex::new(None);

pub struct ProcFn {
    pub attr: TokenStream,
    pub body: ItemFn,
}
impl ProcFn {
    pub fn new(attr: TokenStream, body: ItemFn) -> Self {
        ProcFn { attr, body }
    }
    pub fn collect(self) -> TokenStream {
        self.body.to_token_stream().into()
    }

    pub fn add_attributes(mut self) -> Self {
        println!("add_attributes: quote: add [no_mangle]");
        let body = self.body;
        let body = TokenStream::from(quote! {
            #[no_mangle]
            #body
        });
        self.body = parse(body).unwrap();
        self
    }

    pub fn set_visibility(mut self) -> Self {
        let body = &mut self.body;

        println!("set_visibility: set vis");
        body.vis = Visibility::Public(Token![pub](body.vis.span()));

        println!("set_visibility: set abi");
        let abi_span = body.sig.abi.span();
        body.sig.abi = Some(Abi {
            extern_token: Token![extern](abi_span),
            name: Some(LitStr::new("system", abi_span)),
        });

        println!("set_visibility: update");
        let body = TokenStream::from(quote! {
            #body
        });
        self.body = parse(body).unwrap();
        self
    }

    pub fn set_fn_name(mut self) -> Self {
        let mut ls = Vec::with_capacity(4);
        ls.push(JAVA.to_string());
        // java package
        if let Ok(p) = J_PKG.lock() {
            if let Some(p) = p.as_ref() {
                ls.push(p.clone());
            }
        }
        // java class
        let tmp = get_java_path(self.attr.clone());
        if !tmp.is_empty() {
            ls.push(tmp)
        }
        println!("set_fn_name: collect {ls:?}");
        if ls.len() < 2 {
            panic!("{}", TEMPLATE)
        }

        {
            let id = &mut self.body.sig.ident;
            ls.push(id.to_string());
            println!("set_fn_name: update");
            *id = syn::Ident::new(&ls.join("_"), id.span());
        }
        let body = &self.body;
        let body = TokenStream::from(quote! {
            #body
        });
        self.body = parse(body).unwrap();
        self
    }
}

/// 解析获得 java path
pub fn get_java_path(input: TokenStream) -> String {
    println!("jni_impl: get java-class path");
    for token in input {
        let path = match litrs::StringLit::try_from(token) {
            Err(e) => panic!("get java-class path fail ! {}", e),
            Ok(s) => s.value().to_string(),
        };
        return path.replace(".", "_");
    }
    String::new()
}

/// 尝试匹配mod属性
pub fn get_jpath_from_mod(attr: &TokenStream) {
    let jpath = get_java_path(attr.clone());
    *J_PKG.lock().unwrap() = if jpath.is_empty() {
        println!("clear j-package");
        None
    } else {
        println!("now j-package: {jpath}");
        Some(jpath)
    };
}
pub fn proc_mod(attr: TokenStream, body: ItemMod) -> TokenStream {
    println!("===== mod {} =====", body.ident.to_string());
    get_jpath_from_mod(&attr);
    let mut im = body.clone();
    if let Some((_, lsi)) = &mut im.content {
        let mut iter = lsi.iter_mut();
        while let Some(item) = iter.next() {
            if let Item::Fn(ff) = item {
                // let body = TokenStream::from(quote! {
                //     #[dong]
                //     #ff
                // });
                // if let Ok(tt) = parse(body) {
                //     *ff = tt;
                // }
            }
        }
    }
    TokenStream::from(quote! { #im })
}

pub fn proc_fun(attr: TokenStream, body: ItemFn) -> TokenStream {
    println!("----- fun {}", body.sig.ident.to_string());
    let mut proc = ProcFn::new(attr, body);
    proc.add_attributes()
        .set_visibility()
        .set_fn_name()
        .collect()
}
