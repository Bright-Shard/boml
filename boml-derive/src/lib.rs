use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{parse::{Parse, Parser}, parse_macro_input, spanned::Spanned, Attribute, DataEnum, DataStruct, DeriveInput, FieldsNamed, FieldsUnnamed, Generics, Ident, Token, TypeParam, Variant};

#[proc_macro_derive(FromToml)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let DeriveInput {
		ident,
		generics,
		data,
		attrs,
		..
	} = parse_macro_input!(input);	
	
	match data {
		syn::Data::Struct(data) => derive_struct(ident, generics, data),
		syn::Data::Enum(data) => derive_enum(ident, generics, attrs, data)
			.unwrap_or_else(|e| e.to_compile_error()),
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

// -------------------------------------------------------------------------------------------------
// Enum
// -------------------------------------------------------------------------------------------------

fn derive_enum(ident: Ident, generics: Generics, attrs: Vec<Attribute>, data: DataEnum) -> Result<TokenStream, syn::Error> {
	let variants = data.variants.into_iter().map(|variant| {	
		let ident = variant.ident.clone();
		let ctor = enum_variant_ctor(variant);

		quote! {
			stringify!(#ident) => {
				return Ok(Self::#ctor);
			}
		}
	});

	let attr_fields: Vec<_> = attrs.into_iter()
	.filter(|attr| attr.path().is_ident("boml"))
	.map(|attr| {
		attr.parse_args().map(|attr: BomlAttr| attr.0)
	}).collect::<Result<_,_>>()?;

	let attr_fields = attr_fields.into_iter().flatten().collect::<Vec<_>>();
	
	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);
	
	Ok(quote! {
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
	})
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

enum EnumMode {
	ValueEnum,
	Untagged,
	TagInternal(String),
	TagExternal(String, String),
}

// -------------------------------------------------------------------------------------------------
// boml attribute
// -------------------------------------------------------------------------------------------------

#[proc_macro_attribute]
pub fn boml(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
	item
}


struct BomlAttr(Vec<BomlAttrField>);

impl Parse for BomlAttr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let fields = input.parse_terminated(BomlAttrField::parse, Token![,])?;
		Ok(BomlAttr(fields.into_iter().collect()))
	}
}

enum BomlAttrField {
	ValueEnum,
	Untagged,
	Tag(String),
	Content(String),
}

impl Parse for BomlAttrField {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let ident: syn::Ident = input.parse()?;
		match ident.to_string().as_str() {
			"value_enum" => Ok(BomlAttrField::ValueEnum),
			"untagged" => Ok(BomlAttrField::Untagged),
			"tag" => {
				input.parse::<syn::Token![=]>()?;
				let tag: syn::LitStr = input.parse()?;
				Ok(BomlAttrField::Tag(tag.value()))
			},
			"content" => {
				input.parse::<syn::Token![=]>()?;
				let content: syn::LitStr = input.parse()?;
				Ok(BomlAttrField::Content(content.value()))
			},
			_ => Err(syn::Error::new(ident.span(), "unknown boml attribute")),
		}
	}
}

