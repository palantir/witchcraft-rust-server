// Copyright 2022 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#![allow(clippy::needless_doctest_main)]
use proc_macro::TokenStream;
use quote::quote;
use structmeta::StructMeta;
use syn::{Error, Expr, ItemFn};

/// Marks the entrypoint of a Witchcraft server.
///
/// This macro should be applied to a function taking 3 arguments: the server's install config, a `Refreshable` of the
/// server's runtime config, and a mutable reference to the `Witchcraft` context object. It is a simple convenience
/// function that wraps the annotated function in one that passes it to the `witchcraft_server::init` function.
///
/// # Parameters
///
/// The attribute allows configuration of the Witchcraft builder via optional parameters:
///
/// * `conjure_runtime` - An expression which creates the `Arc<ConjureRuntime>` used for application endpoints.
///
/// # Examples
///
/// ```ignore
/// use conjure_error::Error;
/// use witchcraft_server::config::install::InstallConfig;
/// use witchcraft_server::config::runtime::RuntimeConfig;
/// use witchcraft_server::Witchcraft;
/// use refreshable::Refreshable;
///
/// #[witchcraft_server::main]
/// fn main(
///     install: InstallConfig,
///     runtime: Refreshable<RuntimeConfig, Error>,
///     wc: &mut Witchcraft,
/// ) -> Result<(), Error> {
///     // initialization code...
///     Ok(())
/// }
/// ```
///
/// Expands to:
///
/// ```ignore
/// use conjure_error::Error;
/// use witchcraft_server::config::install::InstallConfig;
/// use witchcraft_server::config::runtime::RuntimeConfig;
/// use witchcraft_server::Witchcraft;
/// use refreshable::Refreshable;
///
/// fn main() {
///     fn inner_main(
///         install: InstallConfig,
///         runtime: Refreshable<RuntimeConfig, Error>,
///         wc: &mut Witchcraft,
///     ) -> Result<(), Error> {
///         // initialization code...
///         Ok(())
///     }
///
///     witchcraft_server::Builder::new()
///         .init(inner_main)
/// }
/// ```
#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match syn::parse::<MainArgs>(args) {
        Ok(args) => args,
        Err(e) => return with_error(input, e),
    };

    let function = match syn::parse::<ItemFn>(input.clone()) {
        Ok(function) => function,
        Err(e) => return with_error(input, e),
    };
    let vis = &function.vis;
    let name = &function.sig.ident;

    let conjure_runtime_call = args.conjure_runtime.map(|e| quote!(.conjure_runtime(#e)));

    quote! {
        #vis fn #name() {
            #function

            witchcraft_server::Builder::new()
                #conjure_runtime_call
                .init(#name)
        }
    }
    .into()
}

#[derive(StructMeta)]
struct MainArgs {
    conjure_runtime: Option<Expr>,
}

fn with_error(mut tokens: TokenStream, error: Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
