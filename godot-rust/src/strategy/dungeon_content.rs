//! C5: the authored dungeon record, and the site rules an owning faction
//! brings to it. `docs/SHIP_PLAN.md` section 9, card C5.
//!
//! The design bible's section 3.13 names twelve spaces for the Tomb of
//! Returning Names, and says why they are twelve *authored* spaces rather than
//! twelve rectangles: "It is twelve authored spaces that force the systems to
//! meet." Those twelve, their purposes and their persistent proofs are content
//! -- `content/dungeons/tomb_of_returning_names.json` -- and this file is the
//! reader that carries them into the simulation. Content owns; Rust carries; a
//! test holds them equal, exactly as C10 did for `content/buildings/`.
//!
//! ## Why this is a separate file from `strategy/dungeon.rs`
//!
//! S9 owns `dungeon.rs`, which builds [`DungeonContext`] and its signature: a
//! *reading of the board*, with no authored record anywhere in it. This file
//! is the other half of the seam S9's module docs name by hand -- "C5 authors
//! the tomb's twelve spaces and A7 authors its site rules" -- and it is a
//! reader of authored JSON, which is a different responsibility. Nothing in
//! `dungeon.rs` is copied, shadowed, or re-implemented here: this module
//! imports [`DungeonContext`] and [`ConceptKey`] and adds the one thing
//! neither of them has, which is the record.
//!
//! ## What selects what
//!
//! [`select_site_rules`] is the whole behaviour: given the authored record and
//! a context read off the board, it returns the site rules in force. The
//! owning faction decides, and only the owning faction -- the corrupted
//! variant's set when the Cthulhu concept key holds the site, the default
//! owner's set otherwise. That is brief section 12 in one function: "A
//! level-five dungeon controlled by one faction must not be the same dungeon
//! with a palette swap as a level-two dungeon controlled by another faction."
//!
//! ## What a site rule ID means here: nothing
//!
//! A `site_rule.<...>` ID is **opaque** in this file and in the record. A7
//! owns what a site rule does (`site_rule.tomb.grave_watch`: hostiles regain
//! two Guard each round) and no registry of rules exists yet, so nothing here
//! reads a rule's meaning, and no rule mechanics are asserted by authoring an
//! ID. When A7 lands, `Battle::new` takes the cell's rules minus the
//! suppressed ones, and the cell's rules are the `siteRuleIds` this module
//! already selects. There is deliberately no second list.
//!
//! ## What is open
//!
//! * **A third owner's rule set** -- `needs decision`. The record carries two
//!   sets, so [`select_site_rules`] gives every owner that is not the
//!   corrupted variant the default set. That is the honest reading of a
//!   two-set record, not a claim that a colonial garrison keeps elven rules.
//! * **Six of the twelve spaces have no world cell.** B6 authored four tomb
//!   interior cells and two approach cells were authored before them; the
//!   record says `worldCellId: null` and a note for each of the other six
//!   rather than inventing a room the bible has not described in place.

use serde::{Deserialize, Serialize};

use crate::expedition::require_stable_id;
use crate::strategy::dungeon::DungeonContext;
use crate::strategy::faction::ConceptKey;

/// Every authored dungeon ID begins here, as `faction.` and `building.` do for
/// theirs.
pub const DUNGEON_ID_PREFIX: &str = "dungeon.";

/// Every authored site-rule ID begins here. A7 owns what follows it.
pub const SITE_RULE_ID_PREFIX: &str = "site_rule.";

/// The one dungeon `content/dungeons/` authors today, named once so tests and
/// callers agree on the spelling.
pub const TOMB_OF_RETURNING_NAMES: &str = "dungeon.tomb_of_returning_names";

