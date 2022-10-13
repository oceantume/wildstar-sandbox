use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Field, Type};

#[proc_macro_derive(Message, attributes(message_id))]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let _ast = parse_macro_input!(input as DeriveInput);

    let expanded = quote! {};

    TokenStream::from(expanded)
}

#[proc_macro_derive(MessageStruct, attributes(aligned, packed, length, variant, ascii))]
pub fn derive_message_struct(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let data_struct = if let syn::Data::Struct(s) = ast.data {
        s
    } else {
        return TokenStream::from(quote!(compile_error!(
            "Deriving MessageStruct is only valid on a struct."
        )));
    };

    let ident = &ast.ident;
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
        .map(|field| get_field_write(field, FieldAccess::AsField))
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl MessageStruct for #ident {}

        impl ws_messages::reader::SimpleReader<#ident> for ws_messages::reader::MessageReader<#ident> {
            fn read(reader_: &mut BitPackReader) -> Result<#ident, BitPackReaderError> {
                use ws_messages::reader::*;
                #(let #field_idents = #field_reads;)*
                Ok(#ident {
                    #(#field_idents,)*
                })
            }
        }

        impl ws_messages::writer::SimpleWriter<#ident> for ws_messages::writer::MessageWriter<#ident> {
            fn write(writer_: &mut BitPackWriter, value_: &#ident) -> Result<(), BitPackWriterError> {
                use ws_messages::writer::*;
                #(#field_writes;)*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(MessageUnion)]
pub fn derive_message_union(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let data_enum = match ast.data {
        syn::Data::Enum(e) => e,
        _ => {
            return TokenStream::from(quote!(compile_error!(
                "Deriving MessageStruct is only valid on a struct."
            )))
        }
    };

    let ident = &ast.ident;
    let variant_indices = data_enum
        .variants
        .iter()
        .enumerate()
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let variant_idents = data_enum
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .collect::<Vec<_>>();
    let variants_with_fields = data_enum
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Named(fields) => {
                let fields = fields.named.iter().collect::<Vec<_>>();
                (variant, fields)
            }
            _ => panic!("Only named fields are supported for unions."),
        });
    let variant_reads = variants_with_fields
        .clone()
        .map(|(variant, fields)| {
            let variant_ident = &variant.ident;
            let field_idents = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();
            let field_reads = fields
                .iter()
                .map(|field| get_field_read(*field))
                .collect::<Vec<_>>();
            quote! {{
                #(let #field_idents = #field_reads;)*
                #ident::#variant_ident {
                    #(#field_idents,)*
                }
            }}
        })
        .collect::<Vec<_>>();
    let variant_writes = variants_with_fields
        .map(|(variant, fields)| {
            let variant_ident = &variant.ident;
            let field_idents = fields
                .iter()
                .map(|field| {
                    field
                        .ident
                        .as_ref()
                        .expect("Unexpected field with no ident")
                })
                .collect::<Vec<_>>();
            let field_writes = fields
                .iter()
                .map(|field| get_field_write(*field, FieldAccess::AsVar))
                .collect::<Vec<_>>();
            quote! {
                #ident::#variant_ident { #(#field_idents,)* } => {
                    #(#field_writes;)*
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl MessageUnion for #ident {}

        impl ws_messages::reader::UnionReader<#ident> for ws_messages::reader::MessageReader<#ident> {
            fn read_union(
                reader_: &mut BitPackReader,
                variant_: usize,
            ) -> Result<#ident, BitPackReaderError> {
                use ws_messages::reader::*;
                Ok(match variant_ {
                    #(#variant_indices => #variant_reads,)*
                    // TODO: use an error instead
                    _ => panic!("Invalid union variant {}", variant_)
                })
            }
        }

        impl ws_messages::writer::SimpleWriter<#ident> for ws_messages::writer::MessageWriter<#ident> {
            fn write(
                writer_: &mut BitPackWriter,
                value_: &#ident,
            ) -> Result<(), BitPackWriterError> {
                use ws_messages::writer::*;
                Ok(match value_ {
                    #(#variant_writes,)*
                })
            }
        }

        impl ws_messages::writer::UnionVariant<#ident> for ws_messages::writer::MessageWriter<#ident> {
            fn variant(value: &#ident) -> usize {
                match value {
                    #(#ident::#variant_idents { .. } => #variant_indices,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_field_read(field: &Field) -> proc_macro2::TokenStream {
    let field_metadata = get_field_metadata(field, FieldAccess::AsVar);

    match &field.ty {
        syn::Type::Path(_) => get_read_expr(&field_metadata),
        Type::Array(a) => {
            let len = &a.len;
            match *a.elem {
                syn::Type::Path(_) => {
                    let read_expr = get_read_expr(&field_metadata);
                    quote! {{
                        let mut result = [Default::default(); #len];
                        for item in &mut result {
                            *item = #read_expr
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

fn get_read_expr(field_metadata: &FieldMetadata) -> proc_macro2::TokenStream {
    match field_metadata {
        FieldMetadata::Simple => quote!(MessageReader::read(reader_)?),
        FieldMetadata::Packed { bits } => quote!(MessageReader::read_packed(reader_, #bits)?),
        FieldMetadata::List { length } => quote!(MessageReader::read_list(reader_, #length)?),
        FieldMetadata::PackedList { bits, length } => {
            quote!(MessageReader::read_packed_list(reader_, #length, #bits)?)
        }
        FieldMetadata::Ascii => quote!(MessageReader::read_ascii(reader_)?),
        FieldMetadata::Union { variant } => quote!(MessageReader::read_union(reader_, #variant)?),
    }
}

fn get_field_write(field: &Field, access: FieldAccess) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let field_metadata = get_field_metadata(field, FieldAccess::AsField);
    let field_access = match access {
        FieldAccess::AsVar => quote!(#ident),
        FieldAccess::AsField => quote!(&value_.#ident),
    };

    match &field.ty {
        syn::Type::Path(_) => get_write_expr(&field_metadata, field_access),
        Type::Array(a) => match *a.elem {
            syn::Type::Path(_) => {
                let write_expr = get_write_expr(&field_metadata, quote!(item));
                quote! {
                    for item in #field_access {
                        #write_expr
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

fn get_write_expr(
    field_metadata: &FieldMetadata,
    value: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match field_metadata {
        FieldMetadata::Simple => quote!(MessageWriter::write(writer_, #value)?),
        FieldMetadata::Packed { bits } => {
            quote!(MessageWriter::write_packed(writer_, #value, #bits)?)
        }
        FieldMetadata::List { .. } => quote!(MessageWriter::write(writer_,  #value)?),
        FieldMetadata::PackedList { bits, .. } => {
            quote!(MessageWriter::write_packed(writer_, #value, #bits)?)
        }
        FieldMetadata::Ascii => quote!(MessageWriter::write_ascii(writer_, #value)?),
        FieldMetadata::Union { .. } => quote!(MessageWriter::write(writer_, #value)?),
    }
}

fn get_field_name(field: &Field) -> String {
    field
        .ident
        .as_ref()
        .map(|i| i.to_token_stream().to_string())
        .unwrap_or_else(|| "?".to_string())
}

/// Indicates how the fields should be accessed.
enum FieldAccess {
    /// Access as a variable with the same ident as the field itself.
    AsVar,
    /// Access as a field of a local variable named `value_`.
    AsField,
}

/// Extra field metadata generated from attributes.
enum FieldMetadata {
    Simple,
    Packed {
        bits: usize,
    },
    List {
        length: proc_macro2::TokenStream,
    },
    PackedList {
        bits: usize,
        length: proc_macro2::TokenStream,
    },
    Ascii,
    Union {
        variant: proc_macro2::TokenStream,
    },
}

fn get_field_metadata(field: &Field, access: FieldAccess) -> FieldMetadata {
    let packed_bits = field
        .attrs
        .iter()
        .find(|a| a.path.is_ident("packed"))
        .and_then(|attr| attr.parse_meta().ok())
        .and_then(|meta| {
            if let syn::Meta::List(list) = meta {
                if let Some(syn::NestedMeta::Lit(syn::Lit::Int(i))) = list.nested.first() {
                    let bits = i.base10_parse().expect("Invalid number of bits");
                    Some(bits)
                } else {
                    None
                }
            } else {
                None
            }
        });

    let length_expr = field
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
        .map(|length| match access {
            FieldAccess::AsVar => quote!(#length as usize),
            FieldAccess::AsField => quote!(value_.#length as usize),
        });

    let variant_expr = field
        .attrs
        .iter()
        .find(|a| a.path.is_ident("variant"))
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
        .map(|variant| match access {
            FieldAccess::AsVar => quote!(#variant as usize),
            FieldAccess::AsField => quote!(value_.#variant as usize),
        });

    let is_ascii = field.attrs.iter().any(|a| a.path.is_ident("ascii"));

    if let Some(length) = length_expr {
        if let Some(bits) = packed_bits {
            FieldMetadata::PackedList { bits, length }
        } else {
            FieldMetadata::List { length }
        }
    } else if let Some(bits) = packed_bits {
        FieldMetadata::Packed { bits }
    } else if let Some(variant) = variant_expr {
        FieldMetadata::Union { variant }
    } else if is_ascii {
        FieldMetadata::Ascii
    } else {
        // NOTE: Invalid combinations currently return this. We could look into that at some point.
        FieldMetadata::Simple
    }
}
