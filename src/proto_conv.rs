#![allow(missing_debug_implementations)]

use ring::digest::Algorithm;

use protobuf::Message;

use crate::proof::{Lemma, Positioned, Proof};
use crate::proto::proof::{LemmaProto, ProofProto};

impl<T> Proof<T> {
    /// Constructs a `Proof` struct from its Protobuf representation.
    pub fn from_protobuf(algorithm: &'static Algorithm, proto: ProofProto) -> Option<Self>
    where
        T: From<Vec<u8>>,
    {
        proto.into_proof(algorithm)
    }

    /// Encode this `Proof` to its Protobuf representation.
    pub fn into_protobuf(self) -> ProofProto
    where
        T: Into<Vec<u8>>,
    {
        ProofProto::from_proof(self)
    }

    /// Parse a `Proof` from its Protobuf binary representation.
    pub fn parse_from_bytes(
        bytes: &[u8],
        algorithm: &'static Algorithm,
    ) -> protobuf::Result<Option<Self>>
    where
        T: From<Vec<u8>>,
    {
        ProofProto::parse_from_bytes(bytes).map(|proto| proto.into_proof(algorithm))
    }

    /// Serialize this `Proof` with Protobuf.
    pub fn write_to_bytes(self) -> protobuf::Result<Vec<u8>>
    where
        T: Into<Vec<u8>>,
    {
        self.into_protobuf().write_to_bytes()
    }
}

impl ProofProto {
    fn from_proof<T>(proof: Proof<T>) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let mut proto = Self::new();
        proto.root_hash = proof.root_hash;
        proto.lemma = protobuf::MessageField::some(LemmaProto::from_lemma(proof.lemma));
        proto.value = proof.value.into();
        proto
    }

    fn into_proof<T>(mut self, algorithm: &'static Algorithm) -> Option<Proof<T>>
    where
        T: From<Vec<u8>>,
    {
        if self.root_hash.is_empty() || self.lemma.is_none() {
            return None;
        }

        let lemma = self.lemma.take()?.into_lemma()?;
        Some(Proof::new(
            algorithm,
            std::mem::take(&mut self.root_hash),
            lemma,
            std::mem::take(&mut self.value).into(),
        ))
    }
}

impl LemmaProto {
    fn from_lemma(lemma: Lemma) -> Self {
        let mut proto = Self::new();
        proto.node_hash = lemma.node_hash;

        if let Some(sub) = lemma.sub_lemma {
            proto.sub_lemma = protobuf::MessageField::some(Self::from_lemma(*sub));
        }

        match lemma.sibling_hash {
            Some(Positioned::Left(hash)) => proto.set_left_sibling_hash(hash),
            Some(Positioned::Right(hash)) => proto.set_right_sibling_hash(hash),
            None => {}
        }

        proto
    }

    fn into_lemma(mut self) -> Option<Lemma> {
        if self.node_hash.is_empty() {
            return None;
        }

        let node_hash = std::mem::take(&mut self.node_hash);

        let sibling_hash = if self.has_left_sibling_hash() {
            Some(Positioned::Left(self.take_left_sibling_hash()))
        } else if self.has_right_sibling_hash() {
            Some(Positioned::Right(self.take_right_sibling_hash()))
        } else {
            None
        };

        if self.sub_lemma.is_some() {
            self.sub_lemma.take()?.into_lemma().map(|sub_lemma| Lemma {
                node_hash,
                sibling_hash,
                sub_lemma: Some(Box::new(sub_lemma)),
            })
        } else {
            Some(Lemma {
                node_hash,
                sibling_hash,
                sub_lemma: None,
            })
        }
    }
}
