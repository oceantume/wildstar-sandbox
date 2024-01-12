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
    let field_bits = data_struct
        .fields
        .iter()
        .map(|field| get_field_bits(field, FieldAccess::AsField))
        .collect::<Vec<_>>();

    let expanded = quote! {
        impl MessageStruct for #ident {}

        impl ws_bitpack::ReadValue for #ident {
            fn read(reader_: &mut ws_bitpack::BitPackReader) -> ws_bitpack::BitPackResult<Self> {
                use ws_bitpack::*;
                #(let #field_idents = #field_reads;)*
                Ok(#ident {
                    #(#field_idents,)*
                })
            }
        }

        impl ws_bitpack::WriteValue for #ident {
            fn write(&self, writer_: &mut ws_bitpack::BitPackWriter) -> ws_bitpack::BitPackResult {
                use ws_bitpack::*;
                #(#field_writes;)*
                Ok(())
            }
            fn bits(&self) -> usize {
                let mut bits_: usize = 0;
                #(#field_bits;)*
                bits_
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
        impl ws_bitpack::UnionVariant for #ident {
            fn variant(&self) -> usize {
                match self {
                    #(#ident::#variant_idents { .. } => #variant_indices,)*
                }
            }
        }

        impl ws_bitpack::ReadUnionValue for #ident {
            fn read_union(
                reader_: &mut BitPackReader,
                variant_: usize,
            ) -> ws_bitpack::BitPackResult<Self> {
                use ws_bitpack::*;
                Ok(match variant_ {
                    #(#variant_indices => #variant_reads,)*
                    // TODO: use an error instead
                    _ => panic!("Invalid union variant {}", variant_)
                })
            }
        }

        impl ws_bitpack::WriteValue for #ident {
            fn write(
                &self,
                writer_: &mut BitPackWriter,
            ) -> ws_bitpack::BitPackResult {
                use ws_bitpack::*;
                Ok(match self {
                    #(#variant_writes,)*
                })
            }
            fn bits(&self) -> usize {
                0 // TODO
            }
        }

        /*
        impl ws_messages::writer::UnionVariant<#ident> for ws_messages::writer::MessageWriter<#ident> {
            fn variant(value: &#ident) -> usize {
                match value {
                    #(#ident::#variant_idents { .. } => #variant_indices,)*
                }
            }
        }
        */
    };

    TokenStream::from(expanded)
}

