use std::convert::TryFrom;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse, parse_quote, spanned::Spanned, Abi, Item, ItemFn, ItemMod, Lit, LitStr, Meta, Token,
    Visibility,
};

const JAVA: &str = "Java";
const IDENT_ATTR: &str = "jni";
const IDENT_CRATE: &str = "jni_macro";

/// 解析获得 java path
fn get_value(input: TokenStream) -> Option<String> {
    // println!("jni_impl: get java-class path");
    for token in input {
        match litrs::StringLit::try_from(token) {
            Ok(s) => return Some(s.value().to_string()),
            Err(e) => panic!("get java-class path fail ! {}", e),
        };
    }
    None
}

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
        // println!("add_attributes: quote: add [no_mangle]");
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

        // println!("set_visibility: set vis");
        body.vis = Visibility::Public(Token![pub](body.vis.span()));

        // println!("set_visibility: set abi");
        let abi_span = body.sig.abi.span();
        body.sig.abi = Some(Abi {
            extern_token: Token![extern](abi_span),
            name: Some(LitStr::new("system", abi_span)),
        });

        // println!("set_visibility: update");
        let body = TokenStream::from(quote! {
            #body
        });
        self.body = parse(body).unwrap();
        self
    }

    pub fn set_fn_name(mut self) -> Self {
        let mut ls = Vec::with_capacity(4);
        ls.push(JAVA.to_string());
        if let Some(value) = get_value(self.attr.clone()) {
            // println!("attr value: {value}");
            if !value.is_empty() {
                ls.push(value.replace(".", "_"))
            }
        }
        // println!("set_fn_name: collect {ls:?}");
        {
            let id = &mut self.body.sig.ident;
            ls.push(id.to_string());
            // println!("set_fn_name: update");
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

/// 处理函数上的 jni属性
pub fn proc_fun(attr: TokenStream, body: ItemFn) -> TokenStream {
    // println!("----- fun {}", body.sig.ident.to_string());
    ProcFn::new(attr, body)
        .add_attributes()
        .set_visibility()
        .set_fn_name()
        .collect()
}

fn is_jni_path(attr_path: &syn::Path) -> bool {
    let mut is_jni = false;
    for pp in &attr_path.segments {
        let id = pp.ident.to_string();
        is_jni = id == IDENT_ATTR;
        if is_jni || id != IDENT_CRATE {
            break;
        }
    }
    is_jni
}

/// 处理 mod上的 jni属性
/// - 解析属性值，遇到不设值的情况退出不处理
///
pub fn proc_mod(attr: TokenStream, body: ItemMod) -> Option<TokenStream> {
    // println!("===== mod {} =====", body.ident.to_string());
    let Some(mut prefix) = get_value(attr) else {
        return None;
    };
    let mut mm = body.clone();
    let Some((_, ls)) = &mut mm.content else {
        return None;
    };
    let mut iter = ls.iter_mut();
    while let Some(item) = iter.next() {
        let Item::Fn(ff) = item else { continue };
        for aa in ff.attrs.iter_mut() {
            match &aa.meta {
                Meta::Path(p) => {
                    if is_jni_path(&p) {
                        *aa = parse_quote! {
                            #[jni_macro::jni(#prefix)]
                        };
                        // println!("updated attr: {:#?}", aa);
                        break;
                    }
                }
                Meta::List(l) => {
                    if !l.path.is_ident("jni") {
                        continue;
                    }
                    let Ok(lit) = l.parse_args::<Lit>() else {
                        continue;
                    };
                    let Lit::Str(s) = lit else { continue };
                    prefix = format!("{}_{}", prefix, s.value());
                    *aa = parse_quote! {
                        #[jni_macro::jni(#prefix)]
                    };
                    break;
                }
                _ => {}
            }
        } // for (ff.attrs)
    } // while (iter.next)
    Some(TokenStream::from(quote! { #mm }))
}
