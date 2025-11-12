//! The base building blocks for working with ontology data.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

/// `Identified` is implemented by entities that have a [`TermId`] as an identifier.
///
/// ## Examples
///
/// [`crate::term::simple::SimpleMinimalTerm`] implements `Identified`.
/// ```
/// use ontolius::{Identified, TermId};
/// use ontolius::term::simple::SimpleMinimalTerm;
///
/// let term_id: TermId = "HP:1234567".parse().expect("CURIE should be valid");
/// let term = SimpleMinimalTerm::new(term_id, "Sample term", vec![], false);
///
/// assert_eq!(term.identifier().to_string(), "HP:1234567")
/// ```
pub trait Identified {
    fn identifier(&self) -> &TermId;
}

/// Identifier of an ontology concept.
///
/// ## Examples
///
/// Create a `TermId` from a `str` with compact URI (CURIE) or from a tuple consisting of *prefix* and *id* :
///
/// ```
/// use ontolius::TermId;
///
/// // Parse a CURIE `str`:
/// let a: TermId = "HP:0001250".parse().expect("value is a valid CURIE");
///
/// // Convert a tuple with `prefix` and `id`:
/// let b = TermId::from(("HP", "0001250"));
///
/// assert_eq!(a, b);
/// ```
///
///
/// ## Errors
///
/// Parsing a CURIE will fail if the CURIE does not contain either `:` or `_` as a delimiter:
///
/// ```
/// use ontolius::{TermId, TermIdParseError};
///
/// let term_id: Result<TermId, _> = "HP*0001250".parse(); // `*` is not valid delimiter
///
/// assert!(term_id.is_err());
/// assert_eq!(term_id.unwrap_err(), TermIdParseError::MissingDelimiter);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermId(InnerTermId);

/// Represents all possible reasons for failure to parse a CURIE into a [`TermId`].
#[derive(Debug, Clone, PartialEq)]
pub enum TermIdParseError {
    /// Missing colon (`:`) or underscore (`_`) in the input CURIE.
    MissingDelimiter,
}

impl Display for TermIdParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TermIdParseError::MissingDelimiter => write!(f, "Missing delimiter"),
        }
    }
}

impl Error for TermIdParseError {}

#[cfg(test)]
mod test_term_id_err {
    use super::TermIdParseError;

    #[test]
    fn term_id_err_can_be_converted_into_anyhow_error() {
        let e = anyhow::Error::from(TermIdParseError::MissingDelimiter);
        assert_eq!(e.to_string(), "Missing delimiter".to_string());
    }
}

/// Try to convert a CURIE `str` into a `TermId`.
///
/// ## Examples
///
/// ```
/// use ontolius::TermId;
///
/// let term_id: TermId = "HP:0001250".parse().expect("value is a valid CURIE");
///
/// assert_eq!(term_id.to_string(), "HP:0001250");
/// ```
impl FromStr for TermId {
    type Err = TermIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InnerTermId::try_from(s).map(TermId::from)
    }
}