/// What went wrong loading an authored dungeon record.
///
/// Every variant is a refusal the validator (`tools/src/validate.mjs`) also
/// makes on the content side, deliberately: a bad record dies at
/// `node tools/src/validate.mjs` and at `cargo test`, and says the same thing
/// both times.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthoredDungeonError {
    /// An ID that is not a stable ID at all.
    MalformedId { field: &'static str, value: String },
    /// An ID whose prefix names the wrong kind of thing.
    WrongIdPrefix {
        found: String,
        expected: &'static str,
    },
    /// A record with no spaces. A dungeon of nothing is not a dungeon.
    NoSpaces { id: String },
    /// A space whose ID is not `<dungeon id>.space.<space slug>`.
    SpaceIdDoesNotMatch { found: String, expected: String },
    /// Two spaces claiming the same slug.
    DuplicateSpace { space: String },
    /// A space with an empty rule set on either side.
    NoSiteRules { space: String, field: &'static str },
    /// The default owner and the corrupted variant are the same concept, so no
    /// owner change could ever change a rule.
    OwnerAndVariantAreTheSame { concept_key: ConceptKey },
    /// A space whose two rule sets are equal: brief section 12's palette swap.
    VariantIsAPaletteSwap { space: String },
}

/// One of the design bible's authored spaces, as content writes it.
///
/// The bible's `purpose` and `persistent_proof` are carried verbatim rather
/// than paraphrased, because they are the acceptance criteria for the space:
/// "Grate, cache and noise state persisted" is what finishing the service
/// crawl means.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredSpace {
    /// `<dungeon id>.space.<space>`.
    pub id: String,
    /// The bible's space, as a lower_snake_case slug. A world cell's
    /// `dungeonContext.space` names this.
    pub space: String,
    /// Placeholder display name. No space of this tomb has been named for
    /// players yet, and each says so.
    pub display_name: String,
    /// The bible's "Primary purpose" column, verbatim.
    pub purpose: String,
    /// The bible's "Persistent proof" column, verbatim.
    pub persistent_proof: String,
    /// The `world.cell.*` that realises this space today, or `None` where none
    /// is authored. `None` is a real answer and the record's `note` says why.
    pub world_cell_id: Option<String>,
    /// The site rules the default owner brings to this space. A realising
    /// cell's `dungeonContext.siteRuleIds` mirrors this list exactly.
    pub site_rule_ids: Vec<String>,
    /// The site rules the corrupted variant brings instead. Never equal to
    /// [`AuthoredSpace::site_rule_ids`].
    pub corrupted_site_rule_ids: Vec<String>,
    /// Why this space maps to that cell, or why no cell realises it.
    pub note: String,
}

/// An authored dungeon: the bible's spaces, the faction that holds the site by
/// default, and the faction whose hold corrupts it.
///
/// Both owners are [`ConceptKey`]s, never names. Brief section 4: "Do not
/// invent proper faction names without explicit approval", so a record that
/// tried to write one would not deserialize.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredDungeon {
    /// `dungeon.<...>`.
    pub id: String,
    /// Placeholder display name for the site as a whole.
    pub display_name: String,
    /// Who holds the site when nobody has taken it: the elves built it.
    pub default_owner_concept_key: ConceptKey,
    /// Whose hold produces the corrupted variant of every rule set.
    pub corrupted_variant_concept_key: ConceptKey,
    /// The bible's spaces, in the bible's order.
    pub spaces: Vec<AuthoredSpace>,
}

impl AuthoredDungeon {
    /// Refuses every record `tools/src/validate.mjs` refuses, before anything
    /// reads a rule out of it.
    pub fn validate(&self) -> Result<(), AuthoredDungeonError> {
        require_stable_id("dungeon.id", &self.id).map_err(|_| {
            AuthoredDungeonError::MalformedId {
                field: "dungeon.id",
                value: self.id.clone(),
            }
        })?;
        if !self.id.starts_with(DUNGEON_ID_PREFIX) {
            return Err(AuthoredDungeonError::WrongIdPrefix {
                found: self.id.clone(),
                expected: DUNGEON_ID_PREFIX,
            });
        }
        if self.default_owner_concept_key == self.corrupted_variant_concept_key {
            return Err(AuthoredDungeonError::OwnerAndVariantAreTheSame {
                concept_key: self.default_owner_concept_key,
            });
        }
        if self.spaces.is_empty() {
            return Err(AuthoredDungeonError::NoSpaces {
                id: self.id.clone(),
            });
        }

        let mut seen: Vec<&str> = Vec::new();
        for space in &self.spaces {
            let expected = format!("{}.space.{}", self.id, space.space);
            if space.id != expected {
                return Err(AuthoredDungeonError::SpaceIdDoesNotMatch {
                    found: space.id.clone(),
                    expected,
                });
            }
            if seen.contains(&space.space.as_str()) {
                return Err(AuthoredDungeonError::DuplicateSpace {
                    space: space.space.clone(),
                });
            }
            seen.push(&space.space);

            for (field, rules) in [
                ("site_rule_ids", &space.site_rule_ids),
                ("corrupted_site_rule_ids", &space.corrupted_site_rule_ids),
            ] {
                if rules.is_empty() {
                    return Err(AuthoredDungeonError::NoSiteRules {
                        space: space.space.clone(),
                        field,
                    });
                }
                for rule_id in rules {
                    require_stable_id("site_rule_id", rule_id).map_err(|_| {
                        AuthoredDungeonError::MalformedId {
                            field: "site_rule_id",
                            value: rule_id.clone(),
                        }
                    })?;
                    if !rule_id.starts_with(SITE_RULE_ID_PREFIX) {
                        return Err(AuthoredDungeonError::WrongIdPrefix {
                            found: rule_id.clone(),
                            expected: SITE_RULE_ID_PREFIX,
                        });
                    }
                }
            }
            // The card's done-when in one refusal: if the corrupted variant
            // selected the same rules, an owner change would be invisible.
            if space.site_rule_ids == space.corrupted_site_rule_ids {
                return Err(AuthoredDungeonError::VariantIsAPaletteSwap {
                    space: space.space.clone(),
                });
            }
        }
        Ok(())
    }

