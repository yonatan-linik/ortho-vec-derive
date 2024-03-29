extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self, DeriveInput, Ident, __private::Span, parse_macro_input, punctuated::Punctuated,
    token::Plus, DataStruct, Fields::Named, Generics, Lifetime, LifetimeParam, TypeParamBound,
    WhereClause,
};

/// Returns [`proc_macro2::TokenStream`] (not [`proc_macro::TokenStream`]).
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

/// Returns [`proc_macro2::TokenStream`] (not [`proc_macro::TokenStream`]).
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
          pub(super) #field_ident: &#ortho_lifetime #field_ty,
        }
    });

    let ortho_generics = add_lifetime_to_generics(generics, ortho_lifetime);

    (
        ortho_struct_name.clone(),
        quote!(
        pub(super) struct #ortho_struct_name #ortho_generics
        #where_clause
        {
            #props_ts_iter
        }),
    )
}

fn build_ortho_struct_mut(
    name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
    ortho_lifetime: &Lifetime,
) -> (Ident, proc_macro2::TokenStream) {
    let ortho_struct_mut_name = Ident::new(
        &("OrthoMut".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let props_ts_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_ty = &named_field.ty;

        quote! {
          pub(super) #field_ident: &#ortho_lifetime mut #field_ty,
        }
    });

    let ortho_generics = add_lifetime_to_generics(generics, ortho_lifetime);

    (
        ortho_struct_mut_name.clone(),
        quote!(
        pub(super) struct #ortho_struct_mut_name #ortho_generics
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
        pub(super) struct #ortho_vec_name #generics
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
            pub(super) fn len(&self) -> usize {
                self.#first_ident_name.len()
            }
        }
    );

    let empty_vecs_with_value_capacity_ts_iter =
        transform_named_fields_into_ts(data_struct, &|named_field| {
            let field_ident = named_field.ident.as_ref().unwrap();

            quote! {
                #field_ident: Vec::with_capacity(value.len()),
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
                    #empty_vecs_with_value_capacity_ts_iter
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
        pub(super) trait #into_ortho_name {
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

fn impl_vec_method_mut_self_move_struct(
    struct_name: &Ident,
    method_name: &str,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let method_name = Ident::new(method_name, Span::call_site());

    let call_method_on_props_pass_value =
        transform_named_fields_into_ts(data_struct, &|named_field| {
            let field_ident = named_field.ident.as_ref().unwrap();

            quote! {
                self.#field_ident.#method_name(value.#field_ident);
            }
        });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn #method_name(&mut self, value: #struct_name #generics_no_trait_bounds) {
                #call_method_on_props_pass_value
            }
        }
    }
}

fn impl_vec_method_mut_self_ret_opt_struct(
    struct_name: &Ident,
    method_name: &str,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let method_name = Ident::new(method_name, Span::call_site());

    let call_method_on_props_assign_member =
        transform_named_fields_into_ts(data_struct, &|named_field| {
            let field_ident = named_field.ident.as_ref().unwrap();

            quote! {
                #field_ident: self.#field_ident.#method_name()?,
            }
        });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn #method_name(&mut self) -> Option<#struct_name #generics_no_trait_bounds> {
                Some(#struct_name {
                    #call_method_on_props_assign_member
                })
            }
        }
    }
}

fn impl_vec_method_mut_self(
    method_name: &str,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let method_name = Ident::new(method_name, Span::call_site());

    let call_method_on_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            self.#field_ident.#method_name();
        }
    });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn #method_name(&mut self) {
                #call_method_on_props
            }
        }
    }
}

fn impl_vec_insert(
    struct_name: &Ident,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let call_insert_on_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            self.#field_ident.insert(index, element.#field_ident);
        }
    });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn insert(&mut self, index: usize, element: #struct_name #generics_no_trait_bounds) {
                #call_insert_on_props
            }
        }
    }
}

fn impl_vec_method_mut_self_index_ret_struct(
    struct_name: &Ident,
    method_name: &str,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let method_name = Ident::new(method_name, Span::call_site());

    let call_method_on_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            #field_ident: self.#field_ident.#method_name(index),
        }
    });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn #method_name(&mut self, index: usize) -> #struct_name #generics_no_trait_bounds {
                #struct_name {
                    #call_method_on_props
                }
            }
        }
    }
}

fn impl_vec_new(
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let call_new_on_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_ty = &named_field.ty;

        quote! {
            #field_ident: Vec::<#field_ty>::new(),
        }
    });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn new() -> #ortho_vec_name #generics_no_trait_bounds {
                #ortho_vec_name {
                    #call_new_on_props
                }
            }
        }
    }
}

fn impl_vec_with_capacity(
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let call_with_capacity_on_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_ty = &named_field.ty;

        quote! {
            #field_ident: Vec::<#field_ty>::with_capacity(capacity),
        }
    });

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    quote! {
        impl #generics #ortho_vec_name #generics_no_trait_bounds
        #where_clause {
            pub(super) fn with_capacity(capacity: usize) -> #ortho_vec_name #generics_no_trait_bounds {
                #ortho_vec_name {
                    #call_with_capacity_on_props
                }
            }
        }
    }
}