/// Test if a tuple with *prefix* and *id* tuple is equal to a term ID.
///
/// ## Examples
///
/// ```
/// use ontolius::TermId;
///
/// assert_eq!(TermId::from(("HP", "0001250")), ("HP", "0001250"));
/// assert_eq!(TermId::from(("NCIT", "C2852")), ("NCIT", "C2852"));
/// ```
impl PartialEq<(&str, &str)> for TermId {
    fn eq(&self, other: &(&str, &str)) -> bool {
        match &self.0 {
            InnerTermId::Known(prefix, id, len) => {
                if prefix.eq(other.0) {
                    match (
                        other.1.parse::<u32>(),
                        u8::try_from(other.1.chars().count()),
                    ) {
                        (Ok(parsed_id), Ok(parsed_len)) => *id == parsed_id && *len == parsed_len,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            InnerTermId::Random(val, idx) => {
                /* Prefix */
                &val[..*idx as usize] == other.0
                /* Id */     && other.1 == &val[*idx as usize..]
            }
        }
    }
}

/// Test if a tuple with *prefix* and *id* tuple is equal to a reference to a term ID.
///
/// ## Examples
///
/// ```
/// use ontolius::TermId;
///
/// let seizure = TermId::from(("HP", "0001250"));
/// assert_eq!(&seizure, ("HP", "0001250"));
///
/// let adenocarcinoma = TermId::from(("NCIT", "C2852"));
/// assert_eq!(&adenocarcinoma, ("NCIT", "C2852"));
/// ```
impl PartialEq<(&str, &str)> for &TermId {
    fn eq(&self, other: &(&str, &str)) -> bool {
        (*self).eq(other)
    }
}

/// Convert a tuple with *prefix* and *id* into a `TermId`.
///
/// ## Examples
///
/// ```
/// use ontolius::TermId;
///
/// let term_id = TermId::from(("HP", "0001250"));
///
/// assert_eq!(term_id.to_string(), "HP:0001250");
/// ```
///
/// ## Panics
///
/// Conversion panics if *prefix* includes more than 255 characters ...
///
/// ```should_panic
/// # use ontolius::TermId;
///
/// // A string concatenated from 256 `'A'` characters
/// let many_as: String = std::iter::repeat('A').take(256).collect();
/// let term_id = TermId::from((many_as.as_str(), "0001250"));
/// ```
///
/// ... or if *id* includes more than 255 characters.
///
/// ```should_panic
/// # use ontolius::TermId;
///
/// // A string concatenated from 256 `'0'` characters
/// let many_zeros: String = std::iter::repeat('0').take(256).collect();
/// let term_id = TermId::from(("HP", many_zeros.as_str()));
/// ```
impl From<(&str, &str)> for TermId {
    fn from(value: (&str, &str)) -> Self {
        TermId::from(InnerTermId::from(value))
    }
}

impl From<InnerTermId> for TermId {
    fn from(value: InnerTermId) -> Self {
        TermId(value)
    }
}

impl TermId {
    pub(crate) const fn from_inner(inner: InnerTermId) -> Self {
        TermId(inner)
    }

    /// Get term id's prefix as [`Prefix`].
    ///
    /// ## Examples
    ///
    /// ```
    /// use ontolius::TermId;
    ///
    /// let term_id: TermId = "HP:0001250".parse().unwrap();
    /// let prefix = term_id.prefix();
    ///
    /// assert_eq!(&prefix, "HP");
    /// ```
    pub const fn prefix(&self) -> Prefix<'_> {
        Prefix(self)
    }
}

impl Display for TermId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The representation of the prefix of a [`TermId`].
///
/// ### Examples
///
/// Prefix can be obtained from a [`TermId`]:
///
/// ```
/// use ontolius::TermId;
///
/// let seizure: TermId = "HP:0001250".parse().unwrap();
/// let prefix = seizure.prefix();
/// ```
///
/// Prefix can be tested for equality with a `&str` or with another prefix.
///
/// ```
/// # use ontolius::TermId;
/// # let seizure: TermId = "HP:0001250".parse().unwrap();
/// # let prefix = seizure.prefix();
/// let arachnodactyly: TermId = "HP:0001166".parse().unwrap();
///
/// assert!(&prefix == "HP");
/// assert!(&arachnodactyly.prefix() == &prefix);
/// ```
///
/// Prefix implements [`PartialOrd`] and [`Ord`] traits.
/// Note that no *particular* order (e.g. alphabetical) is guaranteed.
/// Only that the ordering is defined.
///
/// Prefix implements [`std::fmt::Debug`] and [`std::fmt::Display`].
/// ```
/// # use ontolius::TermId;
/// # let seizure: TermId = "HP:0001250".parse().unwrap();
/// # let prefix = seizure.prefix();
/// assert_eq!(prefix.to_string(), String::from("HP"));
/// ```
#[derive(Clone, Debug, Eq, PartialOrd, Hash)]
pub struct Prefix<'a>(&'a TermId);

impl PartialEq for Prefix<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0 .0, &other.0 .0) {
            (InnerTermId::Known(l, _, _), InnerTermId::Known(r, _, _)) => l == r,
            (InnerTermId::Known(kp, _, _), InnerTermId::Random(val, offset)) => {
                kp == &val[..*offset as usize]
            }
            (InnerTermId::Random(val, offset), InnerTermId::Known(kp, _, _)) => {
                kp == &val[..*offset as usize]
            }
            (InnerTermId::Random(lval, loffset), InnerTermId::Random(rval, roffset)) => {
                lval[..*loffset as usize] == rval[..*roffset as usize]
            }
        }
    }
}

