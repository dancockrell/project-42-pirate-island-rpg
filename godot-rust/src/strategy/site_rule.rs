//! A7: what a site rule *is*, as data.
//!
//! C5 authored the Tomb of Returning Names and registered twenty-six
//! `site_rule.<...>` IDs, and said plainly that they were opaque: "A7 owns what
//! a site rule does ... nothing here reads a rule's meaning, and no rule
//! mechanics are asserted by authoring an ID." This file is that half. Every
//! registered ID now has a record under `content/site_rules/`, and a record
//! carries an `effect` drawn from the closed vocabulary [`SiteRuleEffect`]
//! declares.
//!
//! ## One owner for the vocabulary
//!
//! [`SiteRuleEffect`] is the whole of what a site rule may do. `battle.rs`
//! reads it and applies it; it does not define a second spelling of it, and
//! `strategy/dungeon_content.rs` still selects rule *IDs* without ever reading
//! a meaning out of one. Content owns the records, this module carries them,
//! and `the_site_rule_registry_equals_the_authored_directory` holds the two
//! equal -- C10's shape, one directory further on.
//!
//! ## Two shapes, and no third
//!
//! A record's `effect` is exactly one of:
//!
//! * `{ "guard_regen_per_round": 2 }` -- the rule A7's card names:
//!   `site_rule.tomb.grave_watch`, under which hostiles regain two Guard at
//!   every round boundary. The tomb watches its own dead.
//! * `{ "needs_decision": true }` -- an authored rule whose mechanics nobody
//!   has decided yet. This is the honest answer for twenty-five of the
//!   twenty-six, and it is written down rather than defaulted: an unknown
//!   effect does nothing *and says so*, so a rule cannot quietly become a
//!   no-op by omission.
//!
//! An `effect` object with neither key is refused by [`AuthoredSiteRule::validate`]
//! and by `tools/src/validate.mjs`, in the same words. There is deliberately no
//! `Unknown` fallback variant: a vocabulary that accepts anything is not a
//! vocabulary.
//!
//! ## What is open
//!
//! * **Twenty-five of the twenty-six rules' mechanics** -- `needs decision`.
//!   Each record says so in its own `effect`, and its `displayName` is an
//!   explicit placeholder. Nothing here invents a mechanic the design has not
//!   named.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::expedition::require_stable_id;
pub use crate::strategy::dungeon_content::SITE_RULE_ID_PREFIX;

/// The one site rule whose mechanics A7's card decides, named once so the
/// record, the battle test and the design document agree on the spelling.
pub const GRAVE_WATCH: &str = "site_rule.tomb.grave_watch";

/// The closed vocabulary of what an authored site rule does.
///
/// Externally tagged, so the JSON is the record's own `effect` object:
/// `{"guard_regen_per_round": 2}` or `{"needs_decision": true}`. An object
/// carrying neither key fails to deserialize, which is the refusal A7's card
/// asks for -- there is no catch-all arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteRuleEffect {
    /// Hostiles regain this much Guard at every round boundary while this rule
    /// is in force. `site_rule.tomb.grave_watch` is 2.
    #[serde(rename = "guard_regen_per_round")]
    GuardRegenPerRound(i32),
    /// The rule is authored and registered, and what it does is Open. Carried
    /// as `true`; `false` would be a claim that a decision is not needed,
    /// which is not a thing a record of this shape can say, so it is refused.
    #[serde(rename = "needs_decision")]
    NeedsDecision(bool),
}

/// What went wrong loading an authored site-rule record. Every variant is a
/// refusal `tools/src/validate.mjs` also makes, in the same words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthoredSiteRuleError {
    /// An ID that is not a stable ID at all.
    MalformedId { value: String },
    /// An ID that does not begin `site_rule.`.
    WrongIdPrefix { found: String },
    /// A record with no display name. Every rule says what it is called, even
    /// when the name is an explicit placeholder.
    NoDisplayName { id: String },
    /// `{"guard_regen_per_round": n}` with `n` at or below zero. A rule that
    /// regenerates nothing is a rule that should have said `needs_decision`.
    GuardRegenNotPositive { id: String, amount: i32 },
    /// `{"needs_decision": false}`: the record claims the decision is made and
    /// then names no mechanic. Refused rather than read as "does nothing".
    NeedsDecisionMustBeTrue { id: String },
    /// Two records claiming the same ID.
    DuplicateSiteRule { id: String },
}

/// One site rule exactly as `content/site_rules/<slug>.json` authors it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredSiteRule {
    /// `site_rule.<...>`, the ID C5's dungeon record and the tomb cells
    /// already reference.
    pub id: String,
    /// Placeholder display name. No site rule has been named for players yet
    /// and each record says so.
    pub display_name: String,
    /// Exactly one of the two shapes [`SiteRuleEffect`] admits.
    pub effect: SiteRuleEffect,
}