fn build_ortho_vec_impl_vec_methods(
    struct_name: &Ident,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> proc_macro2::TokenStream {
    let mut_self_move_struct_methods = ["push"].iter().map(|method_name| {
        impl_vec_method_mut_self_move_struct(
            struct_name,
            method_name,
            ortho_vec_name,
            data_struct,
            generics,
            where_clause,
        )
    });

    let mut_self_ret_opt_struct_methods = ["pop"].iter().map(|method_name| {
        impl_vec_method_mut_self_ret_opt_struct(
            struct_name,
            method_name,
            ortho_vec_name,
            data_struct,
            generics,
            where_clause,
        )
    });

    // dedup() and sort()/sort_unstable() won't simply work as we need to compare the elements
    let mut_self_methods = ["clear", "shrink_to_fit", "reverse"]
        .iter()
        .map(|method_name| {
            impl_vec_method_mut_self(
                method_name,
                ortho_vec_name,
                data_struct,
                generics,
                where_clause,
            )
        });

    let insert = impl_vec_insert(
        struct_name,
        ortho_vec_name,
        data_struct,
        generics,
        where_clause,
    );

    let mut_self_index_ret_struct_methods = ["remove", "swap_remove"].iter().map(|method_name| {
        impl_vec_method_mut_self_index_ret_struct(
            struct_name,
            method_name,
            ortho_vec_name,
            data_struct,
            generics,
            where_clause,
        )
    });

    let new = impl_vec_new(ortho_vec_name, data_struct, generics, where_clause);

    let with_capacity = impl_vec_with_capacity(ortho_vec_name, data_struct, generics, where_clause);

    quote! {
        #(#mut_self_move_struct_methods)*
        #(#mut_self_ret_opt_struct_methods)*
        #(#mut_self_methods)*
        #insert
        #(#mut_self_index_ret_struct_methods)*
        #new
        #with_capacity
    }
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
            // SAFETY: We do a bounds check one time on the first vector
            #field_ident: unsafe { self.v.#field_ident.get_unchecked(self.index - 1) },
        }
    });

    (
        ortho_vec_iter_name.clone(),
        quote!(
            pub(super) struct #ortho_vec_iter_name #ortho_generics
            #where_clause
            {
                v: & #ortho_lifetime #ortho_vec_name #generics_no_trait_bounds,
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
                pub(super) fn iter(&#ortho_lifetime self) -> #ortho_vec_iter_name #ortho_generics_no_trait_bounds {
                    #ortho_vec_iter_name {
                        v: &self,
                        index: 0
                    }
                }
            }
        ),
    )
}

fn build_ortho_vec_iter_mut_struct(
    name: &Ident,
    ortho_struct_mut_name: &Ident,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
    ortho_lifetime: &Lifetime,
) -> (Ident, proc_macro2::TokenStream) {
    let ortho_vec_iter_mut_name = Ident::new(
        &("OrthoVecIterMut".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let ortho_generics = add_lifetime_to_generics(generics, ortho_lifetime);
    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);
    let ortho_generics_no_trait_bounds = remove_trait_bounds_from_generics(&ortho_generics);

    let vec_iter_mut_define_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_type = &named_field.ty;

        quote! {
            #field_ident: &#ortho_lifetime mut [#field_type],
        }
    });

    let vec_iter_mut_assign_props_from_self =
        transform_named_fields_into_ts(data_struct, &|named_field| {
            let field_ident = named_field.ident.as_ref().unwrap();

            quote! {
                #field_ident: self.#field_ident.as_mut_slice(),
            }
        });

    let mut_entry_props_assign_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            // SAFETY: The borrow will live long enough because the originial slice lives for 'ortho
            #field_ident: unsafe { &mut *(#field_ident as *mut _) },
        }
    });

    let split_at_first_assignment = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let rest_of_ident = Ident::new(
            &("rest_of_".to_string() + &field_ident.to_string()),
            Span::call_site(),
        );

        quote! {
            // SAFETY: We do a bounds check one time on the first slice
            let (#field_ident, #rest_of_ident) = unsafe { self.#field_ident.split_first_mut().unwrap_unchecked() };
        }
    });

    let assign_rest_of_to_self = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let rest_of_ident = Ident::new(
            &("rest_of_".to_string() + &field_ident.to_string()),
            Span::call_site(),
        );

        quote! {
            // SAFETY: The slice will live long enough because the originial slice lives for 'ortho
            self.#field_ident = unsafe { &mut *(#rest_of_ident as *mut _) };
        }
    });

    let first_ident_name = take_first_named_field_ts(data_struct);

    (
        ortho_vec_iter_mut_name.clone(),
        quote!(
            pub(super) struct #ortho_vec_iter_mut_name #ortho_generics
            #where_clause
            {
                #vec_iter_mut_define_props
                index: usize,
            }

            impl #ortho_generics Iterator for #ortho_vec_iter_mut_name #ortho_generics_no_trait_bounds
            #where_clause
            {
                type Item = #ortho_struct_mut_name #ortho_generics_no_trait_bounds;

                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    if self.index >= self.#first_ident_name.len() {
                        None
                    } else {
                        self.index += 1;
                        #split_at_first_assignment

                        #assign_rest_of_to_self
                        Some(#ortho_struct_mut_name {
                            #mut_entry_props_assign_iter
                        })
                    }
                }
            }

            impl #ortho_generics #ortho_vec_name #generics_no_trait_bounds
            #where_clause
            {
                pub(super) fn iter_mut(&#ortho_lifetime mut self) -> #ortho_vec_iter_mut_name #ortho_generics_no_trait_bounds {
                    #ortho_vec_iter_mut_name {
                        #vec_iter_mut_assign_props_from_self
                        index: 0
                    }
                }
            }
        ),
    )
}

