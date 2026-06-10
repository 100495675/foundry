/// Contrato geográfico de tipos para la persistencia estática.
///
/// Un tipo que implementa `Shape` garantiza que su layout es determinista,
/// estable entre compilaciones y compatible con la deserialización binaria directa.
///
/// # Seguridad
///
/// Quien implemente este trait asume la garantía formal ante el compilador de que el tipo:
/// - No contiene punteros crudos ni referencias no-`'static`.
/// - No utiliza despacho dinámico (`dyn Trait`).
/// - No contiene tipos de referencia contable (`Rc`, `Arc`, `Weak`, etc.).
/// - No contiene tipos de celda (`RefCell`, `Cell`, `Mutex`, `RwLock`).
/// - No contiene handles del sistema (`File`, `TcpStream`, `UdpSocket`, etc.).
/// - Posee un layout determinista y estable entre compilaciones.
/// - Es compatible con `serde::Serialize` y `serde::Deserialize`.
pub unsafe trait Shape {}

// ─── Implementaciones estándar para tipos primitivos ─────────────────────────

unsafe impl Shape for i8 {}
unsafe impl Shape for i16 {}
unsafe impl Shape for i32 {}
unsafe impl Shape for i64 {}
unsafe impl Shape for i128 {}
unsafe impl Shape for isize {}
unsafe impl Shape for u8 {}
unsafe impl Shape for u16 {}
unsafe impl Shape for u32 {}
unsafe impl Shape for u64 {}
unsafe impl Shape for u128 {}
unsafe impl Shape for usize {}
unsafe impl Shape for f32 {}
unsafe impl Shape for f64 {}
unsafe impl Shape for bool {}
unsafe impl Shape for char {}
unsafe impl Shape for String {}
unsafe impl Shape for () {}

// ─── Colecciones estándar ────────────────────────────────────────────────────

unsafe impl<T: Shape> Shape for Vec<T> {}
unsafe impl<T: Shape> Shape for Option<T> {}
unsafe impl<T: Shape, E: Shape> Shape for Result<T, E> {}
unsafe impl<T: Shape> Shape for Box<T> {}

unsafe impl<K: Shape, V: Shape> Shape for std::collections::HashMap<K, V> {}
unsafe impl<T: Shape> Shape for std::collections::HashSet<T> {}
unsafe impl<K: Shape, V: Shape> Shape for std::collections::BTreeMap<K, V> {}
unsafe impl<T: Shape> Shape for std::collections::BTreeSet<T> {}

// ─── Arreglos y rebanadas ────────────────────────────────────────────────────

unsafe impl<T: Shape> Shape for [T] {}
unsafe impl<T: Shape, const N: usize> Shape for [T; N] {}

// ─── Referencias estáticas ───────────────────────────────────────────────────

unsafe impl Shape for &'static str {}
unsafe impl<T: Shape + ?Sized> Shape for &'static T {}
