use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Visibility};
fn impl_query_type(vis: &Visibility, ident: &Ident, fields: &Fields) -> TokenStream {
    let query_ident = Ident::new(&format!("{ident}Query"), ident.span());

    let fields = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        quote! {
            pub #field_ident: ::rejis::Query<#field_type, Root>,
        }
    });

    quote! {
        #[derive(Debug, Clone)]
        #vis struct #query_ident<Root: Table> {
            #(#fields)*
        }

        impl<Root: Table> ::rejis::Queryable<Root> for #ident {
            type QueryType = #query_ident<Root>;
        }
    }
}

fn impl_field_query(ident: &Ident, fields: &Fields) -> TokenStream {
    let query_ident = Ident::new(&format!("{ident}Query"), ident.span());

    let fields = fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = field_ident.to_string();

        quote! {
            #field_ident: ::rejis::Query::new(path.join(#field_name)),
        }
    });

    quote! {
        impl<Root: Table> ::rejis::QueryConstructor<Root> for #query_ident<Root> {
            type Inner = #ident;
            fn new<Field: ::rejis::Queryable<Root>>(path: &::rejis::path::Path) -> Self {
                #query_ident {
                    #(#fields)*
                }
            }
        }
    }
}

#[proc_macro_derive(Queryable)]
pub fn derive_queryable(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let derive: DeriveInput = syn::parse2(stream).unwrap();

    let Data::Struct(data) = derive.data else {
        panic!("Queryable can only be derived from structs currently.");
    };

    let query_type = impl_query_type(&derive.vis, &derive.ident, &data.fields);
    let field_query_impl = impl_field_query(&derive.ident, &data.fields);

    quote! {
        #query_type
        #field_query_impl
    }
    .into()
}

#[proc_macro_derive(Table)]
pub fn derive_table(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = TokenStream::from(stream);
    let derive: DeriveInput = syn::parse2(stream).unwrap();

    let ident = derive.ident;
    let table_name = ident.to_string().to_lowercase();

    quote! {
        impl ::rejis::Table for #ident {
            const TABLE_NAME: &'static str = #table_name;
        }
    }
    .into()
}
