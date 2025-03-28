use std::collections::{HashMap, hash_map};
use std::sync::Arc;

use serde::Serialize;
use serde::ser::SerializeMap;

use super::menu::UnknownTag;
use super::{TagData, TagValue};

/// Set of tags and associated values for an object in the catalog.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TagSet(pub HashMap<Arc<str>, TagValue>); // TODO: make this field private

impl TagSet {
    /// Returns a new empty tag set.
    pub fn new() -> Self {
        TagSet::default()
    }

    /// Returns the authors list.
    pub fn authors(&self) -> &[String] {
        self.0
            .get("author")
            .and_then(|v| v.as_str_list())
            .unwrap_or(&[])
    }
    /// Returns the inventors list.
    pub fn inventors(&self) -> &[String] {
        self.0
            .get("inventor")
            .and_then(|v| v.as_str_list())
            .unwrap_or(&[])
    }

    /// Returns the URL of the puzzle's WCA page.
    pub fn wca_url(&self) -> Option<String> {
        // TODO: museum URL also
        Some(format!(
            "https://www.worldcubeassociation.org/results/rankings/{}/single",
            self.get("external/wca")?.as_str()?,
        ))
    }
    /// Returns the filename where the puzzle was defined, or `"<unknown>"` if
    /// it cannot be found.
    pub fn filename(&self) -> Option<&str> {
        self.get("file").and_then(|v| v.as_str())
    }
    /// Returns whether the tag set contains the "experimental" tag.
    pub fn is_experimental(&self) -> bool {
        self.get("experimental").is_some_and(|v| v.is_present())
    }

    /// Returns whether a tag set meets a search criterion. If `value_str` is
    /// `None`, returns whether the tag is present.
    pub fn meets_search_criterion(&self, tag: &str, value: Option<&str>) -> bool {
        self.get(tag)
            .unwrap_or(&TagValue::False)
            .meets_search_criterion(value)
    }

    /// Returns the value for a tag given its name.
    pub fn get(&self, tag_name: &str) -> Option<&TagValue> {
        self.0.get(tag_name)
    }

    /// Returns an iterator over the tags in the tag set.
    pub fn iter(&self) -> impl Iterator<Item = (&Arc<str>, &TagValue)> {
        self.0.iter()
    }

    /// Adds a tag to the tag set.
    pub fn insert(&mut self, tag: &TagData, value: TagValue) {
        self.0.insert(Arc::clone(&tag.name), value);
    }
    /// Adds a tag to the tag set by name.
    pub fn insert_named(&mut self, tag_name: &str, value: TagValue) -> Result<(), UnknownTag> {
        self.insert(crate::TAGS.get(tag_name)?, value);
        Ok(())
    }

    /// Returns an entry in the map.
    pub fn entry(&mut self, tag: &TagData) -> hash_map::Entry<'_, Arc<str>, TagValue> {
        self.0.entry(Arc::clone(&tag.name))
    }
    /// Returns an entry in the map by name.
    pub fn entry_named(
        &mut self,
        tag_name: &str,
    ) -> Result<hash_map::Entry<'_, Arc<str>, TagValue>, UnknownTag> {
        Ok(self.entry(crate::TAGS.get(tag_name)?))
    }
}

impl Serialize for TagSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(
            self.0
                .values()
                .filter(|v| !matches!(v, TagValue::Inherited))
                .count(),
        ))?;

        for (k, v) in &self.0 {
            let k = &**k;
            match v {
                TagValue::False => map.serialize_entry(k, &false)?,
                TagValue::True => map.serialize_entry(k, &true)?,
                TagValue::Inherited => (),
                TagValue::Int(i) => map.serialize_entry(k, i)?,
                TagValue::Str(s) => map.serialize_entry(k, s)?,
                TagValue::StrList(vec) => map.serialize_entry(k, vec)?,
                TagValue::Puzzle(s) => map.serialize_entry(k, s)?,
            }
        }

        map.end()
    }
}
