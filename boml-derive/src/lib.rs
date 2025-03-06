use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, FieldsNamed, FieldsUnnamed, Generics, Ident, TypeParam, Variant};

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
		syn::Data::Enum(data) => derive_enum(ident, generics, data),
		syn::Data::Union(_) => unimplemented!(),
	}.into()
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

fn derive_struct(ident: Ident, generics: Generics, data: DataStruct) -> TokenStream {
		
	match data.fields {
		syn::Fields::Named(fields_named) => derive_named_struct(ident, generics, fields_named),
		syn::Fields::Unnamed(fields_unnamed) => derive_unnamed_struct(ident, generics, fields_unnamed),
		syn::Fields::Unit => derive_unit_struct(ident, generics),
	}
}

fn derive_named_struct(ident: Ident, generics: Generics, fields: FieldsNamed) -> TokenStream {
	let inner = fields.named.into_iter().map(|field| {			
		let ident = field.ident;	
		quote! { 
			#ident: table.get(stringify!(#ident)).toml_try_into()
				.map_err(|e| e.add_key_context(stringify!(#ident)))?
		}
	});

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);

	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>) 
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				match value {
					Some(TomlValue::Table(table)) => {
						Ok(Self {
							#(#inner),*
						})
					},
					Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
					None => Err(FromTomlError::Missing),
				}
			}
		}
	}
}
fn derive_unnamed_struct(ident: Ident, generics: Generics, fields: FieldsUnnamed) -> TokenStream {
	let inner = fields.unnamed.into_iter().enumerate().map(|(i, _)| {				
		let ident = Literal::string(&i.to_string());
		quote! { 
			table.get(#ident).toml_try_into()
				.map_err(|e| e.add_key_context(#ident))?
		}
	});

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);	

	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>) 
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				match value {
					Some(TomlValue::Table(table)) => {
						Ok(Self(
							#(#inner),*
						))
					},
					Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
					None => Err(FromTomlError::Missing>),
				}
			}
		}
	}
}

fn derive_unit_struct(ident: Ident, generics: Generics) -> TokenStream {
	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);
	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {			
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>) 
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				Ok(Self)
			}
		}
	}
}


fn derive_enum(ident: Ident, generics: Generics, data: DataEnum) -> TokenStream {
	let variants = data.variants.into_iter().map(|variant| {	
		let ident = variant.ident.clone();
		let ctor = enum_variant_ctor(variant);

		quote! {
			stringify!(#ident) => {
				return Ok(Self::#ctor);
			}
		}
	});

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);
	
	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {		
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>) 
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				// externally tagged
				let table = match value {
					Some(TomlValue::Table(table)) => {						
						let key = table.keys().next().ok_or(FromTomlError::Missing)?.as_str();
						let table_inner = table.get_table(key)?;
						
						match key {
							#(#variants),*,
							_ => return Err(FromTomlError::InvalidKey(key)),
						}
					},
					Some(v) => return Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
					None => return Err(FromTomlError::Missing),
				};
			}
		}
	}
}

fn enum_variant_ctor(variant: Variant) -> TokenStream {
	let ident = variant.ident;
	match variant.fields {
		syn::Fields::Named(fields_named) => enum_named_variant_ctor(ident, fields_named),
		syn::Fields::Unnamed(fields_unnamed) => enum_unnamed_variant_ctor(ident, fields_unnamed),
		syn::Fields::Unit => quote! { #ident },
	}
}

fn enum_named_variant_ctor(ident: Ident, fields: FieldsNamed) -> TokenStream {
	let inner = fields.named.into_iter().map(|field| {		
		let ident = field.ident;
		quote! { 
			#ident: table_inner.get(stringify!(#ident)).toml_try_into()
				.map_err(|e| e.add_key_context(stringify!(#ident)))?
		}
	});
	
	quote! {
		#ident {
			#(#inner),*
		}
	}
}

fn enum_unnamed_variant_ctor(ident: Ident, fields: FieldsUnnamed) -> TokenStream {
	let inner = fields.unnamed.into_iter().enumerate().map(|(i, _)| {					
		let ident = Literal::string(&i.to_string());
		quote! { 
			table_inner.get(#ident).toml_try_into()
				.map_err(|e| e.add_key_context(#ident))?
		}
	});
	
	quote! {
		#ident(
			#(#inner),*
		)
	}
}