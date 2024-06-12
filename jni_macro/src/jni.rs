use std::{convert::TryFrom, sync::Mutex};

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_quote, spanned::Spanned, Abi, Item, ItemFn, ItemMod, Lit, LitStr, Meta, Token, Visibility,
};

/// 环境变量中设置的 java限定名
static ENV_JPATH: Mutex<String> = Mutex::new(String::new());

/// 解析获得 java path
fn get_value(input: TokenStream) -> Option<String> {
    // println!("jni_impl: get java-class path");
    if let Some(token) = input.into_iter().next() {
        match litrs::StringLit::try_from(token) {
            Ok(s) => return Some(s.value().to_string()),
            Err(e) => panic!("get java-class path fail ! {}", e),
        }
    }
    None
}

/// 从环境变量获取 java path
fn get_env_jpath() -> Option<String> {
    // println!("get_env_jpath:");
    let mut env = ENV_JPATH.lock().unwrap();
    if !env.is_empty() {
        return Some(env.clone());
    }
    if let Ok(jpath) = std::env::var("JNI_JPATH") {
        // println!("cargo-env-var[JNI_JPATH]={jpath}");
        env.push_str(&jpath);
        Some(jpath)
    } else {
        None
    }
}

/// 函数处理器
struct FnProcessor {
    pub attr: TokenStream,
    /// syn库的函数体结构
    pub body: ItemFn,
}
impl FnProcessor {
    pub fn new(attr: TokenStream, body: ItemFn) -> Self {
        Self { attr, body }
    }
    /// 将函数体结构转换为语法树
    pub fn collect(self) -> TokenStream {
        self.body.to_token_stream().into()
    }

    /// 统一为函数添加 no_mangle 属性
    pub fn add_attributes(mut self) -> Self {
        // println!("add_attributes: quote: add [no_mangle]");
        self.body.attrs.push(parse_quote!(#[no_mangle]));
        self
    }

    /// 统一设置函数访问性
    /// - 公开访问
    /// - 标准 C ABI约束
    pub fn set_visibility(mut self) -> Self {
        let body = &mut self.body;

        // println!("set_visibility: set vis");
        if !matches!(body.vis, Visibility::Public(_)) {
            body.vis = Visibility::Public(Token![pub](body.vis.span()));
        }

        // println!("set_visibility: set abi");
        // 设置 C ABI约束
        let abi_span = body.sig.abi.span();
        body.sig.abi = Some(Abi {
            extern_token: Token![extern](abi_span),
            name: Some(LitStr::new("system", abi_span)),
        });
        self
    }

    /// 统一按 JNI标准修改函数命名
    /// ### Java_my_package_name_JniWrapClass_fname
    /// - 前缀 `Java_`
    /// - 包名 `my_package_name_`
    /// - 包装类 `JniWrapClass_`
    /// - 函数名 `fname`
    pub fn update_name(mut self) -> Self {
        let mut name = String::with_capacity(32);
        name.push_str("Java_");
        if let Some(value) = get_value(self.attr.clone()) {
            // println!("attr value: {value}");
            if !value.is_empty() {
                name.push_str(&value.replace('.', "_"));
                name.push('_');
            }
        }
        let id = &self.body.sig.ident;
        name.push_str(&id.to_string());
        // println!("set_fn_name: result {name}");
        // println!("set_fn_name: update");
        self.body.sig.ident = syn::Ident::new(&name, id.span());
        self
    }
}

/// 处理函数上的 jni属性
pub fn proc_fun(attr: TokenStream, body: ItemFn) -> TokenStream {
    // println!("----- fun {}", body.sig.ident.to_string());
    FnProcessor::new(attr, body)
        .add_attributes()
        .set_visibility()
        .update_name()
        .collect()
}

/// 检查Path是否指向本过程宏
fn is_jni_path(attr_path: &syn::Path) -> bool {
    let mut is_jni = false;
    let mut is_self = false;
    for pp in &attr_path.segments {
        let id = pp.ident.to_string();
        is_self = is_self || id == "jni_macro";
        is_jni = id == "jni";
        if is_jni && is_self {
            break;
        }
    }
    is_jni
}

/// 处理 mod上的 jni属性，并应用到内部函数（不递归）
/// 1. 解析过程宏属性值
/// 2. 遇到不设值的模块，尝试获取环境变量，仍无值则不处理次模块。
/// 3. 将属性值应用到所有设置了 jni属性的内部函数（不递归）
pub fn proc_mod(attr: TokenStream, mut mm: ItemMod) -> Option<TokenStream> {
    // 获取属性值。或尝试从环境变量获取。
    // println!("===== mod {} =====", body.ident.to_string());
    let mut prefix = get_value(attr).or_else(get_env_jpath)?;
    // 获取内部代码。空模块不处理
    let (_, ls) = mm.content.as_mut()?;
    // 迭代内部代码，找到函数，检查是否设置 jni属性，对其应用模组属性值
    for item in ls.iter_mut() {
        let Item::Fn(ff) = item else { continue };
        // 找到 jni属性（过程宏）
        for aa in ff.attrs.iter_mut() {
            match &aa.meta {
                // meta为 Path表示没设置值，此时直接套用本模块的 jni属性
                Meta::Path(p) => {
                    if is_jni_path(p) {
                        *aa = parse_quote! {
                            #[jni(#prefix)]
                        };
                        // println!("updated attr: {:#?}", aa);
                        break;
                    }
                }
                // meta为 List表示有设置值，此时将模组属性与函数属性拼接
                Meta::List(l) => {
                    if !l.path.is_ident("jni") {
                        continue;
                    }
                    let Ok(lit) = l.parse_args::<Lit>() else {
                        continue;
                    };
                    let Lit::Str(s) = lit else { continue };
                    // 拼接：mod.attr_fn.attr
                    prefix = format!("{}_{}", prefix, s.value());
                    *aa = parse_quote! {
                        #[jni(#prefix)]
                    };
                    break;
                }
                _ => {}
            }
        } // for (ff.attrs)
    } // while (iter.next)
    Some(TokenStream::from(quote! { #mm }))
}
