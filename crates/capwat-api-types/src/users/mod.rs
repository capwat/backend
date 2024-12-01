mod salt;
pub use self::salt::UserSalt;

use crate::e2ee::{ClassicKey, PostQuantumKey};
use crate::util::Sensitive;

use bon::Builder;
use serde::{Deserialize, Serialize};

/// A full metadata of user's classic cryptographic keys.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Builder)]
pub struct UserClassicKeys {
    #[builder(into)]
    pub public: Sensitive<ClassicKey>,
    pub encrypted_private: Sensitive<String>,
}

/// A full metadata of user's post-quantum cryptographic keys.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Builder)]
pub struct UserPostQuantumKeys {
    #[builder(into)]
    pub public: Sensitive<PostQuantumKey>,
    pub encrypted_private: Sensitive<String>,
}
