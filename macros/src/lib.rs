extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote;
use quote::ToTokens;
use std::collections::HashSet;
use syn::fold;
use syn::fold::Fold;
use syn::parse_quote;
use syn::Expr;
use syn::Local;
use syn::Pat;
use syn::Result;
use syn::Stmt;
use syn::Type;
use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token,
};

struct Args {
    vars: HashSet<Ident>,
}
impl Args {
    fn should_print_expr(&self, e: &Expr) -> bool {
        match *e {
            Expr::Path(ref e) => {
                // variable shouldn't start wiht ::
                if e.path.leading_colon.is_some() {
                    false
                // should be a single variable like `x=8` not n::x=0
                } else if e.path.segments.len() != 1 {
                    false
                } else {
                    // get the first part
                    let first = e.path.segments.first().unwrap();
                    // check if the variable name is in the Args.vars hashset
                    self.vars.contains(&first.ident) && first.arguments.is_empty()
                }
            }
            _ => false,
        }
    }

    // used for checking if to print let i=0 etc or not
    fn should_print_pat(&self, p: &Pat) -> bool {
        match p {
            // check if variable name is present in set
            Pat::Ident(ref p) => self.vars.contains(&p.ident),
            _ => false,
        }
    }

    // manipulate tree to insert print statement
    fn assign_and_print(&mut self, left: Expr, op: &dyn ToTokens, right: Expr) -> Expr {
        // recurive call on right of the assigment statement
        let right = fold::fold_expr(self, right);
        // returning manipulated sub-tree
        parse_quote!({
            #left #op #right;
            println!(concat!(stringify!(#left), " = {:?}"), #left);
        })
    }

    // manipulating let statement
    fn let_and_print(&mut self, local: Local) -> Stmt {
        let Local { pat, init, .. } = local;
        let init = self.fold_expr(*init.unwrap().1);
        // get the variable name of assigned variable
        let ident = match pat {
            Pat::Ident(ref p) => &p.ident,
            _ => unreachable!(),
        };
        // new sub tree
        parse_quote! {
            let #pat = {
                #[allow(unused_mut)]
                let #pat = #init;
                println!(concat!(stringify!(#ident), " = {:?}"), #ident);
                #ident
            };
        }
    }
}
impl Fold for Args {
    fn fold_expr(&mut self, e: Expr) -> Expr {
        match e {
            // for changing assignment like a=5
            Expr::Assign(e) => {
                // check should print
                if self.should_print_expr(&e.left) {
                    self.assign_and_print(*e.left, &e.eq_token, *e.right)
                } else {
                    // continue with default travesal using default methods
                    Expr::Assign(fold::fold_expr_assign(self, e))
                }
            }
            // for changing assigment and operation like a+=1
            Expr::AssignOp(e) => {
                // check should print
                if self.should_print_expr(&e.left) {
                    self.assign_and_print(*e.left, &e.op, *e.right)
                } else {
                    // continue with default behaviour
                    Expr::AssignOp(fold::fold_expr_assign_op(self, e))
                }
            }
            // continue with default behaviour for rest of expressions
            _ => fold::fold_expr(self, e),
        }
    }

    // for let statements like let d=9
    fn fold_stmt(&mut self, s: Stmt) -> Stmt {
        match s {
            Stmt::Local(s) => {
                if s.init.is_some() && self.should_print_pat(&s.pat) {
                    self.let_and_print(s)
                } else {
                    Stmt::Local(fold::fold_local(self, s))
                }
            }
            _ => fold::fold_stmt(self, s),
        }
    }
}
impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        // parses a,b,c, or a,b,c where a,b and c are Indent
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            vars: vars.into_iter().collect(),
        })
    }
}
///A macro for ensuring
#[proc_macro_attribute]
pub fn restricted_route(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = syn::parse_macro_input!(input as syn::ItemFn);
    let inputextractio = input_fn.clone();
    let attrs = inputextractio.attrs;
    let vis = inputextractio.vis;
    let sig = inputextractio.sig;
    let stmts = inputextractio.block.stmts;

    // Find the Session input parameter, if it exists
    let session_param = sig.inputs.iter().find_map(|param| match param {
        syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => match ty.as_ref() {
            Type::Path(syn::TypePath { path, .. }) => {
                if let Some(ident) = path.segments.last() {
                    if ident.ident == "Session" {
                        return Some(pat.clone());
                    }
                }
                None
            }
            _ => None,
        },
        _ => None,
    });

    // If the Session input parameter was not found, return an error
    let session_name = match session_param {
        Some(pat) => match pat.as_ref() {
            syn::Pat::Ident(syn::PatIdent { ident, .. }) => ident.to_string(),
            _ => "Session".to_string(),
        },
        None => {
            return syn::Error::new_spanned(
                input_fn,
                "function must have an input parameter of type `Session`",
            )
            .to_compile_error()
            .into()
        }
    };
    // Find the Session input parameter, if it exists
    let appdata_param = sig.inputs.iter().find_map(|param| match param {
        syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => match ty.as_ref() {
            Type::Path(syn::TypePath { path, .. }) => {
                if let Some(ident) = path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &ident.arguments {
                        if let Some(arg) = args.args.first() {
                            if let syn::GenericArgument::Type(Type::Path(type_path)) = arg {
                                if let Some(ident) = type_path.path.segments.last() {
                                    if ident.ident == "AppData" {
                                        return Some(pat.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        },
        _ => None,
    });

    // If the Session input parameter was not found, return an error
    let appdata_name = match appdata_param {
        Some(pat) => match pat.as_ref() {
            syn::Pat::Ident(syn::PatIdent { ident, .. }) => ident.to_string(),
            _ => "data".to_string(),
        },
        None => {
            return syn::Error::new_spanned(
                input_fn,
                "function must have an input parameter of type `web::Data<AppData>`",
            )
            .to_compile_error()
            .into()
        }
    };
    // Build the output, possibly using quasi-quotation
    let appdata_name_tok: TokenStream2 = appdata_name.parse().unwrap();
    let appdata_name_tokens = appdata_name_tok.into_token_stream();
    let session_name_tok: TokenStream2 = session_name.parse().unwrap();
    let session_name_tokens = session_name_tok.into_token_stream();
    let expanded = quote::quote! {
        #(#attrs)* #vis #sig {

            let result =#session_name_tokens.get::<i32>("id");
            let sessionid = match result {
            Ok(maybesessionid) => match maybesessionid {
                Some(sid) => sid,
                None => return Ok(HttpResponse::InternalServerError().body("Sessionid was not fund".to_string())),
            },
            Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        };

        let _res = users::Entity::find()
            .join(
                sea_orm::JoinType::LeftJoin,
                users::Relation::TempSessions
                    .def()
                    .on_condition(move |_left, _right| {
                        sea_orm::sea_query::Expr::col(tempsessions::Column::UserId)
                            .is(sessionid)
                            .into_condition()
                    }),
            )
            .one(#appdata_name_tokens.get_db())
            .await;
        let __res = tempsessions::Entity::find().all(#appdata_name_tokens.get_db()).await;
        match __res {
            Ok(op) => {
                let len=op.len();
                for m in op {
                    println!("model: {:?}", m)
                }
                println!("that is all, total length was: {}",len);
            }
            Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
        };
            let user = match _res {
                Ok(op) => match op {
                    Some(us) => us,
                    None => return Ok(HttpResponse::Unauthorized().body("Hello".to_string())),
                },
                Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
                };
        #(#stmts)*
        }
    };
    // Hand the output tokens back to the compiler
    println!("{}", expanded);
    TokenStream::from(expanded)
}
