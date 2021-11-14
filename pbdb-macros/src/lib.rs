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
  let (globals, options): (Vec<_>, Vec<_>) = fds
    .file
    .iter()
    .map(|file| &file.message_type)
    .flatten()
    .filter_map(|dp| process_dp(dp))
    .unzip();
  quote! {
    pub fn open_db(
      path: &std::path::Path
    ) -> Result<::pbdb::DbGuard, pbdb::private::rocksdb::Error> {
      use ::pbdb::private::{DB, rocksdb};
      let mut opts = rocksdb::Options::default();
      opts.create_if_missing(true);
      opts.create_missing_column_families(true);
      let mut cfs = vec![];
      cfs.push(
        rocksdb::ColumnFamilyDescriptor::new(
          "__SingleRecord",
          rocksdb::Options::default()
        )
      );
      #(#options)*
      let db = rocksdb::DB::open_cf_descriptors(&opts, path, cfs)?;
      let mut write = DB.write();
      *write = Some(db);
      Ok(::pbdb::DbGuard{})
    }
    #(#globals)*
  }
}

fn process_dp(dp: &descriptor::DescriptorProto) -> Option<(TokenStream, TokenStream)> {
  generate_collection(dp).or_else(|| generate_single_record(dp))
}

fn generate_collection(dp: &descriptor::DescriptorProto) -> Option<(TokenStream, TokenStream)> {
  let id_fields: Vec<_> = dp
    .field
    .iter()
    .filter(|field| {
      field
        .options
        .as_ref()
        .map_or(false, |options| options.id == Some(true))
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
    Some((
      quote! {
        impl ::pbdb::Collection for #message_name {
          const CF_NAME: &'static str = stringify!(#message_name);

          fn get_id(&self) -> &str {
            self.#id_field_name.as_str()
          }
        }
      },
      quote! {
        cfs.push(
          rocksdb::ColumnFamilyDescriptor::new(
            stringify!(#message_name),
            rocksdb::Options::default()
          )
        );
      },
    ))
  } else {
    None
  }
}

fn generate_single_record(dp: &descriptor::DescriptorProto) -> Option<(TokenStream, TokenStream)> {
  if dp
    .options
    .as_ref()
    .map_or(false, |options| options.single_record == Some(true))
  {
    let message_name = format_ident!("{}", dp.name());
    Some((
      quote! {
        impl ::pbdb::SingleRecord for #message_name {
          const RECORD_ID: &'static str = stringify!(#message_name);
        }
      },
      quote! {},
    ))
  } else {
    None
  }
}