impl PartialEq<str> for Prefix<'_> {
    fn eq(&self, other: &str) -> bool {
        match &self.0 .0 {
            InnerTermId::Known(prefix, _, _) => prefix == other,
            InnerTermId::Random(val, offset) => &val[..(*offset as usize)] == other,
        }
    }
}

impl Ord for Prefix<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0 .0, &other.0 .0) {
            (InnerTermId::Known(l, _, _), InnerTermId::Known(r, _, _)) => l.cmp(r),
            (InnerTermId::Known(_, _, _), InnerTermId::Random(_, _)) => Ordering::Less,
            (InnerTermId::Random(_, _), InnerTermId::Known(_, _, _)) => Ordering::Greater,
            (InnerTermId::Random(lval, loffset), InnerTermId::Random(rval, roffset)) => {
                lval[..*loffset as usize].cmp(&rval[..*roffset as usize])
            }
        }
    }
}

impl Display for Prefix<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 .0 {
            InnerTermId::Known(prefix, _, _) => prefix.fmt(f),
            InnerTermId::Random(val, offset) => write!(f, "{}", &val[..*offset as usize]),
        }
    }
}

#[cfg(test)]
mod test_prefix {
    use super::TermId;

    #[test]
    fn partial_eq() {
        let seizure: TermId = "HP:0001250".parse().unwrap();
        let arachnodactyly: TermId = "HP:0001166".parse().unwrap();

        assert!(seizure.prefix() == arachnodactyly.prefix());
    }

    #[test]
    fn partial_eq_with_str() {
        let seizure: TermId = "HP:0001250".parse().unwrap();
        let prefix = seizure.prefix();

        assert!(&prefix == "HP");
    }

    #[test]
    fn display() {
        let seizure: TermId = "HP:0001250".parse().unwrap();

        assert_eq!(seizure.prefix().to_string().as_str(), "HP");
    }
}

// We really want to have all these private enum members in upper case!
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum KnownPrefix {
    // TODO: others?
    HP,
    OMIM,
    MONDO,
    GO,
    MAXO,
    ORPHA,
    GENO,
    SO,
    CHEBI,
    NCIT,
    PMID,
}

impl PartialEq<str> for KnownPrefix {
    fn eq(&self, other: &str) -> bool {
        match self {
            KnownPrefix::HP => other == "HP",
            KnownPrefix::OMIM => other == "OMIM",
            KnownPrefix::MONDO => other == "MONDO",
            KnownPrefix::GO => other == "GO",
            KnownPrefix::MAXO => other == "MAXO",
            KnownPrefix::ORPHA => other == "ORPHA",
            KnownPrefix::GENO => other == "GENO",
            KnownPrefix::SO => other == "SO",
            KnownPrefix::CHEBI => other == "CHEBI",
            KnownPrefix::NCIT => other == "NCIT",
            KnownPrefix::PMID => other == "PMID",
        }
    }
}

