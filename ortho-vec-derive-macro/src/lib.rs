extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self, DeriveInput, Ident, __private::Span, parse_macro_input, punctuated::Punctuated,
    token::Plus, DataStruct, Fields::Named, Generics, Lifetime, LifetimeParam, TypeParamBound,
    WhereClause,
};

/// Returns [proc_macro2::TokenStream] (not [proc_macro::TokenStream]).
fn transform_named_fields_into_ts(
    data_struct: &DataStruct,
    transform_named_field_fn: &dyn Fn(&syn::Field) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match data_struct.fields {
        Named(ref fields) => {
            // Create iterator over named fields, holding generated props token streams.
            let props_ts_iter = fields
                .named
                .iter()
                .map(|named_field| transform_named_field_fn(named_field));

            // Unwrap iterator into a [proc_macro2::TokenStream].
            quote! {
              #(#props_ts_iter)*
            }
        }
        _ => quote! {},
    }
}

/// Returns [proc_macro2::TokenStream] (not [proc_macro::TokenStream]).
fn take_first_named_field_ts(data_struct: &DataStruct) -> proc_macro2::TokenStream {
    match data_struct.fields {
        Named(ref fields) => {
            // Take first prop ident
            let first_prop_ident = fields
                .named
                .first()
                .expect("Struct should have at least one field")
                .ident
                .as_ref()
                .expect("First field should have an indentifier");

            // convert first ident into a [proc_macro2::TokenStream].
            quote! {
              #first_prop_ident
            }
        }
        _ => quote! {},
    }
}

fn remove_trait_bounds_from_generics(generics: &Generics) -> Generics {
    let mut generics_no_trait_bounds = generics.clone();

    generics_no_trait_bounds.params.iter_mut().for_each(|p| {
        if let syn::GenericParam::Type(tp) = p {
            let mut new_bounds = Punctuated::<TypeParamBound, Plus>::new();
            tp.bounds
                .iter()
                .filter(|bound| !matches!(bound, TypeParamBound::Trait(_)))
                .for_each(|bound| new_bounds.push(bound.clone()));

            tp.bounds = new_bounds;
        }
    });

    generics_no_trait_bounds
}

fn add_lifetime_to_generics(generics: &Generics, lifetime: &Lifetime) -> Generics {
    let lifetime_generic_param = syn::GenericParam::Lifetime(LifetimeParam::new(lifetime.clone()));

    let mut generics_w_lifetime = generics.clone();
    generics_w_lifetime.params.push(lifetime_generic_param);

    generics_w_lifetime
}

fn build_ortho_struct(
    name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
    ortho_lifetime: &Lifetime,
) -> (Ident, proc_macro2::TokenStream) {
    let ortho_struct_name = Ident::new(
        &("Ortho".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let props_ts_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_ty = &named_field.ty;

        quote! {
          #field_ident: &#ortho_lifetime #field_ty,
        }
    });

    let ortho_generics = add_lifetime_to_generics(generics, ortho_lifetime);

    (
        ortho_struct_name.clone(),
        quote!(
        struct #ortho_struct_name #ortho_generics
        #where_clause
        {
            #props_ts_iter
        }),
    )
}

