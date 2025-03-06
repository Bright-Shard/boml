use std::ops::Deref;

use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{
	parse::Parse, parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, FieldsNamed,
	FieldsUnnamed, Generics, Ident, Token, TypeParam, Variant,
};

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
		syn::Data::Enum(data) => {
			derive_enum(ident, generics, attrs, data).unwrap_or_else(|e| e.to_compile_error())
		}
		syn::Data::Union(_) => unimplemented!(),
	}
	.into()
}

fn generate_ty_generics(generics: &Generics) -> TokenStream {
	let lifetimes = generics.lifetimes().map(|_| {
		quote! { '__boml_derive_a }
	});

	let ty_params = generics.type_params().map(|ty_param| {
		let ident = &ty_param.ident;
		quote! { #ident }
	});

	let params = lifetimes.chain(ty_params);

	quote! { <#(#params),*> }
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
		syn::Fields::Unnamed(fields_unnamed) => {
			derive_unnamed_struct(ident, generics, fields_unnamed)
		}
		syn::Fields::Unit => derive_unit_struct(ident, generics),
	}
}

fn derive_named_struct(ident: Ident, generics: Generics, fields: FieldsNamed) -> TokenStream {
	let ctor = create_named_ctor(ident.clone(), fields);

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);

	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>)
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				match value {
					Some(TomlValue::Table(table)) => Ok(#ctor),
					Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
					None => Err(FromTomlError::Missing),
				}
			}
		}
	}
}
fn derive_unnamed_struct(ident: Ident, generics: Generics, fields: FieldsUnnamed) -> TokenStream {
	let ctor = create_unnamed_ctor(ident.clone(), fields);

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);

	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>)
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				match value {
					Some(TomlValue::Table(table)) => Ok(#ctor),
					Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
					None => Err(FromTomlError::Missing),
				}
			}
		}
	}
}