    /// The space with this slug, or `None`.
    pub fn space(&self, space: &str) -> Option<&AuthoredSpace> {
        self.spaces.iter().find(|entry| entry.space == space)
    }

    /// Read and validate a record from its JSON text.
    ///
    /// One reader. Godot hands this crate the same bytes out of
    /// `game/generated/content_bundle.json` that the tests read off disk, so a
    /// record that loads here loads there.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// **The card's behaviour.** The site rules in force in `dungeon` for the
/// faction that holds it in `context`.
///
/// The corrupted variant's rules when [`DungeonContext::owning_faction_id`] is
/// the corrupted variant's `faction.<concept_key>`; the default owner's rules
/// for every other answer, including unheld ground (`None`). Ordered by the
/// record's own space order and de-duplicated, so the same context always
/// gives the same list in the same order and two spaces sharing a rule name it
/// once.
///
/// Pure: it reads the record and the context and touches nothing else. The
/// board's answer to "who holds this?" is S2's, arriving through
/// [`DungeonContext::of`]; this function never decides ownership, it only
/// reads it.
pub fn select_site_rules(dungeon: &AuthoredDungeon, context: &DungeonContext) -> Vec<String> {
    let corrupted = context.owning_faction_id.as_deref()
        == Some(dungeon.corrupted_variant_concept_key.faction_id().as_str());

    let mut selected: Vec<String> = Vec::new();
    for space in &dungeon.spaces {
        let rules = if corrupted {
            &space.corrupted_site_rule_ids
        } else {
            &space.site_rule_ids
        };
        for rule_id in rules {
            if !selected.iter().any(|already| already == rule_id) {
                selected.push(rule_id.clone());
            }
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The authored record, read off disk exactly as
    /// `every_authored_building_record_loads_and_validates` reads C10's.
    fn authored_tomb() -> AuthoredDungeon {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../content/dungeons/tomb_of_returning_names.json"
        );
        let text = std::fs::read_to_string(path).expect("the tomb record is readable");
        let record =
            AuthoredDungeon::from_json(&text).expect("the tomb record is an AuthoredDungeon");
        record.validate().expect("the tomb record validates");
        record
    }

    /// A context that is entirely default except for who holds the ground,
    /// which is the only field [`select_site_rules`] reads. Everything else in
    /// S9's context is the signature's business, not this file's.
    fn held_by(owner: Option<ConceptKey>) -> DungeonContext {
        DungeonContext {
            owning_faction_id: owner.map(ConceptKey::faction_id),
            grammar_id: TOMB_OF_RETURNING_NAMES.into(),
            ..DungeonContext::default()
        }
    }

    /// C10's shape: content owns, Rust carries, a test holds them equal. The
    /// record on disk is the twelve spaces of the design bible's section 3.13,
    /// in its order, and every field the reader carries is the field the file
    /// wrote -- proven by serializing the loaded record back and comparing it
    /// to the file's own JSON value, so a field the reader silently dropped
    /// fails here.
    #[test]
    fn the_authored_tomb_record_equals_what_the_reader_loads() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../content/dungeons/tomb_of_returning_names.json"
        );
        let text = std::fs::read_to_string(path).expect("the tomb record is readable");
        let file: serde_json::Value = serde_json::from_str(&text).expect("the file is JSON");
        let record = authored_tomb();

        assert_eq!(record.id, TOMB_OF_RETURNING_NAMES);
        assert_eq!(record.default_owner_concept_key, ConceptKey::Elves);
        assert_eq!(record.corrupted_variant_concept_key, ConceptKey::Cthulhu);

        let carried = serde_json::to_value(&record).expect("the record serializes");
        for (key, value) in carried.as_object().expect("the record is an object") {
            assert_eq!(
                file.get(key),
                Some(value),
                "the reader's {key} differs from the authored record"
            );
        }

        // The bible's twelve spaces, in the bible's order. Written out here so
        // that dropping one, renaming one, or inventing a thirteenth fails by
        // name rather than by count.
        let spaces: Vec<&str> = record
            .spaces
            .iter()
            .map(|space| space.space.as_str())
            .collect();
        assert_eq!(
            spaces,
            vec![
                "reception_terrace",
                "processional_ramp",
                "registry_court",
                "water_stair",
                "petition_gallery",
                "service_crawl",
                "mourning_court",
                "ossuary_lift",
                "flooded_archive",
                "custodian_bridge",
                "sealed_petition_room",
                "burial_lord_chamber",
            ],
            "the record must carry the design bible section 3.13 spaces, in order"
        );
    }

