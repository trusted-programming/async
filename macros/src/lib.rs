//!
//! orion-async-macros提供orion-async的过程宏实现部分的功能
//!

#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::{ TokenStream };
use quote::{ quote, ToTokens };
use proc_macro2::{ Span };
use syn::{
    parse_macro_input,
    ItemFn, AttributeArgs,
    NestedMeta, Meta,
    Lit,
    spanned::{ Spanned },
};

struct Config {
    send: bool,
}

impl Config {
    const fn new() -> Self {
        Config {
            send: false,
        }
    }
    fn build(&mut self, arg: &NestedMeta) -> Result<(), syn::Error> {
        match arg {
            NestedMeta::Meta(Meta::NameValue(namevalue)) => {
                let ident = namevalue.path.get_ident();
                if ident.is_none() {
                    return Err(syn::Error::new_spanned(&namevalue, "should have specified ident"));
                }
                let ident = ident.unwrap().to_string().to_lowercase();
                match ident.as_str() {
                    "body_send" => {
                        self.send = parse_bool(namevalue.lit.clone(), Spanned::span(&namevalue.lit))?;
                    },
                    _ => {
                        let msg = "unknown attribute, expected is `body_send`";
                        return Err(syn::Error::new_spanned(namevalue, msg));
                    }
                }
            },
            _ => {
                let msg = "unknown attribute";
                return Err(syn::Error::new_spanned(arg, msg));
            }
        }
        Ok(())
    }
}

fn parse_bool(val: Lit, span: Span) -> Result<bool, syn::Error> {
    match val {
        Lit::Bool(b) => Ok(b.value),
        _ => Err(syn::Error::new(span, "value should be true or false")),
    }
}

///
/// 如下的实现的async函数内部使用了Rc，因为不支持Send，无法利用tokio::spawn调度
/// 
/// ```rust
/// async fn foo() -> i32 {
///     let id = Rc::new(0);
///     bar(*id).await;
/// }
/// ```
/// 如下定义即可保证Rc可在Future内部正常使用，同时保证安全和性能
/// ```rust
/// #[orion_async::future(body_send = true)]
/// async fn foo() -> i32 {
///     let id = Rc::new(0);
///     bar(*id).await;
/// }
/// ```
///

#[proc_macro_attribute]
pub fn future(args: TokenStream, item: TokenStream) -> TokenStream {
    let item_copy = item.clone();
    let args = parse_macro_input!(args as AttributeArgs);
    let mut input = parse_macro_input!(item as ItemFn);

    if input.sig.asyncness.is_none() {
        let msg = "should add `async` keyword";
        return syn::Error::new_spanned(input.sig.fn_token, msg).into_compile_error().into();
    }

    let mut conf = Config::new();

    for arg in args {
        if let Err(error) = conf.build(&arg) {
            return error.into_compile_error().into(); 
        }
    }

    if conf.send {
        send_future(&mut input)
    } else {
        item_copy
    }
}

fn send_future(input: &mut ItemFn) -> TokenStream {
    let body = input.block.to_token_stream();
    let tokens: TokenStream = quote!({
        let future = unsafe {
            orion_async::SendFuture::new(async { #body })
        };
        future.await
    }).into();

    let wrapper = parse_macro_input!(tokens as syn::Stmt);
    input.block.stmts.clear();
    input.block.stmts.push(wrapper);

    input.to_token_stream().into()
}
