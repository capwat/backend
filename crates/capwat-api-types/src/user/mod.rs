pub mod salt;
pub use self::salt::*;

use crate::encrypt::ClassicKey;
#[cfg(feature = "experimental")]
use crate::encrypt::PostQuantumKey;

use crate::util::{EncodedBase64, Sensitive};

#[cfg(feature = "server")]
use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "server", derive(Builder))]
pub struct UserClassicKeys {
    #[cfg_attr(feature = "server", builder(into))]
    pub public: Sensitive<ClassicKey>,
    #[cfg_attr(feature = "server", builder(into))]
    pub encrypted_private: Sensitive<EncodedBase64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg(feature = "experimental")]
#[cfg_attr(feature = "server", derive(Builder))]
pub struct UserPostQuantumKeys {
    #[cfg_attr(feature = "server", builder(into))]
    pub public: Sensitive<PostQuantumKey>,
    #[cfg_attr(feature = "server", builder(into))]
    pub encrypted_private: Sensitive<EncodedBase64>,
}
