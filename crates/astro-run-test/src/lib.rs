use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let item_fn = syn::parse_macro_input!(item as ItemFn);

  let test_name = item_fn.sig.ident;
  let output = item_fn.sig.output;
  let content = item_fn.block;
  let is_async = item_fn.sig.asyncness.is_some();

  if is_async {
    return quote! {
      #[tokio::test]
      async fn #test_name() #output {
        astro_run_logger::init_logger();

        #content
      }
    }
    .into();
  }

  quote! {
    #[test]
    fn #test_name() #output {
      astro_run_logger::init_logger();

      #content
    }
  }
  .into()
}
