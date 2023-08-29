use proc_macro::TokenStream;
use quote::quote;
use std::sync::OnceLock;
use syn::{
  parse::{Parse, ParseStream},
  ItemFn,
};

static IS_SUPPORT_DOCKER: OnceLock<bool> = OnceLock::new();

fn is_support_docker() -> bool {
  IS_SUPPORT_DOCKER
    .get_or_init(|| {
      // Check if docker is installed and running
      std::process::Command::new("docker")
        .arg("ps")
        .status()
        .map_or(false, |status| status.success())
    })
    .clone()
}

struct Args {
  is_docker: bool,
}

impl Parse for Args {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    match input.parse::<syn::Ident>() {
      Ok(ident) => {
        return Ok(Self {
          is_docker: ident == "docker",
        })
      }
      Err(_) => return Ok(Self { is_docker: false }),
    }
  }
}

#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item_fn = syn::parse_macro_input!(item as ItemFn);
  let args = syn::parse_macro_input!(attr as Args);

  let test_name = item_fn.sig.ident;
  let output = item_fn.sig.output;
  let content = item_fn.block;

  let is_async = item_fn.sig.asyncness.is_some();

  // Check if docker is installed and running
  // This value will only change when `astro_run_test` is rebuilt
  let is_support_docker = is_support_docker();

  let ignore = if args.is_docker && !is_support_docker {
    quote! { #[ignore] }
  } else {
    quote! {}
  };

  let content = quote! {
    astro_run_logger::init_logger_with_level(log::Level::Trace);

    #content
  };

  if is_async {
    return quote! {
      #ignore
      #[tokio::test]
      async fn #test_name() #output {
        #content
      }
    }
    .into();
  }

  quote! {
    #ignore
    #[test]
    fn #test_name() #output {
      #content
    }
  }
  .into()
}
