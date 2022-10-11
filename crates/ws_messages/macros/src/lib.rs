use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Field, Type};

#[proc_macro_derive(Message, attributes(message_id))]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let _ast = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {};

    return TokenStream::from(expanded);
}

#[proc_macro_derive(MessageStruct, attributes(aligned, packed, length, ascii))]
pub fn derive_message_struct(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let data_struct = if let syn::Data::Struct(s) = ast.data {
        s
    } else {
        // TODO: output a `compiler_error!()` instead?
        panic!("Deriving MessageStruct is only valid on a struct.");
    };

    let struct_name = &ast.ident;

    let field_idents = data_struct
        .fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<_>>();
    let field_reads = data_struct
        .fields
        .iter()
        .map(get_field_read)
        .collect::<Vec<_>>();
    let field_writes = data_struct
        .fields
        .iter()
        .map(get_field_write)
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl MessageStruct for #struct_name {
            fn unpack(
                reader_: &mut ws_bitpack::BitPackReader
            ) -> Result<Self, ws_bitpack::BitPackReaderError> {
                #(let #field_idents = #field_reads;)*
                Ok(Self {
                    #(#field_idents,)*
                })
            }

            fn pack(
                &self,
                writer_: &mut ws_bitpack::BitPackWriter
            ) -> Result<(), ws_bitpack::BitPackWriterError> {
                #(#field_writes;)*
                Ok(())
            }
        }
    };

    return TokenStream::from(expanded);
}

fn get_field_read(field: &Field) -> proc_macro2::TokenStream {
    let field_metadata = get_metadata_stream(field, FieldAccess::FromVar);

    match &field.ty {
        syn::Type::Path(_) => quote! {
            ws_messages::MessageValue::unpack(reader_, &#field_metadata)?
        },
        Type::Array(a) => {
            let len = &a.len;
            match *a.elem {
                syn::Type::Path(_) => {
                    quote! {{
                        let mut result = [Default::default(); #len];
                        for item in &mut result {
                            *item =  ws_messages::MessageValue::unpack(
                                reader_, &#field_metadata)?
                        }
                        result
                    }}
                }
                _ => {
                    let t = a.elem.to_token_stream().to_string();
                    let n = get_field_name(field);
                    let error = format!("Unsupported array element type: {t} for field: {n}");
                    quote!(compile_error!(#error))
                }
            }
        }
        _ => {
            let t = field.ty.to_token_stream().to_string();
            let n = get_field_name(field);
            let error = format!("Unsupported field type: {t} for field: {n}");
            quote!(compile_error!(#error))
        }
    }
}

fn get_field_write(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let field_metadata = get_metadata_stream(field, FieldAccess::FromSelf);

    match &field.ty {
        syn::Type::Path(_) => quote! {
            ws_messages::MessageValue::pack(&self.#ident, writer_, &#field_metadata)?
        },
        Type::Array(a) => match *a.elem {
            syn::Type::Path(_) => {
                quote! {
                    for item in &self.#ident {
                        ws_messages::MessageValue::unpack(
                            item, writer_, &#field_metadata)?;
                    }
                }
            }
            _ => {
                let t = a.elem.to_token_stream().to_string();
                let n = get_field_name(field);
                let error = format!("Unsupported array element type: {t} for field: {n}");
                quote!(compile_error!(#error))
            }
        },
        _ => {
            let t = field.ty.to_token_stream().to_string();
            let n = get_field_name(field);
            let error = format!("Unsupported field type: {t} for field: {n}");
            quote!(compile_error!(#error))
        }
    }
}

fn get_field_name(field: &Field) -> String {
    field
        .ident
        .as_ref()
        .map(|i| i.to_token_stream().to_string())
        .unwrap_or("?".to_string())
}

enum FieldAccess {
    FromVar,
    FromSelf,
}

fn get_metadata_stream(field: &Field, access: FieldAccess) -> proc_macro2::TokenStream {
    let packed_bits = field
        .attrs
        .iter()
        .find(|a| a.path.is_ident("packed"))
        .and_then(|attr| attr.parse_meta().ok())
        .and_then(|meta| {
            if let syn::Meta::List(list) = meta {
                if let Some(syn::NestedMeta::Lit(syn::Lit::Int(i))) = list.nested.first() {
                    let bits = i.to_token_stream();
                    Some(quote!(Some(#bits)))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| quote!(None));

    let length_ident = field
        .attrs
        .iter()
        .find(|a| a.path.is_ident("length"))
        .and_then(|attr| attr.parse_meta().ok())
        .and_then(|meta| {
            if let syn::Meta::List(list) = meta {
                if let Some(syn::NestedMeta::Meta(syn::Meta::Path(p))) = list.nested.first() {
                    p.get_ident().cloned()
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map_or_else(
            || quote!(None),
            |length| match access {
                FieldAccess::FromVar => quote!(Some(#length as usize)),
                FieldAccess::FromSelf => quote!(Some(self.#length as usize)),
            },
        );

    let is_ascii = field
        .attrs
        .iter()
        .find(|a| a.path.is_ident("ascii"))
        .is_some();

    quote! {
        ws_messages::MessageFieldMetadata {
            bits: #packed_bits,
            length: #length_ident,
            ascii: #is_ascii,
        }
    }
}

#[proc_macro_derive(MessageUnion)]
pub fn derive_message_union(_input: TokenStream) -> TokenStream {
    return TokenStream::new();
}