fn get_field_read(field: &Field) -> proc_macro2::TokenStream {
    let field_metadata = get_field_metadata(field, FieldAccess::AsVar);
    let align_expr = match get_field_aligned(field) {
        true => quote!(reader_.align()?),
        false => quote!(),
    };

    match &field.ty {
        syn::Type::Path(_) => {
            let read_expr = get_read_expr(&field_metadata);
            quote! {{ #align_expr; #read_expr }}
        }
        Type::Array(a) => {
            let len = &a.len;
            match *a.elem {
                syn::Type::Path(_) => {
                    let read_expr = get_read_expr(&field_metadata);
                    quote! {{
                        #align_expr;
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
        FieldMetadata::Simple => quote!(ws_bitpack::ReadValue::read(reader_)?),
        FieldMetadata::Packed { bits } => {
            quote!(ws_bitpack::ReadPackedValue::read_packed(reader_, #bits)?)
        }
        FieldMetadata::Array { length } => {
            quote!(ws_bitpack::ReadArrayValue::read_array(reader_, #length)?)
        }
        FieldMetadata::PackedArray { bits, length } => {
            quote!(ws_bitpack::ReadPackedArrayValue::read_packed_array(reader_, #length, #bits)?)
        }
        // todo: handle ascii?
        //FieldMetadata::Ascii => quote!(MessageReader::read_ascii(reader_)?),
        FieldMetadata::Ascii => quote!(ws_bitpack::ReadValue::read(reader_)?),
        FieldMetadata::Union { variant } => {
            // TODO: Verify this. Our trait for it is unfinished.
            quote!(ws_bitpack::ReadUnionValue::read_union(reader_, #variant)?)
        }
    }
}

fn get_field_write(field: &Field, access: FieldAccess) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let field_metadata = get_field_metadata(field, FieldAccess::AsField);
    let field_access = match access {
        FieldAccess::AsVar => quote!(#ident),
        FieldAccess::AsField => quote!(&self.#ident),
    };
    let align_expr = match get_field_aligned(field) {
        true => quote!(writer_.align()?),
        false => quote!(),
    };

    match &field.ty {
        syn::Type::Path(_) => {
            let write_expr = get_write_expr(&field_metadata, field_access);
            quote!({ #align_expr; #write_expr })
        }
        Type::Array(a) => match *a.elem {
            syn::Type::Path(_) => {
                let write_expr = get_write_expr(&field_metadata, quote!(item));
                quote! {
                    #align_expr;
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

fn get_field_bits(field: &Field, access: FieldAccess) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let field_metadata = get_field_metadata(field, FieldAccess::AsField);
    let field_access = match access {
        FieldAccess::AsVar => quote!(#ident),
        FieldAccess::AsField => quote!(&self.#ident),
    };
    let align_expr = match get_field_aligned(field) {
        true => quote!(bits_ += 8 - (bits_ % 8)),
        false => quote!(),
    };

    match &field.ty {
        syn::Type::Path(_) => {
            let write_expr = get_bits_expr(&field_metadata, field_access);
            quote!({ #align_expr; #write_expr; })
        }
        Type::Array(a) => match *a.elem {
            syn::Type::Path(_) => {
                let bits_expr = get_bits_expr(&field_metadata, quote!(item));
                quote! {
                    #align_expr;
                    for item in #field_access {
                        #bits_expr;
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
        FieldMetadata::Simple => quote!(writer_.write(#value)?),
        FieldMetadata::Packed { bits } => quote!(writer_.write_packed(#value, #bits)?),
        FieldMetadata::Array { .. } => quote!(writer_.write_array(#value)?),
        FieldMetadata::PackedArray { bits, .. } => {
            quote!(writer_.write_packed_array(#value, #bits)?)
        }
        FieldMetadata::Ascii => quote!(MessageWriter::write_ascii(writer_, #value)?),
        FieldMetadata::Union { .. } => quote!(writer_.write(#value)?),
    }
}

fn get_bits_expr(
    field_metadata: &FieldMetadata,
    value: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match field_metadata {
        FieldMetadata::Simple => quote!(bits_ += ws_bitpack::WriteValue::bits(#value)),
        FieldMetadata::Packed { bits } => {
            quote!(bits_ += ws_bitpack::WritePackedValue::bits_packed(#value, #bits))
        }
        FieldMetadata::Array { .. } => {
            quote!(bits_ += ws_bitpack::WriteArrayValue::bits_array(#value))
        }
        FieldMetadata::PackedArray { bits, .. } => {
            quote!(bits_ += ws_bitpack::WritePackedArrayValue::bits_packed_array(#value, #bits))
        }
        FieldMetadata::Ascii => todo!(),
        FieldMetadata::Union { .. } => quote!(bits_ += ws_bitpack::WriteValue::bits(#value)),
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
    Array {
        length: proc_macro2::TokenStream,
    },
    PackedArray {
        bits: usize,
        length: proc_macro2::TokenStream,
    },
    Union {
        variant: proc_macro2::TokenStream,
    },
    Ascii,
}

fn get_field_aligned(field: &Field) -> bool {
    field.attrs.iter().any(|a| a.path.is_ident("aligned"))
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
            FieldAccess::AsField => quote!(self.#length as usize),
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
            FieldAccess::AsField => quote!(self.#variant as usize),
        });

    let is_ascii = field.attrs.iter().any(|a| a.path.is_ident("ascii"));

    match (packed_bits, length_expr, variant_expr, is_ascii) {
        (None, None, None, false) => FieldMetadata::Simple,
        (Some(bits), None, None, false) => FieldMetadata::Packed { bits },
        (None, Some(length), None, false) => FieldMetadata::Array { length },
        (Some(bits), Some(length), None, false) => FieldMetadata::PackedArray { bits, length },
        (None, None, Some(variant), false) => FieldMetadata::Union { variant },
        (None, None, None, true) => FieldMetadata::Ascii,
        _ => panic!("invalid attributes combination"),
    }
}
