use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Ident};

/// This derives the `FromRow` trait for structs
/// Requires that the query is in field order, as it just uses row indices
#[proc_macro_derive(ToSql)]
pub fn derive_to_sql(input: TokenStream) -> TokenStream {
    // Parse it as a proc macro
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            let name = input.ident.clone();

            let mut name_snake = input
                .ident
                .to_string()
                .to_case(Case::Snake)
                .replace("_", "s_");
            name_snake.push('s');
            let name_snake = Ident::new(&name_snake, name.span());

            // An iterator for `{field}`
            let field_names = fields
                .named
                .iter()
                .map(|field| field.ident.clone().unwrap().to_string())
                .collect::<Vec<_>>();

            // An iterator for `{field}`
            let field_name = fields.named.iter().map(|field| {
                let name = &field.ident;
                quote!(#name)
            });

            // An iterator for `{field}: {type}`
            let field_name_type = fields.named.iter().map(|field| {
                let name = &field.ident;
                let ty = &field.ty;
                quote!(#name: #ty)
            });

            // An iterator for `&self.{field}`
            let field_self_name = fields.named.iter().map(|field| {
                let name = &field.ident;
                quote!(&self.#name)
            });

            // An iterator for `{field}: row.get(0usize)?`
            let field_gets = fields.named.iter().enumerate().map(|(i, field)| {
                let name = &field.ident;
                quote!(#name: row.get(#i)?)
            });

            let new_docstring = format!("Creates a new `{name}` instance.");

            let upsert_docstring = format!("Inserts or updates a `{name}` in the database.");
            let upsert_statement = format!(
                "INSERT OR REPLACE INTO {name_snake} ({}) VALUES ({});",
                field_names.join(","),
                "?,".repeat(field_names.len()).trim_end_matches(",")
            );

            let retrieve_docstring = format!("Retrieves a `{name}` by its identifier.");
            let retrieve_many_docstring =
                format!("Retrieves `{name}` records by their identifiers.");
            let retrieve_many_statement = format!("SELECT * FROM {name_snake} WHERE id IN ({{}});");
            let retrieve_all_docstring = format!("Retrieves all `{name}` records.");
            let retrieve_all_statement = format!("SELECT * FROM {name_snake};");

            return TokenStream::from(quote!(
                impl #name {
                    #[doc = #new_docstring]
                    pub fn new(#(#field_name_type),*) -> Self {
                        Self { #(#field_name),* }
                    }

                    #[doc = #upsert_docstring]
                    pub fn upsert(&self, txn: &Transaction) -> Result<&Self> {
                        let mut stmt = txn.prepare_cached(#upsert_statement)?;
                        stmt.execute(params![#(#field_self_name),*])?;
                        Ok(self)
                    }

                    #[doc = #retrieve_docstring]
                    pub fn retrieve(txn: &Transaction, id: &Uuid) -> Result<Option<Self>> {
                        Ok(Self::retrieve_many(txn, &[*id])?.pop())
                    }

                    #[doc = #retrieve_many_docstring]
                    pub fn retrieve_many(txn: &Transaction, ids: &[Uuid]) -> Result<Vec<Self>> {
                        let mut stmt = txn.prepare_cached(&format!(
                            #retrieve_many_statement,
                            "?,".repeat(ids.len()).trim_end_matches(",")
                        ))?;
                        let mapped = stmt.query_map(params_from_iter(ids), |row| row.try_into())?;
                        Ok(mapped.collect::<rusqlite::Result<Vec<_>>>()?)
                    }

                    #[doc = #retrieve_all_docstring]
                    pub fn retrieve_all(txn: &Transaction) -> Result<Vec<Self>> {
                        let mut stmt = txn.prepare_cached(#retrieve_all_statement)?;
                        let mapped = stmt.query_map([], |row| row.try_into())?;
                        Ok(mapped.collect::<rusqlite::Result<Vec<_>>>()?)
                    }
                }

                impl TryFrom<&rusqlite::Row<'_>> for #name {
                    type Error = rusqlite::Error;

                    fn try_from(row: &rusqlite::Row<'_>) -> Result<Self, Self::Error> {
                        Ok(Self { #(#field_gets),* })
                    }
                }
            ));
        }
    }

    TokenStream::from(
        syn::Error::new(
            input.ident.span(),
            "Only structs with named fields can derive `ToSql`",
        )
        .to_compile_error(),
    )
}
