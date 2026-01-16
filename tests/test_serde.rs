/// An example of serializing a TermId as a CURIE.
///
/// When working with types that use [`ontolius::TermId`]s and their serialization,
/// we would like to derive both [`serde::Serialize`] and [`serde::Deserialize`] traits
/// to enable interoperability with the `serde` crate.
/// However, we cannot do this directly since [`ontolius::TermId`] does not implement the traits.
///
/// As a workaround, `serde`` allows using custom serialization and deserialization functions
/// and `ontolius` provides functions to use with (de)serialization.
///
/// The functions are available on [`ontolius::TermId`] when the `serde` feature is enabled.
/// An example usage is shown in this module.
#[cfg(feature = "serde")]
mod test_serde {

    use serde;
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    use ontolius::TermId;

    /// An example struct that we want to serialize and deserialize with serde.
    ///
    /// Use the `serde` attributes on the `term_id` field
    /// to serialize the `TermId` as CURIE.
    #[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
    struct Feature {
        #[serde(
            serialize_with = "TermId::serialize_as_curie",
            deserialize_with = "TermId::deserialize_from_curie"
        )]
        term_id: TermId,
    }

    /// Test that serializing `Feature` produces the expected Serde tokens and that
    /// a valid `Feature` can be created by deserializing those tokens.
    #[test]
    fn test_serialize() {
        let feature = Feature {
            term_id: TermId::from(("HP", "0001250")),
        };

        assert_tokens(
            &feature,
            &[
                Token::Struct {
                    name: "Feature",
                    len: 1,
                },
                Token::Str("term_id"),
                Token::Str("HP:0001250"),
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn test_malformed_curie_produces_an_error() {
        let tokens = [
            Token::Struct {
                name: "Feature",
                len: 1,
            },
            Token::Str("term_id"),
            Token::Str("INVALID"),
            Token::StructEnd,
        ];
        assert_de_tokens_error::<Feature>(
            &tokens,
            "invalid value: string \"INVALID\", expected a curie (e.g. \"HP:0001250\")",
        );
    }
}
