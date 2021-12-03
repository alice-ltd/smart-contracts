use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{
    Addr, Api, Binary, CanonicalAddr, RecoverPubkeyError, StdError, StdResult, VerificationError,
};

/// Mock Api that uses bech32 for canonical/human addresses
/// Used for testing relay (meta-transactions)
#[derive(Copy, Clone)]
pub struct MockBech32Api {
    base: MockApi,
}

impl MockBech32Api {
    pub fn new() -> Self {
        MockBech32Api {
            base: MockApi::default(),
        }
    }
}

impl Default for MockBech32Api {
    fn default() -> Self {
        Self::new()
    }
}

impl Api for MockBech32Api {
    fn addr_validate(&self, human: &str) -> StdResult<Addr> {
        self.addr_canonicalize(human).map(|_canonical| ())?;
        Ok(Addr::unchecked(human))
    }

    fn addr_canonicalize(&self, human: &str) -> StdResult<CanonicalAddr> {
        use bech32::FromBase32;
        let (_hrp, data, _variant) = bech32::decode(human)
            .map_err(|_| StdError::generic_err("addr_canonicalize bech32 decode error"))?;
        let out = Vec::<u8>::from_base32(&data).unwrap();
        Ok(CanonicalAddr(Binary(out)))
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        use bech32::{ToBase32, Variant};
        let data = canonical.as_slice().to_base32();
        let encoded = bech32::encode("terra", data, Variant::Bech32)
            .map_err(|_| StdError::generic_err("addr_humanize bech32 encode error"))?;
        Ok(Addr::unchecked(encoded))
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        self.base
            .secp256k1_verify(message_hash, signature, public_key)
    }

    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        self.base
            .secp256k1_recover_pubkey(message_hash, signature, recovery_param)
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        self.base.ed25519_verify(message, signature, public_key)
    }

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        self.base
            .ed25519_batch_verify(messages, signatures, public_keys)
    }

    fn debug(&self, message: &str) {
        self.base.debug(message)
    }
}
