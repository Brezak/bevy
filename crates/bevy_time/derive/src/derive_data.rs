use bevy_macro_utils::BevyManifest;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    spanned::Spanned, DataEnum, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Index,
    Path,
};

#[derive(Debug)]
pub struct TimeTrackingDerive<'a> {
    pub metadata: TimeTrackingMetadata<'a>,
    pub r#type: TimeTrackingType<'a>,
}

impl<'a> TimeTrackingDerive<'a> {
    pub fn from_input(input: &'a DeriveInput) -> syn::Result<Self> {
        let manifest = BevyManifest::default();
        let bevy_time_path = manifest.get_path("bevy_time");
        let bevy_ecs_path = manifest.get_path("bevy_ecs");
        let metadata = TimeTrackingMetadata {
            type_ident: &input.ident,
            generics: &input.generics,
            bevy_time_path,
            bevy_ecs_path,
        };

        let r#type = match &input.data {
            syn::Data::Struct(data_struct) => {
                TimeTrackingType::Struct(TimeTrackingStruct::parse(data_struct)?)
            }
            syn::Data::Enum(data_enum) => {
                TimeTrackingType::Enum(TimeTrackingEnum::parse(data_enum)?)
            }
            syn::Data::Union(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "can't derive `TimeTracking` on a union",
                ));
            }
        };

        Ok(TimeTrackingDerive { metadata, r#type })
    }
}

#[derive(Debug)]
pub struct TimeTrackingMetadata<'a> {
    pub type_ident: &'a Ident,
    pub generics: &'a Generics,
    pub bevy_time_path: Path,
    pub bevy_ecs_path: Path,
}

#[derive(Debug)]
pub enum TimeTrackingType<'a> {
    Struct(TimeTrackingStruct<'a>),
    TupleStruct(TimeTrackingStruct<'a>),
    Enum(TimeTrackingEnum<'a>),
}

#[derive(Debug)]
pub struct TimeTrackingStruct<'a> {
    pub struct_type: StructType,
    pub fields: Vec<ParsedField<'a>>,
}

pub type StructType = FieldsType;

impl<'a> TimeTrackingStruct<'a> {
    pub fn parse(data_struct: &'a DataStruct) -> syn::Result<Self> {
        let (fields, struct_type) = ParsedField::parse_fields(&data_struct.fields)?;
        Ok(Self {
            fields,
            struct_type,
        })
    }
}

#[derive(Debug)]
pub struct TimeTrackingEnum<'a> {
    pub variants: Vec<EnumVariant<'a>>,
}

impl<'a> TimeTrackingEnum<'a> {
    pub fn parse(data_enum: &'a DataEnum) -> syn::Result<Self> {
        let mut variants = Vec::new();

        for variant in &data_enum.variants {
            let ident = &variant.ident;

            let (fields, variant_type) = ParsedField::parse_fields(&variant.fields)?;

            let variant = EnumVariant {
                ident,
                fields,
                variant_type,
            };

            variants.push(variant);
        }

        Ok(TimeTrackingEnum { variants })
    }
}

#[derive(Debug)]
pub struct EnumVariant<'a> {
    pub variant_type: EnumVariantType,
    pub ident: &'a Ident,
    pub fields: Vec<ParsedField<'a>>,
}

pub type EnumVariantType = FieldsType;

#[derive(Debug)]
pub struct ParsedField<'a> {
    pub ignored: bool,
    /// For structs this will be offset into the structure. For enums this will be offset into their variant.
    ///
    /// This is required for things like tuple structs where the index of a filed is also it's accessor.
    pub index_in_structure: usize,
    pub raw: &'a Field,
}

pub const TIME_TRACKING_ATTRIBUTE: &str = "time_tracking";

mod kw {
    syn::custom_keyword!(ignore);
}

#[derive(Debug)]
pub enum FieldsType {
    Named,
    Unnamed,
    Unit,
}

impl<'a> ParsedField<'a> {
    pub fn parse_fields(fields: &'a Fields) -> syn::Result<(Vec<Self>, FieldsType)> {
        let mut parsed_fields = Vec::new();

        for (index_in_structure, field) in fields.iter().enumerate() {
            parsed_fields.push(ParsedField {
                ignored: Self::is_ignored(field)?,
                index_in_structure,
                raw: field,
            })
        }

        let fields_type = match fields {
            Fields::Named(_) => FieldsType::Named,
            Fields::Unnamed(_) => FieldsType::Unnamed,
            Fields::Unit => FieldsType::Unit,
        };

        Ok((parsed_fields, fields_type))
    }

    fn is_ignored(field: &Field) -> syn::Result<bool> {
        // TODO: Handle multiple errors
        for attribute in &field.attrs {
            match &attribute.meta {
                syn::Meta::Path(path) if path.is_ident(TIME_TRACKING_ATTRIBUTE) => {
                    return Err(syn::Error::new(
                        attribute.span(),
                        "`TimeTracking` doesn't support any path attributes",
                    ))
                }
                syn::Meta::NameValue(name_value)
                    if name_value.path.is_ident(TIME_TRACKING_ATTRIBUTE) =>
                {
                    return Err(syn::Error::new(
                        attribute.span(),
                        "`TimeTracking` doesn't support any name value attributes",
                    ))
                }
                syn::Meta::List(list) if list.path.is_ident(TIME_TRACKING_ATTRIBUTE) => {
                    syn::parse2::<kw::ignore>(list.tokens.clone())?;
                    return Ok(true);
                }
                _ => continue,
            }
        }

        Ok(false)
    }

    pub fn accessor(&self) -> TokenStream {
        if let Some(ident) = &self.raw.ident {
            ident.clone().into_token_stream()
        } else {
            Index::from(self.index_in_structure).into_token_stream()
        }
    }
}
