// Part of ethercat-rs. Copyright 2018-2019 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

//! Support for deriving ethercat-plc traits for a struct.

extern crate proc_macro;  // needed even in 2018

use self::proc_macro::TokenStream;
use syn::parse_macro_input;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use quote::ToTokens;


#[proc_macro_derive(SlaveProcessImage, attributes(pdos, entry))]
pub fn derive_single_process_image(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let ident = input.ident;

    let id_str = ident.to_string();
    let slave_id = if id_str.starts_with("EK") {
        let nr = id_str[2..6].parse::<u32>().unwrap();
        quote!(ethercat::SlaveId { vendor_id: 2, product_code: (#nr << 16) | 0x2c52 })
    } else if id_str.starts_with("EL") {
        let nr = id_str[2..6].parse::<u32>().unwrap();
        quote!(ethercat::SlaveId { vendor_id: 2, product_code: (#nr << 16) | 0x3052 })
    } else {
        panic!("cannot interpret struct name '{}' into a slave ID", id_str);
    };

    let mut sync_infos = vec![];
    let mut pdo_regs = vec![];
    let mut running_size = 0usize;
    let mut pdo_mapping = std::collections::HashMap::new();

    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(flds), ..
    }) = input.data {
        for field in flds.named {
            let ty = field.ty.into_token_stream().to_string();
            let bitlen = match &*ty {
                "u8"  | "i8"  => 8,
                "u16" | "i16" => 16,
                "u32" | "i32" | "f32" => 32,
                "u64" | "i64" | "f64" => 64,
                _ => panic!("cannot handle type '{}' in image", ty)
            };
            for attr in &field.attrs {
                if attr.path.is_ident("entry") {
                    if let syn::Meta::List(syn::MetaList { nested, .. }) =
                        attr.parse_meta().unwrap()
                    {
                        let (pdo_str, ix, subix) = if nested.len() == 2 {
                            ("".into(), &nested[0], &nested[1])
                        } else {
                            let pdo = &nested[0];
                            (quote!(#pdo).to_string(), &nested[1], &nested[2])
                        };
                        pdo_regs.push(quote! {
                            (ethercat::PdoEntryIdx { idx: ethercat::Idx::from(#ix),
                                                     sub_idx: ethercat::SubIdx::from(#subix) },
                             ethercat::Offset { byte: #running_size, bit: 0 })
                        });
                        pdo_mapping.entry(pdo_str).or_insert_with(Vec::new).push(quote! {
                            ethercat::PdoEntryInfo {
                                entry_idx: ethercat::PdoEntryIdx {
                                    idx: ethercat::Idx::from(#ix),
                                    sub_idx: ethercat::SubIdx::from(#subix)
                                },
                                bit_len: #bitlen as u8,
                                name: String::new(),
                                pos: ethercat::PdoEntryPos::from(0),  // unused
                            }
                        });
                    }
                }
            }
            running_size += bitlen / 8;
        }
    } else {
        panic!("SlaveProcessImage must be a struct with named fields");
    }

    for attr in &input.attrs {
        if attr.path.is_ident("pdos") {
            if let syn::Meta::List(syn::MetaList { nested, .. }) =
                attr.parse_meta().unwrap()
            {
                let sm = &nested[0];
                let sd = &nested[1];
                let mut pdos = vec![];
                for pdo_index in nested.iter().skip(2) {
                    let pdo_str = quote!(#pdo_index).to_string();
                    let entries = &pdo_mapping.get(&pdo_str).map_or(&[][..], |v| v);
                    pdos.push(quote! {
                        ethercat::PdoCfg {
                            idx: PdoIdx::from(#pdo_index),
                            entries: vec![#( #entries ),*]
                        }
                    })
                }
                sync_infos.push(quote! {
                    (
                        ethercat::SmCfg {
                            idx: SmIdx::from(#sm),
                            direction: ethercat::SyncDirection::#sd,
                            watchdog_mode: ethercat::WatchdogMode::Default,
                        },
                        vec![#( #pdos ),*]
                    )
                });
            }
        }
    }

    let sync_infos = if sync_infos.is_empty() {
        quote!(None)
    } else {
        quote!(Some(vec![#( #sync_infos ),*]))
    };

    let generated = quote! {
        #[automatically_derived]
        impl ProcessImage for #ident {
            const SLAVE_COUNT: usize = 1;
            fn get_slave_ids() -> Vec<SlaveId> { vec![#slave_id] }
            fn get_slave_pdos() -> Vec<Option<Vec<(SmCfg, Vec<PdoCfg>)>>> {
                vec![#sync_infos]
            }
            fn get_slave_regs() -> Vec<Vec<(PdoEntryIdx, Offset)>> {
                vec![vec![ #( #pdo_regs ),* ]]
            }
        }

        macro_rules! impl_it {
            ($n:literal) => {
                impl ProcessImage for [#ident; $n] {
                    const SLAVE_COUNT: usize = $n;
                    fn get_slave_ids() -> Vec<SlaveId> { vec![#slave_id; $n] }
                    fn get_slave_pdos() -> Vec<Option<Vec<(SmCfg, Vec<PdoCfg>)>>> {
                        vec![#sync_infos; $n]
                    }
                    fn get_slave_regs() -> Vec<Vec<(PdoEntryIdx, Offset)>> {
                        vec![vec![ #( #pdo_regs ),* ]; $n]
                    }
                }
            };
        }

        impl_it!(2);
        impl_it!(3);
        impl_it!(4);
        impl_it!(5);
        impl_it!(6);
        impl_it!(7);
        impl_it!(8);
    };

    // println!("{}", generated);
    generated.into()
}


fn sdo_extract(ix: &syn::NestedMeta, subix: &syn::NestedMeta, val: &syn::NestedMeta,
               sdos: &mut Vec<TokenStream2>) {
    match val {
        syn::NestedMeta::Lit(syn::Lit::Str(s)) => {
            let data_str = syn::parse_str::<syn::Expr>(&s.value()).unwrap();
            sdos.push(quote! {
                (ethercat::SdoIdx { idx: ethercat::Idx::from(#ix),
                                    sub_idx: ethercat::SubIdx::from(#subix) },
                 &#data_str)
            });
        }
        syn::NestedMeta::Meta(syn::Meta::Path(p)) => {
            sdos.push(quote! {
                (ethercat::SdoIdx { idx: ethercat::Idx::from(#ix),
                                    sub_idx: ethercat::SubIdx::from(#subix) },
                 {
                     match cfg.get_sdo_var(stringify!(#p)) {
                         None => panic!(concat!("required config value ",
                                                stringify!(#p), " not given")),
                         Some(x) => x
                     }
                 })
            });
        }
        _ => panic!("invalid SDO value, must be a string or identifier"),
    };
}


#[proc_macro_derive(ProcessImage, attributes(slave_id, sdo, array_sdo))]
pub fn derive_process_image(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let ident = input.ident;

    let mut slave_sdos = vec![];
    let mut slave_tys = vec![];
    let mut slave_ids = vec![];

    if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(flds), ..
    }) = input.data {
        for field in flds.named {
            let mut single_sdos = vec![];
            let mut array_sdos = vec![];
            let mut id = None;
            for attr in &field.attrs {
                if attr.path.is_ident("sdo") {
                    if let syn::Meta::List(syn::MetaList { nested, .. }) =
                        attr.parse_meta().unwrap()
                    {
                        sdo_extract(&nested[0], &nested[1], &nested[2], &mut single_sdos);
                    }
                } else if attr.path.is_ident("array_sdo") {
                    if let syn::Meta::List(syn::MetaList { nested, .. }) =
                        attr.parse_meta().unwrap()
                    {
                        let ix: usize = match &nested[0] {
                            syn::NestedMeta::Lit(syn::Lit::Int(lit)) => lit.base10_parse().unwrap(),
                            _ => panic!("invalid sdo_array index")
                        };
                        if array_sdos.len() < ix + 1 {
                            array_sdos.resize(ix + 1, vec![]);
                        }
                        sdo_extract(&nested[1], &nested[2], &nested[3], &mut array_sdos[ix]);
                    }
                } else if attr.path.is_ident("slave_id") {
                    if let syn::Meta::List(syn::MetaList { nested, .. }) =
                        attr.parse_meta().unwrap()
                    {
                        if let syn::NestedMeta::Lit(syn::Lit::Int(p)) = &nested[0] {
                            id = Some(quote!(
                                vec![ethercat::SlaveId { vendor_id: 2, product_code: (#p << 16) | 0x3052 }]
                            ));
                        }
                    }
                }
            }
            let ty = field.ty;
            if !array_sdos.is_empty() {
                for single in array_sdos {
                    slave_sdos.push(quote!( res.push(vec![#( #single ),*]); ));
                }
            } else if !single_sdos.is_empty() {
                slave_sdos.push(quote!( res.push(vec![#( #single_sdos ),*]); ));
            } else {
                slave_sdos.push(quote!( res.extend(#ty::get_slave_sdos(&())); ));
            }
            let id = id.unwrap_or(quote!( <#ty>::get_slave_ids() ));
            slave_tys.push(ty);
            slave_ids.push(id);
        }
    } else {
        return compile_error("only structs with named fields can be a process image");
    }

    let generated = quote! {
        #[automatically_derived]
        impl ProcessImage for #ident {
            const SLAVE_COUNT: usize = #( <#slave_tys>::SLAVE_COUNT )+*;
            fn get_slave_ids() -> Vec<ethercat::SlaveId> {
                let mut res = vec![]; #( res.extend(#slave_ids); )* res
            }
            fn get_slave_pdos() -> Vec<Option<Vec<(ethercat::SmCfg, Vec<ethercat::PdoCfg>)>>> {
                let mut res = vec![]; #( res.extend(<#slave_tys>::get_slave_pdos()); )* res
            }
            fn get_slave_regs() -> Vec<Vec<(ethercat::PdoEntryIdx, ethercat::Offset)>> {
                let mut res = vec![]; #( res.extend(<#slave_tys>::get_slave_regs()); )* res
            }
            fn get_slave_sdos<C: ethercat_plc::ProcessConfig>(cfg: &C) ->
                Vec<Vec<(ethercat::SdoIdx, &dyn ethercat::SdoData)>>
            {
                let mut res = vec![]; #(#slave_sdos)* res
            }
        }
    };

    // println!("{}", generated);
    generated.into()
}

#[proc_macro_derive(ExternImage, attributes(plc))]
pub fn derive_extern_image(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let ident = input.ident;

    // currently a no-op, later: auto-generate Default from #[plc] attributes
    let generated = quote! {
        impl ExternImage for #ident {}
    };
    generated.into()
}

fn compile_error(message: impl Into<String>) -> TokenStream {
    let message = message.into();
    quote!(compile_error! { #message }).into()
}