impl Display for KnownPrefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KnownPrefix::HP => f.write_str("HP"),
            KnownPrefix::OMIM => f.write_str("OMIM"),
            KnownPrefix::MONDO => f.write_str("MONDO"),
            KnownPrefix::GO => f.write_str("GO"),
            KnownPrefix::MAXO => f.write_str("MAXO"),
            KnownPrefix::ORPHA => f.write_str("ORPHA"),
            KnownPrefix::GENO => f.write_str("GENO"),
            KnownPrefix::SO => f.write_str("SO"),
            KnownPrefix::CHEBI => f.write_str("CHEBI"),
            KnownPrefix::NCIT => f.write_str("NCIT"),
            KnownPrefix::PMID => f.write_str("PMID"),
        }
    }
}

impl TryFrom<&str> for KnownPrefix {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO: this could arguably be improved!
        // We could also use a trie here..
        if value.starts_with("HP") {
            Ok(KnownPrefix::HP)
        } else if value.starts_with("OMIM") {
            Ok(KnownPrefix::OMIM)
        } else if value.starts_with("MONDO") {
            Ok(KnownPrefix::MONDO)
        } else if value.starts_with("GO") {
            Ok(KnownPrefix::GO)
        } else if value.starts_with("MAXO") {
            Ok(KnownPrefix::MAXO)
        } else if value.starts_with("ORPHA") {
            Ok(KnownPrefix::ORPHA)
        } else if value.starts_with("GENO") {
            Ok(KnownPrefix::GENO)
        } else if value.starts_with("SO") {
            Ok(KnownPrefix::SO)
        } else if value.starts_with("CHEBI") {
            Ok(KnownPrefix::CHEBI)
        } else if value.starts_with("NCIT") {
            Ok(KnownPrefix::NCIT)
        } else if value.starts_with("PMID") {
            Ok(KnownPrefix::PMID)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod test_known_prefix {
    use super::{KnownPrefix, TermId};

    #[test]
    fn partial_eq_with_str() {
        assert!(&KnownPrefix::HP == "HP");
        assert!(&KnownPrefix::HP != "SO");

        assert!(&KnownPrefix::SO == "SO");
        assert!(&KnownPrefix::PMID == "PMID");
    }

    #[test]
    fn partial_eq() {
        let seizure: TermId = "HP:0001250".parse().unwrap();
        let arachnodactyly: TermId = "HP:0001166".parse().unwrap();

        assert_eq!(seizure.prefix(), arachnodactyly.prefix());
    }

    #[test]
    fn display() {
        assert!(&KnownPrefix::HP.to_string() == "HP");
        assert!(&KnownPrefix::SO.to_string() == "SO");
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InnerTermId {
    // Most of the time we will have a CURIE that has a known Prefix and an integral id.
    // We store the prefix, the id, and the length of the id (e.g. 7 for HP:1234567 or 6 for OMIM:256000)
    Known(KnownPrefix, u32, u8),
    // Boxing the String to reduce the size because the Random variant is rare.
    // Size of `Random(Box<String>, u8)` is 16 while size of `Random(String, u8)` is 32.
    // Hence, disabling the `box_collection` lint below.
    #[allow(clippy::box_collection)]
    Random(Box<String>, u8),
}

impl InnerTermId {
    fn find_delimiter(curie: &str) -> Result<usize, TermIdParseError> {
        if let Some(idx) = curie.find(':') {
            Ok(idx)
        } else if let Some(idx) = curie.find('_') {
            Ok(idx)
        } else {
            Err(TermIdParseError::MissingDelimiter)
        }
    }
}

impl TryFrom<&str> for InnerTermId {
    type Error = TermIdParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let delimiter = InnerTermId::find_delimiter(value)?;
        Ok(InnerTermId::from((
            &value[..delimiter],
            &value[delimiter + 1..],
        )))
    }
}

impl From<(&str, &str)> for InnerTermId {
    fn from(value: (&str, &str)) -> Self {
        let (prefix, ident) = value;
        let p = KnownPrefix::try_from(prefix);
        let a: Result<u32, _> = ident.parse();
        let id_len: Result<_, _> = u8::try_from(ident.len());
        if let (Ok(prefix), Ok(id)) = (p, a) {
            InnerTermId::Known(
                // Prefix is known
                prefix,
                id,
                id_len.expect("ID should not be longer than 255 chars!"),
            )
        } else {
            // Unknown prefix, we must allocate the data into a `String`.
            let val = Box::new([prefix, ident].concat());
            let idx = u8::try_from(prefix.chars().count())
                .expect("Curie prefix should not be longer than 255 chars!");
            InnerTermId::Random(val, idx)
        }
    }
}

impl Display for InnerTermId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerTermId::Known(prefix, id, padding) => {
                // `n` for dynamic padding of `id` to "`padding`" length.
                write!(f, "{prefix}:{id:0>n$}", n = *padding as usize)
            }
            InnerTermId::Random(val, delimiter) => {
                let delim = *delimiter as usize;
                write!(
                    f,
                    "{prefix}:{id}",
                    prefix = &val[..delim],
                    id = &val[delim..]
                )
            }
        }
    }
}