impl AuthoredSiteRule {
    /// Refuses every record the validator refuses, before anything reads an
    /// effect out of it.
    pub fn validate(&self) -> Result<(), AuthoredSiteRuleError> {
        require_stable_id("site_rule.id", &self.id).map_err(|_| {
            AuthoredSiteRuleError::MalformedId {
                value: self.id.clone(),
            }
        })?;
        if !self.id.starts_with(SITE_RULE_ID_PREFIX) {
            return Err(AuthoredSiteRuleError::WrongIdPrefix {
                found: self.id.clone(),
            });
        }
        if self.display_name.trim().is_empty() {
            return Err(AuthoredSiteRuleError::NoDisplayName {
                id: self.id.clone(),
            });
        }
        match self.effect {
            SiteRuleEffect::GuardRegenPerRound(amount) if amount <= 0 => {
                Err(AuthoredSiteRuleError::GuardRegenNotPositive {
                    id: self.id.clone(),
                    amount,
                })
            }
            SiteRuleEffect::NeedsDecision(false) => {
                Err(AuthoredSiteRuleError::NeedsDecisionMustBeTrue {
                    id: self.id.clone(),
                })
            }
            _ => Ok(()),
        }
    }

    /// Read and validate a record from its JSON text. One reader: Godot hands
    /// this crate the same bytes out of `game/generated/content_bundle.json`
    /// that the tests read off disk.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// Every authored site rule, keyed by its own stable ID.
///
/// The bridge builds one from the content bundle and hands it to every battle
/// it arms, exactly as it builds `FactionDefinitions` and `BuildingDefinitions`
/// (B15, B16). An empty registry is a legal state -- a payload authored before
/// this card names no site rules -- and a battle then runs with no rules in
/// force, which is what it did before this card existed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SiteRules {
    by_id: BTreeMap<String, AuthoredSiteRule>,
}

