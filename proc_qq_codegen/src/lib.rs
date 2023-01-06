use crate::event_arg::*;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, FnArg, Meta, NestedMeta, Token};

mod event_arg;

/// debug = note expanded codes if env PROC_QQ_CODEGEN_DEBUG exists
macro_rules! emit {
    ($tokens:expr) => {{
        use devise::ext::SpanDiagnosticExt;
        let mut tokens = $tokens;
        if std::env::var_os("PROC_QQ_CODEGEN_DEBUG").is_some() {
            let debug_tokens = proc_macro2::Span::call_site()
                .note("emitting proc_qq_codegen debug output")
                .note(tokens.to_string())
                .emit_as_item_tokens();

            tokens.extend(debug_tokens);
        }
        tokens.into()
    }};
}

/// event proc
#[proc_macro_error]
#[proc_macro_attribute]
pub fn event(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(args as syn::AttributeArgs);
    let all = parse_args(attrs);
    // must append to async fn
    let method = parse_macro_input!(input as syn::ItemFn);
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // params check
    let params = &method.sig.inputs;
    let param = if params.len() == 1 {
        params.first().unwrap()
    } else if params.len() == 2 {
        let param = params.first().unwrap();
        match param {
            FnArg::Receiver(_) => (),
            FnArg::Typed(_) => abort!(&param.span(), "两个参数时第一个必须为self"),
        }
        params.iter().skip(1).next().unwrap()
    } else {
        abort!(&params.span(), "暂时不支持两个参数以上的参数");
    };
    let param = match param {
        FnArg::Receiver(_) => abort!(&param.span(), "不支持self"),
        FnArg::Typed(pt) => pt,
    };
    let param_pat = param.pat.as_ref();
    let param_ty = param.ty.as_ref();
    let param_ty = quote! {#param_ty};
    let tokens = match param_ty.to_string().as_str() {
        "& LoginEvent" => (
            quote! {::proc_qq::LoginEventProcess},
            quote! {::proc_qq::ModuleEventProcess::LoginEvent},
        ),
        "& GroupMessageEvent" => (
            quote! {::proc_qq::GroupMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMessage},
        ),
        "& FriendMessageEvent" => (
            quote! {::proc_qq::FriendMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendMessage},
        ),
        "& GroupTempMessageEvent" => (
            quote! {::proc_qq::GroupTempMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupTempMessage},
        ),
        "& JoinGroupRequestEvent" => (
            quote! {::proc_qq::JoinGroupRequestEventProcess},
            quote! {::proc_qq::ModuleEventProcess::JoinGroupRequest},
        ),
        "& NewFriendRequestEvent" => (
            quote! {::proc_qq::NewFriendRequestEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewFriendRequest},
        ),
        "& NewFriendEvent" => (
            quote! {::proc_qq::NewFriendEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewFriendEvent},
        ),
        "& FriendPokeEvent" => (
            quote! {::proc_qq::FriendPokeEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendPoke},
        ),
        "& DeleteFriendEvent" => (
            quote! {::proc_qq::DeleteFriendEventProcess},
            quote! {::proc_qq::ModuleEventProcess::DeleteFriend},
        ),
        "& GroupMuteEvent" => (
            quote! {::proc_qq::GroupMuteEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMute},
        ),
        "& GroupLeaveEvent" => (
            quote! {::proc_qq::GroupLeaveEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupLeave},
        ),
        "& GroupNameUpdateEvent" => (
            quote! {::proc_qq::GroupNameUpdateEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupNameUpdate},
        ),
        "& GroupMessageRecallEvent" => (
            quote! {::proc_qq::GroupMessageRecallEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMessageRecall},
        ),
        "& FriendMessageRecallEvent" => (
            quote! {::proc_qq::FriendMessageRecallEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendMessageRecall},
        ),
        "& MessageEvent" => (
            quote! {::proc_qq::MessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::Message},
        ),
        "& MSFOfflineEvent" => (
            quote! {::proc_qq::MSFOfflineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::MSFOffline},
        ),
        "& KickedOfflineEvent" => (
            quote! {::proc_qq::KickedOfflineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::KickedOffline},
        ),
        "& ConnectedAndOnlineEvent" => (
            quote! {::proc_qq::ConnectedAndOnlineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::ConnectedAndOnline},
        ),
        "& DisconnectedAndOfflineEvent" => (
            quote! {::proc_qq::DisconnectedAndOfflineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::DisconnectedAndOffline},
        ),
        "& GroupDisbandEvent" => (
            quote! {::proc_qq::GroupDisbandEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupDisband},
        ),
        "& MemberPermissionChangeEvent" => (
            quote! {::proc_qq::MemberPermissionChangeEventProcess},
            quote! {::proc_qq::ModuleEventProcess::MemberPermissionChange},
        ),
        "& NewMemberEventEvent" => (
            quote! {::proc_qq::NewMemberEventEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewMemberEvent},
        ),
        "& SelfInvitedEventEvent" => (
            quote! {::proc_qq::SelfInvitedEventEventProcess},
            quote! {::proc_qq::ModuleEventProcess::SelfInvitedEvent},
        ),
        t => abort!(
            param.span(),
            format!("未知的参数类型 {}, 请在文档中查看兼容的事件以及参数类型 https://github.com/niuhuan/rust_proc_qq", t),
        ),
    };
    let trait_name = tokens.0;
    let enum_name = tokens.1;
    // gen token stream
    let ident = &method.sig.ident;
    let ident_str = format!("{}", ident);
    let build_struct = quote! {
        #[allow(non_camel_case_types)]
        pub struct #ident {}
    };
    // gen trait
    let block = &method.block;
    let build_trait = if all.is_empty() {
        quote! {
            #[::proc_qq::re_exports::async_trait::async_trait]
            impl #trait_name for #ident {
                async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
            }
        }
    } else {
        match param_ty.to_string().as_str() {
            "& MessageEvent" => (),
            "& GroupMessageEvent" => (),
            "& FriendMessageEvent" => (),
            "& GroupTempMessageEvent" => (),
            _ => abort!(
                &method.sig.span(),
                "event 的参数只支持消息类型事件 (MessageEvent,*MessageEvent)"
            ),
        }
        let args_vec = args_to_token(all);
        quote! {
            #[::proc_qq::re_exports::async_trait::async_trait]
            impl #trait_name for #ident {
                async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> {
                    if !::proc_qq::match_event_args_all(#args_vec, #param_pat.into())? {
                        return Ok(false);
                    }
                    self.raw(#param_pat).await
                }
            }
            impl #ident {
                async fn raw(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
            }
        }
    };
    // gen into
    let build_into = quote! {
        impl Into<::proc_qq::ModuleEventHandler> for #ident {
            fn into(self) -> ::proc_qq::ModuleEventHandler {
                ::proc_qq::ModuleEventHandler{
                    name: #ident_str.into(),
                    process: #enum_name(Box::new(self)),
                }
            }
        }
    };
    // emit
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn result(_: TokenStream, input: TokenStream) -> TokenStream {
    // must append to async fn
    let method = parse_macro_input!(input as syn::ItemFn);
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // params check
    let params = &method.sig.inputs;
    if params.len() != 1 && params.len() != 2 {
        abort!(&method.sig.span(), "必须有且只能有1~2个参数");
    };
    if params.len() == 1 {
        let pm = params.first().unwrap();
        let pm = match pm {
            FnArg::Receiver(_) => abort!(&pm.span(), "不支持self"),
            FnArg::Typed(pt) => pt,
        };
        let pm_ty = pm.ty.as_ref();
        let pm_ty = quote! {#pm_ty};
        if !pm_ty.to_string().as_str().eq("& EventResult") {
            abort!(&pm.span(), "一个参数时只支持 &EventResult");
        }
        let pm_pat = pm.pat.as_ref();
        // gen token stream
        let ident = &method.sig.ident;
        let ident_str = format!("{}", ident);
        let block = &method.block;
        return emit!(quote! {
            #[allow(non_camel_case_types)]
            pub struct #ident {}
            #[::proc_qq::re_exports::async_trait::async_trait]
            impl ::proc_qq::OnlyResultHandler for #ident {
                async fn handle(&self, #pm_pat: #pm_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
            }
            impl Into<::proc_qq::EventResultHandler> for #ident {
                fn into(self) -> ::proc_qq::EventResultHandler {
                    ::proc_qq::EventResultHandler{
                        name: #ident_str.into(),
                        process: ::proc_qq::ResultProcess::OnlyResult(Box::new(self)),
                    }
                }
            }
        });
    }
    abort!(method.span(), "现在只支持一个参数");
}

