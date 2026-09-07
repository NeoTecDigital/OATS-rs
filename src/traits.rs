use crate::objects::ObjectId;
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Trait identifier
pub type TraitId = Uuid;

/// A trait represents immutable domain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    /// Unique identifier for this trait
    pub id: TraitId,
    /// Name of the trait
    pub name: String,
    /// Version of the trait
    pub version: u32,
    /// The actual trait data
    pub data: TraitData,
    /// Metadata about the trait
    pub metadata: HashMap<String, String>,
}

/// The actual data contained in a trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraitData {
    /// Simple string value
    String(String),
    /// Approximate real value.
    ///
    /// Binary floating point: use it for measurements and ratios. Never use it
    /// for money or for any quantity that must round-trip exactly - use
    /// [`TraitData::Decimal`] or [`TraitData::Integer`] instead.
    Number(f64),
    /// Signed whole number (`<int>`): counts, quantities, identifiers
    Integer(i64),
    /// Exact decimal (`<decimal>`): money, rates, anything that must not drift
    Decimal(Decimal),
    /// Absolute point in time in UTC (`<timestamp>`)
    Timestamp(DateTime<Utc>),
    /// Calendar date with no time and no zone (`<date>`)
    Date(NaiveDate),
    /// Elapsed time (`<duration>`): a **signed** nanosecond quantity.
    ///
    /// Signed, and `chrono::TimeDelta` rather than `std::time::Duration`, because a
    /// duration here is as often a difference as a length — a residual between what was
    /// planned and what happened runs in both directions, and an unsigned type would make
    /// half of those unrepresentable. It also matches Go's `time.Duration`, which is a
    /// signed `int64` of nanoseconds; the unsigned type would have meant the two languages
    /// disagreeing about the domain of the same declared type.
    Duration(TimeDelta),
    /// Boolean value
    Boolean(bool),
    /// Complex structured data
    Object(HashMap<String, serde_json::Value>),
    /// Array of values
    Array(Vec<serde_json::Value>),
    /// Binary data
    Binary(Vec<u8>),
    /// Reference to another object.
    ///
    /// Vocabularies are full of `<kind>` references. Without this variant a
    /// reference has to degrade to a `String`, which loses the fact that it is
    /// a reference and cannot be resolved against a registry.
    Ref(ObjectId),
}

impl Trait {
    /// Create a new trait with the given name and data
    #[inline]
    pub fn new(name: impl Into<String>, data: TraitData) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            data,
            metadata: HashMap::new(),
        }
    }

    /// Create a new trait with metadata
    pub fn with_metadata(
        name: impl Into<String>,
        data: TraitData,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            data,
            metadata,
        }
    }

    /// Create a new trait with pre-allocated capacity
    pub fn with_capacity(
        name: impl Into<String>,
        data: TraitData,
        metadata_capacity: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: 1,
            data,
            metadata: HashMap::with_capacity(metadata_capacity),
        }
    }

    /// Get the trait name
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the trait data
    #[inline]
    pub fn data(&self) -> &TraitData {
        &self.data
    }

    /// Get mutable trait data
    #[inline]
    pub fn data_mut(&mut self) -> &mut TraitData {
        &mut self.data
    }

    /// Get the trait ID
    #[inline]
    pub fn id(&self) -> TraitId {
        self.id
    }

    /// Get the trait version
    #[inline]
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get a metadata value
    #[inline]
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set a metadata value
    #[inline]
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Create a new version of this trait
    pub fn new_version(&self, data: TraitData) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            version: self.version + 1,
            data,
            metadata: self.metadata.clone(),
        }
    }

    /// Get metadata count
    #[inline]
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Reserve capacity for metadata
    #[inline]
    pub fn reserve_metadata(&mut self, additional: usize) {
        self.metadata.reserve(additional);
    }

    /// Clear all metadata
    #[inline]
    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
    }
}

impl TraitData {
    /// Check if this trait data is a string
    pub fn is_string(&self) -> bool {
        matches!(self, TraitData::String(_))
    }

    /// Check if this trait data is an approximate real
    pub fn is_number(&self) -> bool {
        matches!(self, TraitData::Number(_))
    }

    /// Check if this trait data is a whole number
    pub fn is_integer(&self) -> bool {
        matches!(self, TraitData::Integer(_))
    }

    /// Check if this trait data is an exact decimal
    pub fn is_decimal(&self) -> bool {
        matches!(self, TraitData::Decimal(_))
    }

