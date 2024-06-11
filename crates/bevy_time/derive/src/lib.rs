#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![allow(missing_docs)]

mod derive_data;

use std::{borrow::Cow, ops::Deref};

use derive_data::{
    EnumVariant, EnumVariantType, ParsedField, TimeTrackingDerive, TimeTrackingEnum, TimeTrackingMetadata, TimeTrackingStruct, TimeTrackingType
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident, Index, Type};

extern crate proc_macro;

#[proc_macro_derive(TimeTracking, attributes(time_tracking))]
pub fn derive_time_tracking(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(token_stream as DeriveInput);

    let time_tracking_derive = match TimeTrackingDerive::from_input(&input) {
        Ok(data) => data,
        Err(err) => return err.into_compile_error().into(),
    };

    match &time_tracking_derive.r#type {
        TimeTrackingType::Struct(data) => {
            generate_struct_impl(data, &time_tracking_derive.metadata, false)
        }
        TimeTrackingType::TupleStruct(data) => {
            generate_struct_impl(data, &time_tracking_derive.metadata, true)
        }
        TimeTrackingType::Enum(data) => generate_enum_impl(data, &time_tracking_derive.metadata),
    }
    .into()
}

fn generate_struct_impl(
    r#struct: &TimeTrackingStruct,
    metadata: &TimeTrackingMetadata,
    _is_tuple_struct: bool,
) -> TokenStream {
    /*
    To lower compile times, make systems smaller and to make the code simpler to reason about we deduplicate types.

    Now, we don't actually have access to type data at this point but we can take advantage of the fact that if
    2 type expressions are equal they must be referring to the same type (this only holds in struct and enum definitions).
    The disadvantage is that we're still going to have some type duplicati. Take for example:

    ```
    use module::SomeType;
    struct Hello {
        a: SomeType,
        b: module::SomeType,
    }
    ```

    Without the knowledge about the use statement above there's no way for you to determine if `a` and `b` have the same type
    so you're forced to assume that they're the same.

    The method chosen is simple but doesn't scale well in the worst case (every field has a unique type expression).
    We maintain 2 vectors. Vector 1 holds all the unique type expressions we have encountered. Vector 2 holds all the
    fields we care about + an index into the first vector that points to this fields type expression. To add a new field
    to vector 2:

    * Linearly scan vector 1 for a type expression equal to this fields type expression.
    * If found take store this field along with the index of the unique type expression.
    * If not found store this type expression at the end of vector 1 and store our field along with and index for the last
      element of vector 1

    This method will have the worst case time complexity of O(n^2). Now if you're looking to improve this code here's what you should know.
    Vector 1 is used to construct 3 things: a expression determining if a Time tracker needs updating in the `First` schedule, a expression
    that does the same for the `FixedUpdate` schedule and a tuple of resource required to update all the time trackers in this time tracker.
    Vector 2 is used too pull all the fields from the struct we're deriving and update them with the correct time resource. This is when
    the index comes into play as it's used to pull the correct resource from the tuple created from vector 1.
    */

    let mut unique_types = Vec::new();
    let not_ignored_fields = match_fields_to_types(r#struct.fields.iter(), Mutability::Mutable(&mut unique_types));

    let time_path = &metadata.bevy_time_path;
    let bevy_ecs_path = &metadata.bevy_ecs_path;

    let first_update_fields = not_ignored_fields.iter().map(|(index, field)| {
        let resource_ident = Ident::new(&format!("resource_{index}"), Span::call_site());
        let field_ident = field.raw.ident.as_ref().map(|field| field.clone().into_token_stream()).unwrap_or_else(|| {
            Index::from(field.index_in_structure).into_token_stream()
        });
        let field_type = unique_types[*index];

        quote!(<#field_type as #time_path::TimeTracking>::update_first(&mut self.#field_ident, &#resource_ident))
    });

    let fixed_update_fields = not_ignored_fields.iter().map(|(index, field)| {
        let resource_ident = Ident::new(&format!("resource_{index}"), Span::call_site());
        let field_ident = field.accessor();
        let field_type = unique_types[*index];

        quote!(<#field_type as #time_path::TimeTracking>::update_fixed(&mut self.#field_ident, &#resource_ident))
    });

    let time_res_fields = generate_resource_fields(&unique_types);

    let functions = quote!(
        fn update_first<'a: 'b, 'b>(&mut self, #time_res_fields: &'b <<Self as #time_path::TimeTracking>::TimeRes<'a> as #bevy_ecs_path::system::SystemParam>::Item<'_, '_>) {
            #(#first_update_fields);*
        }

        fn update_fixed<'a: 'b, 'b>(&mut self, #time_res_fields: &'b <<Self as #time_path::TimeTracking>::TimeResFixed<'a> as #bevy_ecs_path::system::SystemParam>::Item<'_, '_>) {
            #(#fixed_update_fields);*
        }
    );

    let bounds = generate_constants_and_assoc_types(&unique_types, metadata);

    let struct_ident = metadata.type_ident;
    let (impl_generics, ty_generics, where_clause) = metadata.generics.split_for_impl();
    quote!(impl #impl_generics #time_path::TimeTracking for #struct_ident #ty_generics #where_clause {
        #bounds

        #functions
    })
}

