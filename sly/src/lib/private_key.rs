use std::fs;
use std::io::Read;

use ic_agent::identity::{BasicIdentity, Secp256k1Identity};
use ic_agent::Identity;
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::Private;
use pem::Pem;
use ring::signature::Ed25519KeyPair;

/// A key that can be converted to a Identity.
#[derive(Clone)]
pub enum PrivateKey {
    Ed25519Pkcs8(Vec<u8>),
    Secp256k1(EcKey<Private>),
}

impl PrivateKey {
    /// Try to load a pem file.
    pub fn from_pem_file<P: AsRef<std::path::Path>>(file_path: P) -> anyhow::Result<Self> {
        let reader = fs::File::open(file_path)?;
        let pem = reader
            .bytes()
            .collect::<Result<Vec<u8>, std::io::Error>>()?;

        if let Ok(private_key) = EcKey::private_key_from_pem(&pem) {
            log::trace!("Valid secp256k1.");
            return Ok(Self::Secp256k1(private_key));
        }

        log::trace!("Trying to parse as Ed25519 keypair");
        let pkcs8 = pem::parse(&pem)?.contents;
        // Validation step.
        Ed25519KeyPair::from_pkcs8(pkcs8.as_slice())?;

        Ok(Self::Ed25519Pkcs8(pkcs8))
    }

    pub fn generate() -> Self {
        let group = EcGroup::from_curve_name(Nid::SECP256K1).expect("Cannot create EcGroup.");
        let private_key = EcKey::generate(&group).unwrap();
        Self::Secp256k1(private_key)
    }

    /// Store the PEM file to the given location.
    pub fn store_pem_file<P: AsRef<std::path::Path>>(
        &self,
        name: &str,
        path: P,
    ) -> anyhow::Result<()> {
        match &self {
            PrivateKey::Ed25519Pkcs8(pkcs8) => {
                log::trace!("Storing Ed25519 pem file.");
                let pem = Pem {
                    tag: name.into(),
                    contents: pkcs8.clone(),
                };

                let contents = pem::encode(&pem);
                fs::write(path, contents)?;
                Ok(())
            }
            PrivateKey::Secp256k1(key) => {
                log::trace!("Storing Secp256k1 pem file.");
                let contents = key.private_key_to_pem()?;
                fs::write(path, contents)?;
                Ok(())
            }
        }
    }

    /// Convert to an identity.
    pub fn into_identity(self) -> Box<dyn Identity> {
        match self {
            PrivateKey::Ed25519Pkcs8(pkcs8) => {
                let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_slice()).unwrap();
                Box::new(BasicIdentity::from_key_pair(key_pair))
            }
            PrivateKey::Secp256k1(private_key) => {
                Box::new(Secp256k1Identity::from_private_key(private_key))
            }
        }
    }
}