impl PartialEq<Self> for InnerTermId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Known(l_prefix, l_id, _), Self::Known(r_prefix, r_id, _)) => {
                l_prefix == r_prefix && l_id == r_id
            }
            (Self::Random(l_val, _), Self::Random(r_val, _)) => l_val == r_val,
            _ => false,
        }
    }
}

impl Eq for InnerTermId {}

impl std::cmp::PartialOrd for InnerTermId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for InnerTermId {
    fn cmp(&self, other: &Self) -> Ordering {
        if std::mem::discriminant(self) == std::mem::discriminant(other) {
            // Comparing the same enum variants.
            match (self, other) {
                (InnerTermId::Known(l_prefix, l_id, _), InnerTermId::Known(r_prefix, r_id, _)) => {
                    match l_prefix.cmp(r_prefix) {
                        Ordering::Less => Ordering::Less,
                        Ordering::Equal => l_id.cmp(r_id),
                        Ordering::Greater => Ordering::Greater,
                    }
                }
                (InnerTermId::Random(l_val, _), InnerTermId::Random(r_val, _)) => l_val.cmp(r_val),
                _ => unreachable!("Enum discriminants were equal but the enum variants were not!"),
            }
        } else {
            match self {
                // `Known`` is always less than `Random``.
                InnerTermId::Known(_, _, _) => Ordering::Less,
                InnerTermId::Random(_, _) => Ordering::Greater,
            }
        }
    }
}

impl std::hash::Hash for InnerTermId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            InnerTermId::Known(prefix, id, _) => {
                prefix.hash(state);
                id.hash(state);
            }
            InnerTermId::Random(val, _) => {
                val.hash(state);
            }
        }
    }
}

impl Identified for TermId {
    fn identifier(&self) -> &TermId {
        self
    }
}

#[cfg(test)]
mod test_creation {

    use super::TermId;

    #[test]
    fn test_term_id_from_tuple() {
        macro_rules! round_trip_from_tuple {
            ($vals: expr, $expected: literal) => {
                let term_id = TermId::from($vals);
                assert_eq!(term_id.to_string(), $expected);
            };
        }

        round_trip_from_tuple!(("HP", "1"), "HP:1");
        round_trip_from_tuple!(("MONDO", "123456"), "MONDO:123456");
    }

    #[test]
    fn test_term_id_from_curie() {
        macro_rules! round_trip_from_curie {
            ($val: literal, $expected: literal) => {
                let term_id: Result<TermId, _> = $val.parse();
                assert!(term_id.is_ok());

                let term_id = term_id.unwrap();
                assert_eq!(term_id.to_string(), $expected);
            };
        }

        round_trip_from_curie!("HP:123456", "HP:123456");
        round_trip_from_curie!("HP_123456", "HP:123456");
        round_trip_from_curie!("MONDO:123456", "MONDO:123456");
        round_trip_from_curie!("OMIM:256000", "OMIM:256000");
        round_trip_from_curie!("NCIT_C2852", "NCIT:C2852");
        round_trip_from_curie!("SNOMEDCT_US:139394000", "SNOMEDCT_US:139394000");
        round_trip_from_curie!("SNOMEDCT_US:449491000124101", "SNOMEDCT_US:449491000124101");
        round_trip_from_curie!("UMLS:C0028734", "UMLS:C0028734");
        round_trip_from_curie!("WHATEVER:12", "WHATEVER:12");
    }
}