    /// The card's done-when, first half: the same context selects the same
    /// rules. Not "usually" -- the function is pure, so this is the fact that
    /// makes a site rule survive a reload.
    #[test]
    fn the_same_context_selects_the_same_site_rules() {
        let tomb = authored_tomb();
        let context = held_by(Some(ConceptKey::Elves));

        let first = select_site_rules(&tomb, &context);
        let second = select_site_rules(&tomb, &held_by(Some(ConceptKey::Elves)));

        assert!(!first.is_empty(), "the elves' hold selects rules");
        assert_eq!(first, second, "the same context must select the same rules");
        assert_eq!(
            first,
            select_site_rules(&tomb, &context),
            "and must do so however many times it is asked"
        );
    }

    /// The card's done-when, second half: an owner change selects different
    /// rules. Brief section 12 -- not the same dungeon with a palette swap.
    #[test]
    fn an_owner_change_selects_different_site_rules() {
        let tomb = authored_tomb();

        let elven = select_site_rules(&tomb, &held_by(Some(ConceptKey::Elves)));
        let corrupted = select_site_rules(&tomb, &held_by(Some(ConceptKey::Cthulhu)));

        assert_ne!(
            elven, corrupted,
            "the Cthulhu concept key must select the corrupted variant's rules"
        );
        assert!(
            elven
                .iter()
                .collect::<BTreeSet<_>>()
                .intersection(&corrupted.iter().collect::<BTreeSet<_>>())
                .next()
                .is_none(),
            "no rule may stand under both owners; a shared rule is a rule the owner does not decide"
        );
        assert_eq!(
            corrupted,
            select_site_rules(&tomb, &held_by(Some(ConceptKey::Cthulhu))),
            "the corrupted selection is as stable as the default one"
        );
    }

    /// Every owner that is not the corrupted variant brings the default set,
    /// unheld ground included. The record carries two sets and this is the
    /// honest reading of two sets; a third set is `needs decision`.
    #[test]
    fn every_other_owner_and_unheld_ground_bring_the_default_rules() {
        let tomb = authored_tomb();
        let elven = select_site_rules(&tomb, &held_by(Some(ConceptKey::Elves)));

        for owner in [
            None,
            Some(ConceptKey::Michael),
            Some(ConceptKey::Pirates),
            Some(ConceptKey::ColonialPowers),
            Some(ConceptKey::FoxPeople),
        ] {
            assert_eq!(
                select_site_rules(&tomb, &held_by(owner)),
                elven,
                "{owner:?} is not the corrupted variant, so it brings the default rules"
            );
        }
    }