#[derive(Clone, Copy)]
enum UpdateType {
    Fixed,
    First,
}

fn generate_enum_impl(r#enum: &TimeTrackingEnum, metadata: &TimeTrackingMetadata) -> TokenStream {
    //For an explanation of what this vector does read the essay in the generate_struct_impl function
    let mut unique_types = Vec::new();

    for variant in &r#enum.variants {
        for field in variant.fields.iter().filter(|field| !field.ignored) {
            if unique_types
                .iter()
                .find(|ty| &&&field.raw.ty == ty)
                .is_none()
            {
                unique_types.push(&field.raw.ty)
            }
        }
    }

    let bevy_time = &metadata.bevy_time_path;
    let bevy_ecs_path = &metadata.bevy_ecs_path;
    
    let time_res_fields = generate_resource_fields(&unique_types);
    
    let first_update_matches = generate_enum_variants_matches(&r#enum.variants, &unique_types, metadata, UpdateType::First);
    let fixed_update_matches = generate_enum_variants_matches(&r#enum.variants, &unique_types, metadata, UpdateType::Fixed);
    
    let functions = quote!(
        fn update_first<'a, 'b: 'a>(&mut self, #time_res_fields: &'b <<Self as #bevy_time::TimeTracking>::TimeResFixed<'a> as #bevy_ecs_path::system::SystemParam>::Item<'_, '_>) {
            match self #first_update_matches
        }
        
        fn update_fixed<'a, 'b: 'a>(&mut self, #time_res_fields: &'b <<Self as #bevy_time::TimeTracking>::TimeResFixed<'a> as #bevy_ecs_path::system::SystemParam>::Item<'_, '_>) {
            match self #fixed_update_matches
        }
    );
    
    let enum_ident = metadata.type_ident;
    let bounds = generate_constants_and_assoc_types(&unique_types, metadata);
    let (impl_generics, ty_generics, where_clause) = metadata.generics.split_for_impl();
    quote!(impl #impl_generics #bevy_time::TimeTracking for #enum_ident #ty_generics #where_clause {
        #bounds

        #functions
    })
}

enum Mutability<'a, T> {
    ReadOnly(&'a T),
    Mutable(&'a mut T),
}

impl<'a, T> AsRef<T> for Mutability<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Mutability::ReadOnly(n) => *n,
            Mutability::Mutable(n) => &**n,
        }
    }
}

/// If unique types is mutable it will be extended if any new types are discovered
// TODO: Simplify lifetimes. A bunch of them could probably be implicit.
fn match_fields_to_types<'fields, 'fields_ref, 'type_ref>(
    fields: impl Iterator<Item = &'fields_ref ParsedField<'fields>>,
    mut unique_types: Mutability<Vec<&'type_ref Type>>,
) -> Vec<(usize, &'fields_ref ParsedField<'fields>)>
where
    'fields: 'fields_ref, // 'fields_ref is a borrow of 'fields it can't outlive it.
    'fields: 'type_ref, // 'type_ref borrows from 'fields it also can't outlive it.
 {
    let mut collected_fields = Vec::new();

    for field in fields {
        let type_index = unique_types
            .as_ref()
            .iter()
            .enumerate()
            .find_map(|(index, ty)| (&&field.raw.ty == ty).then_some(index));

        let type_index = type_index.unwrap_or_else(|| {
            if let Mutability::Mutable(unique_types) = &mut unique_types {
                unique_types.push(&field.raw.ty);
                unique_types.len() - 1
            } else {
                panic!("field type was not in unique_types vector")
            }
        });

        collected_fields.push((type_index, field));
    }

    collected_fields
}

