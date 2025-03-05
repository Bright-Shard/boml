use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, Generics, Ident, ItemImpl, Lifetime, LifetimeParam, TypeParam};

#[proc_macro_derive(FromToml)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let DeriveInput {
		ident,
		generics,
		data,
		..
	} = parse_macro_input!(input);	
	
	match data {
		syn::Data::Struct(data) => derive_struct(ident, generics, data),
		syn::Data::Enum(_) => unimplemented!(),
		syn::Data::Union(_) => unimplemented!(),
	}.into()
} 

fn derive_struct(ident: syn::Ident, generics: syn::Generics, data: syn::DataStruct) -> TokenStream {
	let output = match data.fields {
		syn::Fields::Named(fields_named) => derive_named_struct(ident, generics, fields_named),
		syn::Fields::Unnamed(fields_unnamed) => todo!(),
		syn::Fields::Unit => quote! {
			impl<'t, 'r> TryFrom<&'r Toml<'t>> for #ident #generics {
				type Error = TomlGetError<'r, 't>;

				fn try_from(value: &'r Toml<'t>) -> Result<Self, Self::Error> {
					Ok(Self {})
				}
			}
		},
	};	
	output
}


fn generate_ty_generics(generics: &Generics) -> TokenStream {
	let lifetimes = generics.lifetimes().map(|_| {		
		quote! { '__boml_derive_a }
	});

	let ty_params = generics.type_params().map(|ty_param| {
		let ident = &ty_param.ident;
		quote! { #ident }
	});

	quote! { <#(#lifetimes),*, #(#ty_params),*> }
}

fn generate_impl_generics(generics: &Generics) -> TokenStream {
	let ty_params = generics.type_params().map(|ty_param| {
		let mut bounds = ty_param.bounds.clone();
		bounds.push(syn::parse_quote! { FromToml<'__boml_derive_a> });
		TypeParam {
			bounds,
			..ty_param.clone()
		}
	});

	quote! { <'__boml_derive_a, #(#ty_params),*> }
}

fn derive_named_struct(ident: syn::Ident, generics: syn::Generics, fields: FieldsNamed) -> TokenStream {
	let inner = fields.named.into_iter().map(|field| {		
		let ident = field.ident;
		quote! { 
			#ident: table.get(stringify!(#ident)).toml_try_into()? 
		}
	});

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);

	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>) 
				-> Result<Self, TomlGetError<'__boml_derive_a, '__boml_derive_a>> {
				match value {
					Some(TomlValue::Table(table)) => {
						Ok(Self {
							#(#inner),*
						})
					},
					Some(v) => Err(TomlGetError::TypeMismatch(v, TomlValueType::Table)),
					None => Err(TomlGetError::InvalidKey),
				}
			}
		}
	}
}
