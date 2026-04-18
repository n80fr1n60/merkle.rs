#[allow(missing_debug_implementations)]
mod proof;

use ring::digest::Algorithm;

use protobuf::Message;

use crate::proof::{Lemma, Positioned, Proof};

pub use self::proof::{LemmaProto, ProofProto};

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
    pub fn from_proof<T>(proof: Proof<T>) -> Self
    where
        T: Into<Vec<u8>>,
    {
        let mut proto = Self::new();

        match proof {
            Proof {
                root_hash,
                lemma,
                value,
                ..
            } => {
                proto.root_hash = root_hash;
                proto.lemma = protobuf::MessageField::some(LemmaProto::from_lemma(lemma));
                proto.value = value.into();
            }
        }

        proto
    }

    pub fn into_proof<T>(self, algorithm: &'static Algorithm) -> Option<Proof<T>>
    where
        T: From<Vec<u8>>,
    {
        if self.root_hash.is_empty() || self.lemma.is_none() {
            return None;
        }

        self.lemma.into_option().and_then(|lemma| {
            lemma.into_lemma().map(|lemma| {
                Proof::new(
                    algorithm,
                    self.root_hash,
                    lemma,
                    self.value.into(),
                )
            })
        })
    }
}

impl LemmaProto {
    pub fn from_lemma(lemma: Lemma) -> Self {
        let mut proto = Self::new();

        match lemma {
            Lemma {
                node_hash,
                sibling_hash,
                sub_lemma,
            } => {
                proto.node_hash = node_hash;

                if let Some(sub_proto) = sub_lemma.map(|l| Self::from_lemma(*l)) {
                    proto.sub_lemma = protobuf::MessageField::some(sub_proto);
                }

                match sibling_hash {
                    Some(Positioned::Left(hash)) => proto.left_sibling_hash = hash,
                    Some(Positioned::Right(hash)) => proto.right_sibling_hash = hash,
                    None => {}
                }
            }
        }

        proto
    }

    pub fn into_lemma(self) -> Option<Lemma> {
        if self.node_hash.is_empty() {
            return None;
        }

        let node_hash = self.node_hash;

        let sibling_hash = if !self.left_sibling_hash.is_empty() {
            Some(Positioned::Left(self.left_sibling_hash))
        } else if !self.right_sibling_hash.is_empty() {
            Some(Positioned::Right(self.right_sibling_hash))
        } else {
            None
        };

        if self.sub_lemma.is_some() {
            self.sub_lemma.into_option().and_then(|sub| {
                sub.into_lemma().map(|sub_lemma| Lemma {
                    node_hash,
                    sibling_hash,
                    sub_lemma: Some(Box::new(sub_lemma)),
                })
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
