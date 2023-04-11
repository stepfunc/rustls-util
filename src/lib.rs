#![doc = include_str!("../README.md")]
#![deny(
dead_code,
arithmetic_overflow,
invalid_type_param_default,
missing_fragment_specifier,
mutable_transmutes,
no_mangle_const_items,
overflowing_literals,
patterns_in_fns_without_body,
pub_use_of_private_extern_crate,
unknown_crate_types,
order_dependent_trait_objects,
illegal_floating_point_literal_pattern,
improper_ctypes,
late_bound_lifetime_arguments,
non_camel_case_types,
non_shorthand_field_patterns,
non_snake_case,
non_upper_case_globals,
no_mangle_generic_items,
private_in_public,
stable_features,
type_alias_bounds,
tyvar_behind_raw_pointer,
unconditional_recursion,
unused_comparisons,
unreachable_pub,
anonymous_parameters,
missing_copy_implementations,
//missing_debug_implementations,
missing_docs,
trivial_casts,
trivial_numeric_casts,
unused_import_braces,
unused_qualifications,
clippy::all
)]
#![forbid(
    unsafe_code,
    rustdoc::broken_intra_doc_links,
    while_true,
    bare_trait_objects
)]

mod client;
mod name;
mod self_signed;
mod server;

pub use client::*;
pub use name::*;
pub use self_signed::*;
pub use server::*;

pub(crate) fn pki_error(error: webpki::Error) -> rustls::Error {
    use webpki::Error::*;
    match error {
        BadDer | BadDerTime => rustls::CertificateError::BadEncoding.into(),
        CertNotValidYet => rustls::CertificateError::NotValidYet.into(),
        CertExpired | InvalidCertValidity => rustls::CertificateError::Expired.into(),
        UnknownIssuer => rustls::CertificateError::UnknownIssuer.into(),
        CertNotValidForName => rustls::CertificateError::NotValidForName.into(),

        InvalidSignatureForPublicKey
        | UnsupportedSignatureAlgorithm
        | UnsupportedSignatureAlgorithmForPublicKey => {
            rustls::CertificateError::BadSignature.into()
        }
        _ => rustls::CertificateError::Other(std::sync::Arc::new(error)).into(),
    }
}