fn derive_unit_struct(ident: Ident, generics: Generics) -> TokenStream {
	let impl_generics = generate_impl_generics(&generics);
	quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident {
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

fn derive_enum(
	ident: Ident,
	generics: Generics,
	attrs: Vec<Attribute>,
	data: DataEnum,
) -> Result<TokenStream, syn::Error> {
	let variants = data.variants.into_iter().map(|variant| {
		let ident = variant.ident.clone();
		let ctor = enum_variant_ctor(variant);

		quote! {
			stringify!(#ident) => {
				return Ok(Self::#ctor);
			}
		}
	});

	let attr_fields: Vec<_> = attrs
		.into_iter()
		.filter(|attr| attr.path().is_ident("boml"))
		.map(|attr| attr.parse_args().map(|attr: BomlAttr| attr.0))
		.collect::<Result<_, _>>()?;

	let attr_fields: BomlAttr = attr_fields.into_iter().flatten().collect();
	attr_fields.check_duplicates()?;
	let strategy = EnumStrategy::try_from(attr_fields)?;

	let strategy_quote = match strategy {
		EnumStrategy::ValueEnum => {
			return Err(syn::Error::new(
				Span::call_site(),
				"value_enum is not implemented yet",
			))
		}
		EnumStrategy::Untagged => {
			return Err(syn::Error::new(
				Span::call_site(),
				"untagged is not implemented yet",
			))
		}
		EnumStrategy::TagExternal => quote! {
			let key = table.keys().next().ok_or(FromTomlError::Missing)?.as_str();
			let table = table.get_table(key)
				.map_err(|e| FromTomlError::from(e).add_key_context(key))?;

			match key {
				#(#variants),*,
				_ => return Err(FromTomlError::InvalidKey(key)),
			}
		},
		EnumStrategy::TagInternal(tag) => quote! {
			let key = table.get_string(#tag)
				.map_err(|e| FromTomlError::from(e).add_key_context(#tag))?;

			match key {
				#(#variants),*,
				_ => return Err(FromTomlError::InvalidKey(key)),
			}
		},
		EnumStrategy::TagAdjecent(tag, content) => quote! {
			let key = table.get_string(#tag)
				.map_err(|e| FromTomlError::from(e).add_key_context(#tag))?;

			let table = table.get_table(#content)
				.map_err(|e| FromTomlError::from(e).add_key_context(#tag))?;

			match key {
				#(#variants),*,
				_ => return Err(FromTomlError::InvalidKey(key)),
			}
		},
	};

	let ty_generics = generate_ty_generics(&generics);
	let impl_generics = generate_impl_generics(&generics);

	Ok(quote! {
		impl #impl_generics FromToml<'__boml_derive_a> for #ident #ty_generics {
			fn from_toml(value: Option<&'__boml_derive_a TomlValue<'__boml_derive_a>>)
				-> Result<Self, FromTomlError<'__boml_derive_a>> {
				// externally tagged
				let table = match value {
					Some(TomlValue::Table(table)) => {
						#strategy_quote
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
		syn::Fields::Named(fields_named) => create_named_ctor(ident, fields_named),
		syn::Fields::Unnamed(fields_unnamed) => create_unnamed_ctor(ident, fields_unnamed),
		syn::Fields::Unit => quote! { #ident },
	}
}

fn create_named_ctor(ident: Ident, fields: FieldsNamed) -> TokenStream {
	let inner = fields.named.into_iter().map(|field| {
		let ident = field.ident;
		quote! {
			#ident: table.get(stringify!(#ident)).toml_try_into()
				.map_err(|e| e.add_key_context(stringify!(#ident)))?
		}
	});

	quote! {
		#ident {
			#(#inner),*
		}
	}
}

fn create_unnamed_ctor(ident: Ident, fields: FieldsUnnamed) -> TokenStream {
	let inner = fields.unnamed.into_iter().enumerate().map(|(i, _)| {
		let ident = Literal::string(&i.to_string());
		quote! {
			table.get(#ident).toml_try_into()
				.map_err(|e| e.add_key_context(#ident))?
		}
	});

	quote! {
		#ident(
			#(#inner),*
		)
	}
}

#[derive(PartialEq)]
enum EnumStrategy {
	ValueEnum,
	Untagged,
	TagExternal,
	TagInternal(String),
	TagAdjecent(String, String),
}

impl TryFrom<BomlAttr> for EnumStrategy {
	type Error = syn::Error;
	fn try_from(value: BomlAttr) -> Result<Self, Self::Error> {
		macro_rules! mutex_error {
			($span:expr, $lhs:expr,$rhs:expr) => {
				Err(syn::Error::new(
					$span,
					format!("`{}` and `{}` are mutually exclusive", $lhs, $rhs),
				))
			};
		}
		let mut tag: Option<String> = None;
		let mut content: Option<String> = None;
		let mut mode = None;

		for field in value {
			match field {
				BomlAttrField::ValueEnum(span) => {
					if tag.is_some() {
						return mutex_error!(span, "value_enum", "tag");
					}
					if content.is_some() {
						return mutex_error!(span, "value_enum", "content");
					}
					if mode == Some(EnumStrategy::Untagged) {
						return mutex_error!(span, "value_enum", "untagged");
					}
					mode = Some(EnumStrategy::ValueEnum);
				}
				BomlAttrField::Untagged(span) => {
					if tag.is_some() {
						return mutex_error!(span, "untagged", "tag");
					}
					if content.is_some() {
						return mutex_error!(span, "untagged", "content");
					}
					if mode == Some(EnumStrategy::ValueEnum) {
						return mutex_error!(span, "untagged", "value_enum");
					}
					mode = Some(EnumStrategy::Untagged);
				}
				BomlAttrField::Tag(span, t) => {
					if mode == Some(EnumStrategy::ValueEnum) {
						return mutex_error!(span, "tag", "value_enum");
					}
					if mode == Some(EnumStrategy::Untagged) {
						return mutex_error!(span, "tag", "untagged");
					}
					if let Some(content) = &content {
						mode = Some(EnumStrategy::TagAdjecent(t, content.clone()));
					} else {
						mode = Some(EnumStrategy::TagInternal(t.clone()));
						tag = Some(t);
					}
				}
				BomlAttrField::Content(span, c) => {
					if mode == Some(EnumStrategy::ValueEnum) {
						return mutex_error!(span, "tag", "value_enum");
					}
					if mode == Some(EnumStrategy::Untagged) {
						return mutex_error!(span, "tag", "untagged");
					}
					if let Some(tag) = &tag {
						mode = Some(EnumStrategy::TagAdjecent(tag.clone(), c));
					} else {
						content = Some(c);
					}
				}
			}
		}
		if content.is_some() && tag.is_none() {
			return Err(syn::Error::new(
				Span::call_site(),
				"`content` requires `tag`",
			));
		}

		Ok(mode.unwrap_or(EnumStrategy::TagExternal))
	}
}

// -------------------------------------------------------------------------------------------------
// boml attribute
// -------------------------------------------------------------------------------------------------

#[proc_macro_attribute]
pub fn boml(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	item
}

struct BomlAttr(Vec<BomlAttrField>);

impl BomlAttr {
	fn check_duplicates(&self) -> Result<(), syn::Error> {
		let mut tags = std::collections::HashSet::new();
		for field in &self.0 {
			if let BomlAttrField::Tag(span, tag) = field {
				if !tags.insert(tag) {
					return Err(syn::Error::new(*span, "duplicate tag"));
				}
			}
		}
		Ok(())
	}
}

impl Deref for BomlAttr {
	type Target = Vec<BomlAttrField>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl IntoIterator for BomlAttr {
	type Item = BomlAttrField;
	type IntoIter = std::vec::IntoIter<BomlAttrField>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}
impl FromIterator<BomlAttrField> for BomlAttr {
	fn from_iter<T: IntoIterator<Item = BomlAttrField>>(iter: T) -> Self {
		BomlAttr(iter.into_iter().collect())
	}
}

impl Parse for BomlAttr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let fields = input.parse_terminated(BomlAttrField::parse, Token![,])?;
		Ok(BomlAttr(fields.into_iter().collect()))
	}
}

enum BomlAttrField {
	ValueEnum(Span),
	Untagged(Span),
	Tag(Span, String),
	Content(Span, String),
}

impl Parse for BomlAttrField {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let ident: syn::Ident = input.parse()?;
		match ident.to_string().as_str() {
			"value_enum" => Ok(BomlAttrField::ValueEnum(ident.span())),
			"untagged" => Ok(BomlAttrField::Untagged(ident.span())),
			"tag" => {
				input.parse::<syn::Token![=]>()?;
				let tag: syn::LitStr = input.parse()?;
				Ok(BomlAttrField::Tag(ident.span(), tag.value()))
			}
			"content" => {
				input.parse::<syn::Token![=]>()?;
				let content: syn::LitStr = input.parse()?;
				Ok(BomlAttrField::Content(ident.span(), content.value()))
			}
			_ => Err(syn::Error::new(ident.span(), "unknown boml attribute")),
		}
	}
}