fn build_ortho_vec_struct(
    name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> (Ident, proc_macro2::TokenStream) {
    let ortho_vec_name = Ident::new(
        &("OrthoVec".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let vec_props_ts_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_ty = &named_field.ty;

        quote! {
            #field_ident: Vec<#field_ty>,
        }
    });

    let ortho_vec_struct_decl = quote!(
        struct #ortho_vec_name #generics
        #where_clause
        {
            #vec_props_ts_iter
        }
    );

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);
    let first_ident_name = take_first_named_field_ts(data_struct);

    let ortho_vec_len_impl = quote!(
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause
        {
            fn len(&self) -> usize {
                self.#first_ident_name.len()
            }
        }
    );

    let empty_vecs_props_ts_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            #field_ident: vec![],
        }
    });

    let push_p_into_v_props_ts_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
          v.#field_ident.push(p.#field_ident);
        }
    });

    let ortho_vec_from_vec_impl = quote!(
        impl #generics From<Vec<#name #generics_no_trait_bounds>> for #ortho_vec_name #generics_no_trait_bounds
        #where_clause
        {
            fn from(value: Vec<#name #generics_no_trait_bounds>) -> Self {
                let mut v = Self {
                    #empty_vecs_props_ts_iter
                };

                for p in value {
                    #push_p_into_v_props_ts_iter
                }

                v
            }
        }
    );

    let into_ortho_name = Ident::new(
        &("IntoOrtho".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let vec_into_ortho_impl = quote!(
        pub trait #into_ortho_name {
            type OrthoVec;
        
            fn into_ortho(self) -> Self::OrthoVec;
        }

        impl #generics #into_ortho_name for Vec<#name #generics_no_trait_bounds>
        #where_clause
        {
            type OrthoVec = #ortho_vec_name #generics_no_trait_bounds;

            fn into_ortho(self) -> Self::OrthoVec {
                self.into()
            }
        }
    );

    (
        ortho_vec_name,
        quote!(
            #ortho_vec_struct_decl

            #ortho_vec_len_impl

            #ortho_vec_from_vec_impl

            #vec_into_ortho_impl
        ),
    )
}

fn build_ortho_vec_iter_struct(
    name: &Ident,
    ortho_struct_name: &Ident,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
    ortho_lifetime: &Lifetime,
) -> (Ident, proc_macro2::TokenStream) {
    let ortho_vec_iter_name = Ident::new(
        &("OrthoVecIter".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let ortho_generics = add_lifetime_to_generics(generics, ortho_lifetime);
    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);
    let ortho_generics_no_trait_bounds = remove_trait_bounds_from_generics(&ortho_generics);

    let vec_iter_props_assign_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
          #field_ident: &self.v.#field_ident[self.index - 1],
        }
    });

    (
        ortho_vec_iter_name.clone(),
        quote!(
            struct #ortho_vec_iter_name #ortho_generics
            #where_clause
            {
                v: &'ortho #ortho_vec_name #generics_no_trait_bounds,
                index: usize,
            }

            impl #ortho_generics Iterator for #ortho_vec_iter_name #ortho_generics_no_trait_bounds
            #where_clause
            {
                type Item = #ortho_struct_name #ortho_generics_no_trait_bounds;

                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    if self.index >= self.v.len() {
                        None
                    } else {
                        self.index += 1;
                        Some(#ortho_struct_name {
                            #vec_iter_props_assign_iter
                        })
                    }
                }
            }

            impl #ortho_generics #ortho_vec_name #generics_no_trait_bounds
            #where_clause
            {
                fn iter(&'ortho self) -> #ortho_vec_iter_name #ortho_generics_no_trait_bounds {
                    #ortho_vec_iter_name {
                        v: &self,
                        index: 0
                    }
                }
            }
        ),
    )
}

#[proc_macro_derive(OrthoVec)]
pub fn ortho_vec(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: struct_name_ident,
        data,
        mut generics,
        ..
    }: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &struct_name_ident;

    let gen = if let syn::Data::Struct(data_struct) = data {
        let where_clause = generics.where_clause.take();

        let ortho_lifetime = Lifetime::new("'ortho", Span::call_site());

        let (ortho_struct_name, ortho_struct_ts) = build_ortho_struct(
            name,
            &data_struct,
            &generics,
            &where_clause,
            &ortho_lifetime,
        );

        let (ortho_vec_name, ortho_vec_ts) =
            build_ortho_vec_struct(name, &data_struct, &generics, &where_clause);

        let (_, ortho_vec_iter_ts) = build_ortho_vec_iter_struct(
            name,
            &ortho_struct_name,
            &ortho_vec_name,
            &data_struct,
            &generics,
            &where_clause,
            &ortho_lifetime,
        );

        quote! {
            #ortho_struct_ts

            #ortho_vec_ts

            #ortho_vec_iter_ts
        }
    } else {
        quote!()
    };

    gen.into()
}

// struct OrthoVec<T> {}