use proc_macro2::{TokenStream, Span};
use devise::{Spanned, Result, ext::SpanDiagnosticExt};
use syn::{Token, parse_quote, parse_quote_spanned};
use syn::{TraitItemFn, TypeParamBound, ReturnType, Attribute};
use syn::punctuated::Punctuated;
use syn::parse::Parser;

fn _async_bound(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> Result<TokenStream> {
    let bounds = <Punctuated<TypeParamBound, Token![+]>>::parse_terminated.parse(args)?;
    if bounds.is_empty() {
        return Ok(input.into());
    }

    let mut func: TraitItemFn = syn::parse(input)?;
    let original: TraitItemFn = func.clone();
    if func.sig.asyncness.is_none() {
        let diag = Span::call_site()
            .error("attribute can only be applied to async fns")
            .span_help(func.sig.span(), "this fn declaration must be `async`");

        return Err(diag);
    }

    let doc: Attribute = parse_quote! {
        #[doc = concat!(
            "# Future Bounds",
            "\n",
            "**The `Future` generated by this `async fn` must be `", stringify!(#bounds), "`**."
        )]
    };

    func.sig.asyncness = None;
    func.sig.output = match func.sig.output {
        ReturnType::Type(arrow, ty) => parse_quote_spanned!(ty.span() =>
            #arrow impl ::core::future::Future<Output = #ty> + #bounds
        ),
        default@ReturnType::Default => parse_quote_spanned!(default.span() =>
            -> impl ::core::future::Future<Output = ()> + #bounds
        ),
    };

    Ok(quote! {
        #[cfg(all(not(doc), rust_analyzer))]
        #original

        #[cfg(all(doc, not(rust_analyzer)))]
        #doc
        #original

        #[cfg(not(any(doc, rust_analyzer)))]
        #func
    })
}

pub fn async_bound(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream
) -> TokenStream {
    _async_bound(args, input).unwrap_or_else(|d| d.emit_as_item_tokens())
}
