#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro2;
use proc_macro2::TokenStream;
use proc_macro_hack::proc_macro_hack;
use quote::{quote, ToTokens};
use syn;

type Res<T> = Result<T, T>;

fn need_iterable(r: Res<syn::Expr>) -> syn::Expr {
    match r {
        Ok(x) => x,
        Err(e) => syn::parse2(quote! {{
            #e;
            None.into_iter()
        }})
        .unwrap(),
    }
}

/// Add one to an expression.
#[proc_macro_hack]
pub fn fake_yield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match fake_yield_impl(input.into()) {
        Ok(x) => x.into(),
        Err(e) => panic!("Unexpected code: {}", e),
    }
}

fn fake_yield_impl(input: TokenStream) -> Res<TokenStream> {
    if input.is_empty() {
        Ok(quote! { None.into_iter() })
    } else if let Ok(mut block) = syn::parse2(input.clone()) {
        handle_block(&mut block);
        Ok(block.into_token_stream())
    } else if let Ok(expr) = syn::parse2(input.clone()) {
        match handle_expr(expr) {
            Ok(x) => Ok(x.into_token_stream()),
            Err(x) => Err(x.into_token_stream()),
        }
    } else if let Ok(stmt) = syn::parse2(input.clone()) {
        match handle_stmt(stmt) {
            Ok(x) => Ok(x.into_token_stream()),
            Err(x) => Err(x.into_token_stream()),
        }
    } else {
        Err(input)
    }
}

fn handle_expr(expr: syn::Expr) -> Res<syn::Expr> {
    use syn::Expr::*;
    match expr {
        Macro(expr_macro) => {
            if expr_macro.mac.path.is_ident("_yield") {
                let tts = expr_macro.mac.tts;
                Ok(syn::parse2(quote!(Some(#tts).into_iter())).expect("Converting _yield failed"))
            } else if expr_macro.mac.path.is_ident("_yield_from") {
                let tts = expr_macro.mac.tts;
                Ok(syn::parse2(tts).expect("Converting _yield_from failed"))
            } else {
                Err(expr_macro.into())
            }
        }
        If(syn::ExprIf {
            if_token,
            cond,
            mut then_branch,
            else_branch,
            ..
        }) => {
            let mut rv = if_token.into_token_stream();
            cond.to_tokens(&mut rv);
            handle_block(&mut then_branch);

            (quote! {
                { ::itertools::Either::Left(#then_branch) }
            })
            .to_tokens(&mut rv);

            if let Some((else_token, else_branch)) = else_branch {
                else_token.to_tokens(&mut rv);
                let else_branch = need_iterable(handle_expr(*else_branch));
                (quote! {
                    { ::itertools::Either::Right(#else_branch) }
                })
                .to_tokens(&mut rv);
            } else {
                (quote! {
                    else {
                        ::itertools::Either::Right(None.into_iter())
                    }
                })
                .to_tokens(&mut rv);
            }

            Ok(syn::parse2(rv).expect("converting if failed"))
        }
        Match(mut expr_match) => {
            handle_match_arms(&mut expr_match.arms);
            Ok(expr_match.into())
        }
        Block(mut expr_block) => {
            handle_block(&mut expr_block.block);
            Ok(expr_block.into())
        }
        x => Err(x),
    }
}

fn handle_match_arms(arms: &mut [syn::Arm]) {
    if arms.len() <= 1 {
        if arms.len() == 1 {
            let new_body = need_iterable(handle_expr((*arms[0].body).clone()));
            arms[0].body = Box::new(new_body);
        }
        return;
    }

    let split = arms.len() / 2;
    handle_match_arms(&mut arms[..split]);
    handle_match_arms(&mut arms[split..]);

    for mut arm in &mut arms[..split] {
        let old_body = &arm.body;
        let new_body = quote! {
            ::itertools::Either::Right(#old_body)
        };
        arm.body = syn::parse2(new_body).unwrap();
    }

    for mut arm in &mut arms[split..] {
        let old_body = &arm.body;
        let new_body = quote! {
            ::itertools::Either::Left(#old_body)
        };
        arm.body = syn::parse2(new_body).unwrap();
    }
}

fn handle_block(block: &mut syn::Block) {
    let mut chain = quote! { None.into_iter() };
    for stmt in block.stmts.iter().cloned().rev() {
        match handle_stmt(stmt) {
            Ok(elem) => {
                chain = quote! {
                    ::fake_yield::CallbackIterator::Uncalled(move || {
                        #elem.chain(#chain)
                    })
                };
            }
            Err(other) => {
                chain = quote! {
                    ::fake_yield::CallbackIterator::Uncalled(move || {
                        #other
                        #chain
                    })
                };
            }
        }
    }

    block.stmts = vec![syn::Stmt::Expr(syn::parse2(chain.clone()).unwrap())];
}

fn handle_stmt(stmt: syn::Stmt) -> Res<syn::Stmt> {
    use syn::Stmt::*;
    match stmt {
        Semi(expr, semi) => match handle_expr(expr) {
            Ok(x) => Ok(Expr(x)),
            Err(e) => Err(Semi(e, semi)),
        },
        Expr(expr) => Ok(Expr(need_iterable(handle_expr(expr)))),
        x => Err(x),
    }
}