    /// The mirror is held equal. A world cell that realises a space carries
    /// that space's default-owner rules in `dungeonContext.siteRuleIds`, and
    /// the authored cell files are checked against the record here as well as
    /// in `tools/src/validate.mjs`, so the two answers cannot drift.
    #[test]
    fn every_tomb_cell_mirrors_its_authored_space() {
        let tomb = authored_tomb();
        let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/world/");
        let mut mirrored = 0;

        for entry in std::fs::read_dir(directory).expect("content/world/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.extension().and_then(|name| name.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("the cell file is readable");
            let cell: serde_json::Value = serde_json::from_str(&text).expect("the cell is JSON");
            let Some(block) = cell.get("dungeonContext") else {
                continue;
            };
            mirrored += 1;

            assert_eq!(
                block.get("dungeonId").and_then(serde_json::Value::as_str),
                Some(TOMB_OF_RETURNING_NAMES),
                "{} names a dungeon no record authors",
                path.display()
            );
            assert_eq!(
                block
                    .get("ownerConceptKey")
                    .and_then(serde_json::Value::as_str),
                Some(tomb.default_owner_concept_key.as_key()),
                "{} disagrees with the record's default owner",
                path.display()
            );
            assert_eq!(
                block
                    .get("corruptedVariantConceptKey")
                    .and_then(serde_json::Value::as_str),
                Some(tomb.corrupted_variant_concept_key.as_key()),
                "{} disagrees with the record's corrupted variant",
                path.display()
            );

            let slug = block
                .get("space")
                .and_then(serde_json::Value::as_str)
                .expect("a cell's dungeonContext names a space");
            let space = tomb.space(slug).unwrap_or_else(|| {
                panic!(
                    "{} names space {slug}, which the record does not carry",
                    path.display()
                )
            });
            assert_eq!(
                space.world_cell_id.as_deref(),
                cell.get("id").and_then(serde_json::Value::as_str),
                "the record says {slug} is realised elsewhere"
            );
            assert_eq!(
                block.get("siteRuleIds"),
                Some(&serde_json::to_value(&space.site_rule_ids).expect("rules serialize")),
                "{} carries site rules the record does not give {slug}",
                path.display()
            );
        }

        assert_eq!(
            mirrored, 4,
            "B6 authored four tomb interior cells and each carries a dungeonContext block"
        );
    }

    /// Six of the twelve spaces have no cell, and the record says so in the
    /// open rather than by omission. This test is what makes that honest: a
    /// space with neither a cell nor a note fails.
    #[test]
    fn every_space_either_names_a_cell_or_says_that_none_is_authored() {
        let tomb = authored_tomb();
        let realised = tomb
            .spaces
            .iter()
            .filter(|space| space.world_cell_id.is_some())
            .count();

        assert_eq!(
            realised, 6,
            "six spaces are realised today: two approach cells and B6's four tomb interiors"
        );
        for space in &tomb.spaces {
            assert!(
                !space.note.trim().is_empty(),
                "{} must say how it maps, or that nothing realises it",
                space.space
            );
            assert!(
                !space.purpose.trim().is_empty() && !space.persistent_proof.trim().is_empty(),
                "{} must carry the design bible's purpose and persistent proof",
                space.space
            );
        }
    }

    /// The refusals, each against a record that differs from the authored one
    /// in exactly the one way named.
    #[test]
    fn a_bad_record_refuses_before_a_rule_is_read() {
        let mut palette_swap = authored_tomb();
        palette_swap.spaces[0].corrupted_site_rule_ids =
            palette_swap.spaces[0].site_rule_ids.clone();
        assert_eq!(
            palette_swap.validate(),
            Err(AuthoredDungeonError::VariantIsAPaletteSwap {
                space: "reception_terrace".into()
            })
        );

        let mut same_owner = authored_tomb();
        same_owner.corrupted_variant_concept_key = ConceptKey::Elves;
        assert_eq!(
            same_owner.validate(),
            Err(AuthoredDungeonError::OwnerAndVariantAreTheSame {
                concept_key: ConceptKey::Elves
            })
        );

        let mut renamed = authored_tomb();
        renamed.spaces[2].space = "crypt_of_coffins".into();
        assert!(matches!(
            renamed.validate(),
            Err(AuthoredDungeonError::SpaceIdDoesNotMatch { .. })
        ));

        let mut wrong_prefix = authored_tomb();
        wrong_prefix.spaces[0].site_rule_ids = vec!["condition.tomb.grave_watch".into()];
        assert_eq!(
            wrong_prefix.validate(),
            Err(AuthoredDungeonError::WrongIdPrefix {
                found: "condition.tomb.grave_watch".into(),
                expected: SITE_RULE_ID_PREFIX
            })
        );

        let mut empty = authored_tomb();
        empty.spaces[0].site_rule_ids.clear();
        assert_eq!(
            empty.validate(),
            Err(AuthoredDungeonError::NoSiteRules {
                space: "reception_terrace".into(),
                field: "site_rule_ids"
            })
        );
    }
}