impl SiteRules {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and stores one record. A second record for the same ID is
    /// refused rather than overwriting the first.
    pub fn insert(&mut self, rule: AuthoredSiteRule) -> Result<(), AuthoredSiteRuleError> {
        rule.validate()?;
        if self.by_id.contains_key(&rule.id) {
            return Err(AuthoredSiteRuleError::DuplicateSiteRule { id: rule.id });
        }
        self.by_id.insert(rule.id.clone(), rule);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&AuthoredSiteRule> {
        self.by_id.get(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.by_id.keys().map(String::as_str)
    }

    /// The rules in force for a battle: the cell's own `site_rule_ids`, in the
    /// cell's authored order, minus the ones this expedition has suppressed,
    /// minus any this registry has never heard of.
    ///
    /// A rule the registry does not know is dropped rather than guessed at. It
    /// cannot be silent: the validator refuses a `siteRuleIds` entry with no
    /// record, so an unknown ID here means a payload older than the record it
    /// names, and the truthful answer for an unknown rule is that it is not in
    /// force.
    pub fn in_force<'a>(
        &self,
        site_rule_ids: impl IntoIterator<Item = &'a String>,
        suppressed: &std::collections::BTreeSet<String>,
    ) -> Vec<ActiveSiteRule> {
        site_rule_ids
            .into_iter()
            .filter(|id| !suppressed.contains(*id))
            .filter_map(|id| {
                self.get(id).map(|rule| ActiveSiteRule {
                    id: rule.id.clone(),
                    effect: rule.effect,
                })
            })
            .collect()
    }
}

/// One site rule actually in force in one battle: the ID, so Ayla's Override
/// Tomb Rule can name what it overrode, and the effect, so the round boundary
/// can apply it. `Battle` carries a `Vec` of these and nothing else about
/// sites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveSiteRule {
    pub id: String,
    pub effect: SiteRuleEffect,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn authored_directory() -> Vec<(String, AuthoredSiteRule)> {
        let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/site_rules/");
        let mut records = Vec::new();
        for entry in std::fs::read_dir(directory).expect("content/site_rules/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.extension().and_then(|name| name.to_str()) != Some("json") {
                continue;
            }
            let filename = path
                .file_stem()
                .and_then(|name| name.to_str())
                .expect("a readable file name")
                .to_owned();
            let text = std::fs::read_to_string(&path).expect("the site-rule file is readable");
            let record =
                AuthoredSiteRule::from_json(&text).expect("the site-rule file is a site rule");
            records.push((filename, record));
        }
        records.sort_by(|left, right| left.0.cmp(&right.0));
        records
    }

    /// C10's shape: content owns, Rust carries, a test holds them equal. The
    /// registry the simulation runs on is exactly `content/site_rules/`, every
    /// record validates, and the file's own name is its ID's slug -- so a
    /// record deleted, renamed or added fails here by name.
    #[test]
    fn the_site_rule_registry_equals_the_authored_directory() {
        let authored = authored_directory();
        assert!(
            !authored.is_empty(),
            "content/site_rules/ must contain records for this test to mean anything"
        );
        let mut registry = SiteRules::new();
        for (filename, record) in &authored {
            assert_eq!(
                record.id,
                format!("{SITE_RULE_ID_PREFIX}{filename}"),
                "content/site_rules/{filename}.json declares {}; the file name is the ID's slug",
                record.id
            );
            registry
                .insert(record.clone())
                .unwrap_or_else(|error| panic!("{} does not load: {error:?}", record.id));
        }
        let authored_ids: BTreeSet<&str> = authored
            .iter()
            .map(|(_, record)| record.id.as_str())
            .collect();
        let registry_ids: BTreeSet<&str> = registry.ids().collect();
        assert_eq!(
            registry_ids, authored_ids,
            "the registry and content/site_rules/ must hold the same rules"
        );
    }

    /// Every `site_rule.<...>` C5's dungeon record registers has a record
    /// here, on both sides: an authored rule nobody selects is a noodle to
    /// nowhere, and a selected rule with no record is a mechanic nobody wrote.
    #[test]
    fn every_site_rule_the_tomb_selects_has_a_record_and_the_other_way_round() {
        use crate::strategy::dungeon_content::AuthoredDungeon;

        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../content/dungeons/tomb_of_returning_names.json"
        );
        let text = std::fs::read_to_string(path).expect("the tomb record is readable");
        let tomb = AuthoredDungeon::from_json(&text).expect("the tomb record loads");
        let mut selected: BTreeSet<String> = BTreeSet::new();
        for space in &tomb.spaces {
            selected.extend(space.site_rule_ids.iter().cloned());
            selected.extend(space.corrupted_site_rule_ids.iter().cloned());
        }
        let authored: BTreeSet<String> = authored_directory()
            .into_iter()
            .map(|(_, record)| record.id)
            .collect();
        assert_eq!(
            authored, selected,
            "content/site_rules/ and the site rules content/dungeons/ selects must be the same set"
        );
    }

    /// The card's one decided rule, read off disk rather than restated.
    #[test]
    fn grave_watch_regenerates_two_guard_a_round() {
        let mut registry = SiteRules::new();
        for (_, record) in authored_directory() {
            registry.insert(record).expect("the record loads");
        }
        assert_eq!(
            registry.get(GRAVE_WATCH).map(|rule| rule.effect),
            Some(SiteRuleEffect::GuardRegenPerRound(2))
        );
    }

    /// An `effect` naming neither shape is refused at deserialization, with no
    /// fallback variant to land in.
    #[test]
    fn an_effect_the_vocabulary_does_not_name_does_not_load() {
        let text = r#"{"id":"site_rule.tomb.invented","displayName":"Invented","effect":{"summon_a_dragon":true}}"#;
        assert!(AuthoredSiteRule::from_json(text).is_err());
        let empty = r#"{"id":"site_rule.tomb.invented","displayName":"Invented","effect":{}}"#;
        assert!(AuthoredSiteRule::from_json(empty).is_err());
    }

    #[test]
    fn a_record_that_says_no_decision_is_needed_and_names_no_mechanic_is_refused() {
        let text = r#"{"id":"site_rule.tomb.invented","displayName":"Invented","effect":{"needs_decision":false}}"#;
        let record = AuthoredSiteRule::from_json(text).expect("it deserializes");
        assert_eq!(
            record.validate(),
            Err(AuthoredSiteRuleError::NeedsDecisionMustBeTrue {
                id: "site_rule.tomb.invented".into()
            })
        );
    }

    /// A suppressed rule is not in force, and an unknown one is not guessed at.
    #[test]
    fn suppressed_and_unknown_rules_are_not_in_force() {
        let mut registry = SiteRules::new();
        for (_, record) in authored_directory() {
            registry.insert(record).expect("the record loads");
        }
        let ids = vec![
            GRAVE_WATCH.to_owned(),
            "site_rule.tomb.registry_order".to_owned(),
            "site_rule.tomb.nobody_authored_this".to_owned(),
        ];
        let none = BTreeSet::new();
        let in_force = registry.in_force(&ids, &none);
        assert_eq!(
            in_force
                .iter()
                .map(|rule| rule.id.as_str())
                .collect::<Vec<_>>(),
            vec![GRAVE_WATCH, "site_rule.tomb.registry_order"]
        );
        let suppressed = BTreeSet::from([GRAVE_WATCH.to_owned()]);
        let in_force = registry.in_force(&ids, &suppressed);
        assert_eq!(
            in_force
                .iter()
                .map(|rule| rule.id.as_str())
                .collect::<Vec<_>>(),
            vec!["site_rule.tomb.registry_order"]
        );
    }
}