#[cfg(test)]
mod test_comparison_and_ordering {
    use std::cmp::Ordering;
    use std::str::FromStr;

    use super::TermId;

    macro_rules! term_ids_compare_to_ordering {
        ($left_curie: literal, $right_curie: literal, $val: expr) => {
            let left = TermId::from_str($left_curie).expect("Left CURIE is invalid!");
            let right = TermId::from_str($right_curie).expect("Right CURIE is invalid!");
            assert_eq!(left.cmp(&right), $val);
        };
    }

    #[test]
    fn known_vs_random() {
        term_ids_compare_to_ordering!("HP:1234567", "WHATEVER:1234567", Ordering::Less);
        term_ids_compare_to_ordering!("WHATEVER:1234567", "HP:1234567", Ordering::Greater);
    }

    #[test]
    fn known() {
        term_ids_compare_to_ordering!("HP:1", "HP:1", Ordering::Equal);
        term_ids_compare_to_ordering!("HP:0", "HP:1", Ordering::Less);
        term_ids_compare_to_ordering!("HP:2", "HP:1", Ordering::Greater);
        term_ids_compare_to_ordering!("HP:10", "HP:1", Ordering::Greater);
    }

    #[test]
    fn known_other_prefixes() {
        term_ids_compare_to_ordering!("HP:1", "HP_1", Ordering::Equal);
        term_ids_compare_to_ordering!("HP:0", "HP_1", Ordering::Less);
        term_ids_compare_to_ordering!("HP:2", "HP_1", Ordering::Greater);
        term_ids_compare_to_ordering!("HP:10", "HP_1", Ordering::Greater);
    }

    #[test]
    fn random() {
        term_ids_compare_to_ordering!("WHATEVER:1", "WHATEVER_1", Ordering::Equal);
        term_ids_compare_to_ordering!("WHATEVER:0", "WHATEVER:1", Ordering::Less);
        term_ids_compare_to_ordering!("WHATEVER:2", "WHATEVER:1", Ordering::Greater);
        term_ids_compare_to_ordering!("WHATEVER:10", "WHATEVER:1", Ordering::Greater);
    }
}

#[cfg(test)]
mod test_equalities {

    use super::*;

    macro_rules! term_ids_partial_eq {
        ($curie: literal, $other: expr, $val: literal) => {
            let term_id = TermId::from_str($curie).expect("CURIE is invalid!");
            if $val {
                assert_eq!(term_id, $other);
            } else {
                assert_ne!(term_id, $other);
            }
        };
    }

    #[test]
    fn test_partial_eq_to_str_tuple() {
        term_ids_partial_eq!("HP:1234567", ("HP", "1234567"), true);
        term_ids_partial_eq!("HP:0001250", ("HP", "0001250"), true);
        term_ids_partial_eq!("NCIT:C2852", ("NCIT", "C2852"), true);
        term_ids_partial_eq!("HP:0000000", ("HP", "0000000"), true);

        term_ids_partial_eq!("HP:0001250", ("MP", "0001250"), false);
        term_ids_partial_eq!("HP:1234567", ("HP", "0000000"), false);
        term_ids_partial_eq!("NCIT:C2852", ("HP", "0001250"), false);
    }
}

#[cfg(test)]
mod test_sizes {

    use std::mem::size_of;

    use super::InnerTermId;

    #[test]
    fn test_size_of_inner_term_id() {
        assert_eq!(size_of::<InnerTermId>(), 16);
    }
}
