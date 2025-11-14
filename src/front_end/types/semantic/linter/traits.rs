use super::types::ThrushAttributeComparator;

pub trait ThrushAttributeComparatorExtensions {
    fn into_attr_cmp(&self) -> ThrushAttributeComparator;
}
