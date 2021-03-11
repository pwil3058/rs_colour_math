// Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

#[proc_macro_derive(Colour, attributes(colour))]
pub fn colour_interface_derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let (impl_generics, ty_generics, where_clause) = parsed_input.generics.split_for_impl();
    let mut first: Option<Ident> = None;
    let mut colour: Option<Ident> = None;
    match parsed_input.data {
        Data::Struct(st) => {
            if let syn::Fields::Named(fields) = st.fields {
                for field in fields.named.iter() {
                    if first.is_none() {
                        first = Some(field.ident.clone().unwrap());
                    }
                    for attr in field.attrs.iter() {
                        if attr.path.is_ident("colour") {
                            colour = Some(field.ident.clone().unwrap());
                        }
                    }
                }
            }
        }
        _ => panic!("'Colour' can only be derived for structs."),
    }
    let colour = if let Some(colour) = colour {
        colour
    } else {
        first.unwrap()
    };
    let tokens = quote! {
        impl #impl_generics colour_math::ColourBasics for #struct_name #ty_generics #where_clause {
            fn hue(&self) -> Option<colour_math::Hue> {
                self.#colour.hue()
            }

            fn hue_angle(&self) -> Option<colour_math::Angle> {
                self.#colour.hue_angle()
            }

            fn hue_rgb<L: LightLevel>(&self) -> Option<colour_math::RGB<L>> {
                self.#colour.hue_rgb::<L>()
            }

            fn hue_hcv(&self) -> Option<colour_math::HCV> {
                self.#colour.hue_hcv()
            }

            fn is_grey(&self) -> bool {
                self.#colour.is_grey()
            }

            fn chroma(&self) -> colour_math::Chroma {
                self.#colour.chroma()
            }

            fn value(&self) -> colour_math::Value {
                self.#colour.value()
            }

            fn greyness(&self) -> colour_math::Greyness {
                self.#colour.greyness()
            }

            fn warmth(&self) -> colour_math::Warmth {
                self.#colour.warmth()
            }

            fn hcv(&self) -> colour_math::HCV {
                self.#colour.hcv()
            }

            fn rgb<L: LightLevel>(&self) -> colour_math::RGB<L> {
                self.#colour.rgb::<L>()
            }

            fn monochrome_hcv(&self) -> colour_math::HCV {
                self.#colour.monochrome_hcv()
            }

            fn monochrome_rgb<L: LightLevel>(&self) -> colour_math::RGB<L> {
                self.#colour.monochrome_rgb::<L>()
            }

            fn best_foreground(&self) -> colour_math::HCV {
                self.#colour.best_foreground()
            }

            fn pango_string(&self) -> String {
                self.#colour.pango_string()
            }
        }

        impl #impl_generics colour_math::ColourAttributes for #struct_name #ty_generics #where_clause {}
        impl #impl_generics colour_math::ColourIfce for #struct_name #ty_generics #where_clause {}
    };

    proc_macro::TokenStream::from(tokens)
}