struct ModuleParams {
    span: Span,
    expressions: Vec<String>,
}

impl Parse for ModuleParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let params = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(ModuleParams {
            span,
            expressions: params
                .into_iter()
                .map(|param| param.to_token_stream().to_string())
                .collect(),
        })
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn module(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as ModuleParams);
    if params.expressions.len() < 2 {
        abort!(params.span, "参数数量不足")
    }
    let id = syn::parse_str::<Expr>(&params.expressions[0]).expect("id 解析错误");
    let name = syn::parse_str::<Expr>(&params.expressions[1]).expect("name 解析错误");
    let mut handle_builder = String::new();
    for i in 2..params.expressions.len() {
        handle_builder.push_str(&format!("{} {{}}.into(),", params.expressions[i]));
    }
    let handle_invoker =
        syn::parse_str::<Expr>(&format!("vec![{handle_builder}]")).expect("handle invoker解析错误");
    TokenStream::from(quote! {
        ::proc_qq::Module {
            id: #id.to_owned(),
            name: #name.to_owned(),
            handles: #handle_invoker,
        }
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn event_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(args as syn::AttributeArgs);
    let method = parse_macro_input!(input as syn::ItemFn);
    let mut struct_names = vec![];
    for attr in attrs {
        let struct_name = match attr {
            NestedMeta::Meta(me) => match me {
                Meta::Path(p) => {
                    if p.segments.len() != 1 {
                        abort!(&p.span(), "格式为event_fn(ident1,ident2,...)");
                    }
                    p.segments.first().unwrap().ident.clone()
                }
                _ => abort!(&me.span(), "格式为event_fn(ident1,ident2,...)"),
            },
            NestedMeta::Lit(_) => abort!(&attr.span(), "格式为event_fn(ident1,ident2,...)"),
        };
        struct_names.push(struct_name);
    }
    let params = &method.sig.inputs;
    let mut params_token_stream = quote! {&self};
    for param in params {
        match param {
            FnArg::Receiver(_) => (),
            FnArg::Typed(pt) => {
                let param_pat = pt.pat.as_ref();
                let param_ty = pt.ty.as_ref();
                params_token_stream.append_all(quote! {, #param_pat: #param_ty })
            }
        }
    }
    let vis = method.vis;
    let asyncness = method.sig.asyncness;
    let method_name = method.sig.ident;
    let block = &method.block;
    let mut result = quote! {};
    for struct_name in struct_names {
        result.append_all(quote! {
            impl #struct_name {
                #vis #asyncness fn #method_name ( #params_token_stream ) #block
            }
        });
    }
    emit!(result)
}