    /// Check if this trait data is a timestamp
    pub fn is_timestamp(&self) -> bool {
        matches!(self, TraitData::Timestamp(_))
    }

    /// Check if this trait data is a calendar date
    pub fn is_date(&self) -> bool {
        matches!(self, TraitData::Date(_))
    }

    /// Check if this trait data is an elapsed time
    pub fn is_duration(&self) -> bool {
        matches!(self, TraitData::Duration(_))
    }

    /// Check if this trait data is a boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, TraitData::Boolean(_))
    }

    /// Check if this trait data is an object
    pub fn is_object(&self) -> bool {
        matches!(self, TraitData::Object(_))
    }

    /// Check if this trait data is an array
    pub fn is_array(&self) -> bool {
        matches!(self, TraitData::Array(_))
    }

    /// Check if this trait data is binary
    pub fn is_binary(&self) -> bool {
        matches!(self, TraitData::Binary(_))
    }

    /// Check if this trait data is a reference to another object
    pub fn is_ref(&self) -> bool {
        matches!(self, TraitData::Ref(_))
    }

    /// Try to get the string value
    pub fn as_string(&self) -> Option<&String> {
        match self {
            TraitData::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the approximate real value.
    ///
    /// Deliberately strict: an [`TraitData::Integer`] or [`TraitData::Decimal`]
    /// is not silently widened to `f64`, because that is exactly how exact
    /// values lose their exactness.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            TraitData::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get the whole number value
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            TraitData::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get the exact decimal value
    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            TraitData::Decimal(d) => Some(*d),
            _ => None,
        }
    }

    /// Try to get the timestamp value
    pub fn as_timestamp(&self) -> Option<DateTime<Utc>> {
        match self {
            TraitData::Timestamp(t) => Some(*t),
            _ => None,
        }
    }

    /// Try to get the calendar date value
    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            TraitData::Date(d) => Some(*d),
            _ => None,
        }
    }

    /// Try to get the elapsed time
    pub fn as_duration(&self) -> Option<TimeDelta> {
        match self {
            TraitData::Duration(d) => Some(*d),
            _ => None,
        }
    }

    /// Try to get the boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            TraitData::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get the object value
    pub fn as_object(&self) -> Option<&HashMap<String, serde_json::Value>> {
        match self {
            TraitData::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Try to get the array value
    pub fn as_array(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            TraitData::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get the binary value
    pub fn as_binary(&self) -> Option<&Vec<u8>> {
        match self {
            TraitData::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// Try to get the referenced object id
    pub fn as_ref_id(&self) -> Option<ObjectId> {
        match self {
            TraitData::Ref(id) => Some(*id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_creation() {
        let trait_data = TraitData::String("test".to_string());
        let trait_obj = Trait::new("test_trait", trait_data);

        assert_eq!(trait_obj.name(), "test_trait");
        assert_eq!(trait_obj.version, 1);
        assert!(trait_obj.data.is_string());
    }

    #[test]
    fn test_trait_data_methods() {
        let string_data = TraitData::String("hello".to_string());
        let number_data = TraitData::Number(42.0);
        let bool_data = TraitData::Boolean(true);

        assert!(string_data.is_string());
        assert!(number_data.is_number());
        assert!(bool_data.is_boolean());

        assert_eq!(string_data.as_string(), Some(&"hello".to_string()));
        assert_eq!(number_data.as_number(), Some(42.0));
        assert_eq!(bool_data.as_boolean(), Some(true));
    }

    #[test]
    fn duration_is_signed_and_round_trips_exactly() {
        let ahead = TraitData::Duration(TimeDelta::nanoseconds(1_500_000_000));
        assert!(ahead.is_duration());
        assert_eq!(
            ahead.as_duration().and_then(|d| d.num_nanoseconds()),
            Some(1_500_000_000)
        );

        // The reason it is not `std::time::Duration`: a residual runs both ways, and an
        // unsigned type would make half of every variance unrepresentable.
        let behind = TraitData::Duration(TimeDelta::nanoseconds(-250));
        assert_eq!(
            behind.as_duration().and_then(|d| d.num_nanoseconds()),
            Some(-250)
        );

        // Nanosecond precision survives serde, which is what the canonical form encodes.
        let text = serde_json::to_string(&ahead).unwrap();
        let back: TraitData = serde_json::from_str(&text).unwrap();
        assert_eq!(
            back.as_duration().and_then(|d| d.num_nanoseconds()),
            Some(1_500_000_000)
        );

        assert_eq!(ahead.as_integer(), None, "it is not an integer wearing a name");
    }

    #[test]
    fn test_numeric_variants_are_distinct() {
        let integer = TraitData::Integer(42);
        let decimal = TraitData::Decimal(Decimal::new(4200, 2));
        let real = TraitData::Number(42.0);

        assert!(integer.is_integer() && !integer.is_number() && !integer.is_decimal());
        assert!(decimal.is_decimal() && !decimal.is_number() && !decimal.is_integer());
        assert!(real.is_number() && !real.is_integer() && !real.is_decimal());

        assert_eq!(integer.as_integer(), Some(42));
        assert_eq!(decimal.as_decimal(), Some(Decimal::new(4200, 2)));
        assert_eq!(real.as_number(), Some(42.0));

        // No silent widening: that is how exactness gets lost.
        assert_eq!(integer.as_number(), None);
        assert_eq!(decimal.as_number(), None);
        assert_eq!(real.as_decimal(), None);
    }

    #[test]
    fn test_decimal_money_does_not_drift() {
        let charge = Decimal::new(9999, 2);
        let mut exact = Decimal::new(50000, 2);
        let mut approximate = 500.0_f64;

        for _ in 0..3 {
            exact -= charge;
            approximate -= 99.99;
        }

        assert_eq!(exact, Decimal::new(20003, 2));
        assert_eq!(exact.to_string(), "200.03");
        assert_ne!(
            approximate, 200.03,
            "f64 drifts, which is why money is not f64"
        );
    }

    #[test]
    fn test_temporal_variants_round_trip() {
        let instant = DateTime::parse_from_rfc3339("2026-08-30T12:34:56Z")
            .expect("valid timestamp")
            .with_timezone(&Utc);
        let day = NaiveDate::from_ymd_opt(2026, 8, 30).expect("valid date");

        let timestamp = Trait::new("signed_at", TraitData::Timestamp(instant));
        let date = Trait::new("effective_on", TraitData::Date(day));

        let timestamp: Trait =
            serde_json::from_str(&serde_json::to_string(&timestamp).expect("serialize"))
                .expect("deserialize");
        let date: Trait = serde_json::from_str(&serde_json::to_string(&date).expect("serialize"))
            .expect("deserialize");

        assert_eq!(timestamp.data().as_timestamp(), Some(instant));
        assert_eq!(date.data().as_date(), Some(day));
        assert_eq!(timestamp.data().as_date(), None);
        assert_eq!(date.data().as_timestamp(), None);
    }

    #[test]
    fn test_decimal_round_trips() {
        let amount = Decimal::new(99999, 2);
        let trait_obj = Trait::new("price", TraitData::Decimal(amount));

        let json = serde_json::to_string(&trait_obj).expect("serialize");
        let restored: Trait = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.data().as_decimal(), Some(amount));
        assert_eq!(restored.data().as_decimal().unwrap().to_string(), "999.99");
    }

    #[test]
    fn test_trait_data_ref() {
        use crate::objects::Object;

        let supplier = Object::new("acme_supply", "supplier");
        let reference = TraitData::Ref(supplier.id());

        assert!(reference.is_ref());
        assert!(!reference.is_string());
        assert_eq!(reference.as_ref_id(), Some(supplier.id()));
        assert_eq!(reference.as_string(), None);

        // A reference must stay a reference, not degrade to a string.
        let as_string = TraitData::String(supplier.id().to_string());
        assert!(!as_string.is_ref());
        assert_eq!(as_string.as_ref_id(), None);
    }

    #[test]
    fn test_trait_data_ref_round_trips() {
        use crate::objects::Object;

        let supplier = Object::new("acme_supply", "supplier");
        let trait_obj = Trait::new("supplier", TraitData::Ref(supplier.id()));

        let json = serde_json::to_string(&trait_obj).expect("serialize");
        let restored: Trait = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.data().as_ref_id(), Some(supplier.id()));
    }

    #[test]
    fn test_trait_metadata() {
        let mut trait_obj = Trait::new("test", TraitData::String("value".to_string()));
        trait_obj.set_metadata("key", "value");

        assert_eq!(trait_obj.get_metadata("key"), Some(&"value".to_string()));
        assert_eq!(trait_obj.get_metadata("nonexistent"), None);
    }
}
