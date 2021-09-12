use std::{
  env,
  path::{Path, PathBuf},
};

use proc_macro2::TokenStream;
use prost::Message;
use quote::{format_ident, quote};

mod descriptor {
  include!(concat!(env!("OUT_DIR"), "/pbdb.descriptor.rs"));
}

#[proc_macro]
pub fn pbdb_impls(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
  process_fds(&read_descriptor(
    &PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
      .join("file_descriptor_set.bin"),
  ))
  .into()
}

fn read_descriptor(path: &Path) -> descriptor::FileDescriptorSet {
  let bytes = std::fs::read(path).unwrap();
  descriptor::FileDescriptorSet::decode(bytes.as_slice()).unwrap()
}

fn process_fds(fds: &descriptor::FileDescriptorSet) -> TokenStream {
  fds
    .file
    .iter()
    .map(|file| &file.message_type)
    .flatten()
    .map(|dp| process_dp(dp))
    .collect()
}

fn process_dp(dp: &descriptor::DescriptorProto) -> TokenStream {
  let id_fields: Vec<_> = dp
    .field
    .iter()
    .filter(|field| {
      field
        .options
        .as_ref()
        .map_or(false, |options| options.pbdb_id == Some(true))
    })
    .collect();
  if id_fields.len() > 1 {
    unimplemented!("Multiple id fields are not supported yet");
  }
  if let Some(id_field) = id_fields.first() {
    if id_field.r#type() != descriptor::field_descriptor_proto::Type::String {
      unimplemented!("Non-string id fields are not supported yet");
    }
    if id_field.label() == descriptor::field_descriptor_proto::Label::Repeated {
      unimplemented!("Repeated id fields are not supported yet");
    }
    let message_name = format_ident!("{}", dp.name());
    let id_field_name = format_ident!("{}", id_field.name());
    quote! {
      impl #message_name {
        pub fn id(id: &str) -> ::pbdb::Id<Self> {
          ::pbdb::Id::new(id.as_bytes().to_owned())
        }
      }

      impl ::pbdb::Collection for #message_name {
        const CF_NAME: &'static str = stringify!(#message_name);

        fn id(&self) -> ::pbdb::Id<Self> {
          ::pbdb::Id::new(self.#id_field_name.as_bytes().to_owned())
        }
      }
    }
  } else {
    TokenStream::new()
  }
}
