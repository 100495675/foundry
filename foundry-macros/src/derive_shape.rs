use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Implementación de `#[derive(Shape)]`.
///
/// Opera como la primera línea de defensa sintáctica del ecosistema, interceptando
/// en tiempo de compilación el uso de tipos incompatibles con la persistencia estática
/// antes de que puedan corromper la memoria en runtime.
pub fn derive_shape_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    if let Err(e) = verificar_anatomia_sintactica(&input) {
        return e.to_compile_error().into();
    }

    let expanded = quote! {
        unsafe impl #impl_generics ::foundry::shape::Shape for #name #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}

/// Verifica la anatomía sintáctica del tipo derivado.
fn verificar_anatomia_sintactica(input: &DeriveInput) -> Result<(), syn::Error> {
    match &input.data {
        Data::Struct(s) => analizar_campos(&s.fields),
        Data::Enum(e) => {
            for v in &e.variants {
                analizar_campos(&v.fields)?;
            }
            Ok(())
        }
        Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Shape prohibe Unions (Layout no determinista)",
        )),
    }
}

/// Analiza los campos de una variante o estructura.
fn analizar_campos(fields: &Fields) -> Result<(), syn::Error> {
    let iterador: Box<dyn Iterator<Item = &syn::Field>> = match fields {
        Fields::Named(n) => Box::new(n.named.iter()),
        Fields::Unnamed(u) => Box::new(u.unnamed.iter()),
        Fields::Unit => Box::new(std::iter::empty()),
    };
    for field in iterador {
        evaluar_tipo(&field.ty)?;
    }
    Ok(())
}

/// Evalúa recursivamente un tipo para detectar construcciones prohibidas.
fn evaluar_tipo(ty: &Type) -> Result<(), syn::Error> {
    match ty {
        Type::Ptr(_) => Err(syn::Error::new_spanned(
            ty,
            "Shape prohibe el uso de punteros crudos.",
        )),
        Type::TraitObject(_) => Err(syn::Error::new_spanned(
            ty,
            "Shape no tolera despacho dinámico (dyn Trait).",
        )),
        Type::Reference(r) => {
            if let Some(lifetime) = &r.lifetime {
                if lifetime.ident.to_string() != "static" {
                    return Err(syn::Error::new_spanned(
                        lifetime,
                        "Shape solo permite referencias con lifetime 'static",
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    r,
                    "Referencias requieren un lifetime 'static explícito.",
                ));
            }
            evaluar_tipo(&r.elem)
        }
        Type::Path(p) => {
            if let Some(segment) = p.path.segments.last() {
                let ident_str = segment.ident.to_string();
                const LISTA_NEGRA: &[&str] = &[
                    "Rc",
                    "Arc",
                    "Weak",
                    "RefCell",
                    "Cell",
                    "Mutex",
                    "RwLock",
                    "File",
                    "TcpStream",
                    "UdpSocket",
                    "Sender",
                    "Receiver",
                ];
                if LISTA_NEGRA.contains(&ident_str.as_str()) {
                    return Err(syn::Error::new_spanned(
                        segment,
                        format!("El tipo [{}] rompe los invariantes de Shape.", ident_str),
                    ));
                }
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            evaluar_tipo(inner_ty)?;
                        }
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
