pub mod key;
pub use self::key::{ClassicKey, ClassicKeyType, Key};

#[cfg(feature = "experimental")]
pub use self::key::{PostQuantumKey, PostQuantumKeyType};