fn build_ortho_vec_into_iter_struct(
    name: &Ident,
    ortho_vec_name: &Ident,
    data_struct: &DataStruct,
    generics: &Generics,
    where_clause: &Option<WhereClause>,
) -> (Ident, proc_macro2::TokenStream) {
    let ortho_vec_into_iter_name = Ident::new(
        &("OrthoVecIntoIter".to_string() + &name.to_string()),
        Span::call_site(),
    );

    let generics_no_trait_bounds = remove_trait_bounds_from_generics(generics);

    let into_iter_props = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();
        let field_ty = named_field.ty.clone();

        quote! {
            #field_ident: <Vec<#field_ty> as IntoIterator>::IntoIter,
        }
    });

    let iter_props_assign_into_iter = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            // SAFETY: We do a bounds check
            #field_ident: unsafe { self.#field_ident.next().unwrap_unchecked() },
        }
    });

    let into_iter_for_each_vec = transform_named_fields_into_ts(data_struct, &|named_field| {
        let field_ident = named_field.ident.as_ref().unwrap();

        quote! {
            #field_ident: self.#field_ident.into_iter(),
        }
    });

    (
        ortho_vec_into_iter_name.clone(),
        quote!(
            pub(super) struct #ortho_vec_into_iter_name #generics
            #where_clause
            {
                index: usize,
                len: usize,
                #into_iter_props
            }

            impl #generics Iterator for #ortho_vec_into_iter_name #generics_no_trait_bounds
            #where_clause
            {
                type Item = #name #generics_no_trait_bounds;

                #[inline]
                fn next(&mut self) -> Option<Self::Item> {
                    if self.index >= self.len {
                        None
                    } else {
                        self.index += 1;
                        Some(#name {
                            #iter_props_assign_into_iter
                        })
                    }
                }
            }

            impl #generics IntoIterator for #ortho_vec_name #generics_no_trait_bounds
            #where_clause
            {
                type Item = #name #generics_no_trait_bounds;
                type IntoIter = #ortho_vec_into_iter_name #generics_no_trait_bounds;

                fn into_iter(self) -> #ortho_vec_into_iter_name #generics_no_trait_bounds {
                    #ortho_vec_into_iter_name {
                        index: 0,
                        len: self.len(),
                        #into_iter_for_each_vec
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

        let (ortho_vec_name, ortho_vec_ts) =
            build_ortho_vec_struct(name, &data_struct, &generics, &where_clause);

        let ortho_vec_methods_ts = build_ortho_vec_impl_vec_methods(
            name,
            &ortho_vec_name,
            &data_struct,
            &generics,
            &where_clause,
        );

        let (ortho_struct_name, ortho_struct_ts) = build_ortho_struct(
            name,
            &data_struct,
            &generics,
            &where_clause,
            &ortho_lifetime,
        );

        let (_, ortho_vec_iter_ts) = build_ortho_vec_iter_struct(
            name,
            &ortho_struct_name,
            &ortho_vec_name,
            &data_struct,
            &generics,
            &where_clause,
            &ortho_lifetime,
        );

        let (ortho_struct_mut_name, ortho_struct_mut_ts) = build_ortho_struct_mut(
            name,
            &data_struct,
            &generics,
            &where_clause,
            &ortho_lifetime,
        );

        let (_, ortho_vec_iter_mut_ts) = build_ortho_vec_iter_mut_struct(
            name,
            &ortho_struct_mut_name,
            &ortho_vec_name,
            &data_struct,
            &generics,
            &where_clause,
            &ortho_lifetime,
        );

        let (_, ortho_vec_into_iter_ts) = build_ortho_vec_into_iter_struct(
            name,
            &ortho_vec_name,
            &data_struct,
            &generics,
            &where_clause,
        );

        let ortho_mod_name = Ident::new(
            &("ortho_mod_".to_string() + &name.to_string()),
            Span::call_site(),
        );

        quote! {
            // Wrap with a module so no one can mutate attributes unsafely
            pub(crate) mod #ortho_mod_name {
                use super::#struct_name_ident;

                #ortho_vec_ts
                #ortho_vec_methods_ts

                #ortho_struct_ts
                #ortho_vec_iter_ts

                #ortho_struct_mut_ts
                #ortho_vec_iter_mut_ts

                #ortho_vec_into_iter_ts
            }
            use #ortho_mod_name::*;
        }
    } else {
        quote!()
    };

    gen.into()
}