fn generate_enum_variants_matches<'a>(variants: &[EnumVariant<'a>], types: &Vec<&Type>, metadata: &TimeTrackingMetadata, update_type: UpdateType) -> TokenStream {
    let enum_ident = &metadata.type_ident;
    let bevy_time = &metadata.bevy_time_path;

    let matches = variants.iter().map(|variant| {
        let variant_ident = variant.ident;
        let update_function_ident = match update_type {
            UpdateType::First => Ident::new("update_first", Span::call_site()),
            UpdateType::Fixed => Ident::new("update_fixed", Span::call_site()),
        };

        let fields = match_fields_to_types(variant.fields.iter(), Mutability::ReadOnly(types));

        let field_destructure = match variant.variant_type {
            EnumVariantType::Unit => quote!(),
            EnumVariantType::Unnamed => {
                let all_fields = variant.fields.iter().map(|field| {
                    let ident_str = format!("field_{}", field.index_in_structure);

                    Ident::new(&ident_str, Span::call_site())
                });

                quote!((#(#all_fields),*))
            },
            EnumVariantType::Named => {
                let field_destructure = variant.fields.iter().map(|field| {
                    let destructure_ident = format!("field_{}", field.index_in_structure);
                    let destructure_ident = Ident::new(&destructure_ident, Span::call_site());
                    
                    let field_ident = field.raw.ident.as_ref().expect("named fields should not contain a field without a ident");

                    // Use struct destructuring so we don't encounter name collisions
                    quote!(#field_ident: #destructure_ident)
                });

                quote!({#(#field_destructure),*})
            },
        };

        let field_update = fields.into_iter().map(|(type_index, field)| {
            let destructure_ident = Ident::new(&format!("field_{}", field.index_in_structure), Span::call_site());

            let ty = types[type_index];
            let resource_ident = Ident::new(&format!("resource_{type_index}"), Span::call_site());

            quote!(<#ty as #bevy_time::TimeTracking>::#update_function_ident(#destructure_ident, &#resource_ident);)
        });

        quote!(#enum_ident::#variant_ident #field_destructure => {
            #(#field_update)*
        })
    });

    quote!({
        #(#matches)*
    })
}

fn generate_constants_and_assoc_types(
    types: &[&Type],
    metadata: &TimeTrackingMetadata,
) -> TokenStream {
    let time_path = &metadata.bevy_time_path;

    let needs_fixed_update = types
        .iter()
        .map(|ty| quote!(<#ty as #time_path::TimeTracking>::NEEDS_FIXED_UPDATE));

    let needs_first_update = types
        .iter()
        .map(|ty| quote!(<#ty as #time_path::TimeTracking>::NEEDS_FIRST_UPDATE));

    let needs_fixed_update = if types.is_empty() {
        quote!(false)
    } else {
        quote!((#(#needs_fixed_update)||*))
    };
    let needs_first_update = if types.is_empty() {
        quote!(false)
    } else {
        quote!((#(#needs_first_update)||*))
    };

    let first_time_res_type = nested_tuples(
        &types,
        MAX_TUPLE_SIZE,
        |ty, _| quote!(<#ty as #time_path::TimeTracking>::TimeRes<'w>),
    );

    let fixed_time_res_type = nested_tuples(
        &types,
        MAX_TUPLE_SIZE,
        |ty, _| quote!(<#ty as #time_path::TimeTracking>::TimeResFixed<'w>),
    );

    quote! {
        const NEEDS_FIXED_UPDATE: bool = #needs_fixed_update;
        const NEEDS_FIRST_UPDATE: bool = #needs_first_update;
        type TimeRes<'w> = #first_time_res_type;
        type TimeResFixed<'w> = #fixed_time_res_type;
    }
}

fn generate_resource_fields(types: &[&Type]) -> TokenStream {
    // We destructure resource tuple in function arguments so we don't have to special handle indexing into nested tuples.
    let time_res_fields = nested_tuples(types, MAX_TUPLE_SIZE, |_, index| {
        let ident = Ident::new(&format!("resource_{index}"), Span::call_site());

        ident.into_token_stream()
    });

    time_res_fields
}

/// Bevy usually only implements traits for tuples up to size 16
const MAX_TUPLE_SIZE: usize = 16;

/// Runs the `token_extractor` on `elements` and arranges the resulting tokens into tuples of max length 16, nesting them if necessary.
///
/// Since rust doesn't have variadic generics we only implement SystemParam for up to 16 element tuples.
/// This limit can be sidesteped by nesting tuples which is exactly what this function does.
fn nested_tuples<T>(
    elements: &[T],
    max_tuple_size: usize,
    mut token_extractor: impl FnMut(&T, usize) -> TokenStream,
) -> TokenStream {
    assert!(max_tuple_size > 1);

    // Do chunks of size 15 since the 16th element of a tuple will be the contents of the next chunk
    let chunks: Vec<Vec<TokenStream>> = elements
        .chunks(max_tuple_size - 1)
        .enumerate()
        .map(|(chunk_index, chunk)| {
            chunk
                .iter()
                .enumerate()
                .map(|(chunk_offset, field)| {
                    token_extractor(field, chunk_index * (max_tuple_size - 1) + chunk_offset)
                })
                .collect()
        })
        .collect();

    let mut chunks_rev = chunks.iter().rev();

    let last_tuple = if let Some(last_chunk) = chunks_rev.next() {
        quote!((#(#last_chunk),*))
    } else {
        // There were no fields
        return quote!(());
    };

    // Repeatedly create a tuple of tuples by spreading our 15 element chunks into a tuple and appending the last
    // produced chunk to the end.
    chunks_rev.fold(
        last_tuple,
        |last_tuple, chunk| quote!((#(#chunk),*, #last_tuple)),
    )
}
