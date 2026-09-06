//! S1: what a faction *is* (authored) and what a faction *has right now*
//! (saved). `docs/SHIP_PLAN.md` section 7, card S1.
//!
//! Two types, two lifetimes. [`FactionDefinition`] is content: authored once
//! in `content/factions/<concept_key>.json` (C9's card), loaded into a
//! [`FactionDefinitions`] registry, never mutated by the simulation.
//! [`FactionState`] is save data: it lives in
//! [`ExpeditionState::factions`](crate::expedition::ExpeditionState::factions)
//! and is the only half a strategic tick may write.
//!
//! `FactionDefinition`'s field list is brief section 19's "Faction definition"
//! block verbatim and in the brief's order, so the design document and the
//! code stay one vocabulary and a reviewer can diff them by eye.
//!
//! **Three constraints this file is built around, all from the brief.**
//!
//! *No proper names.* Brief section 4 accepts six faction *concepts* and
//! explicitly refuses to invent names for them. So the identity of a faction
//! is a [`ConceptKey`] -- one of six -- and its stable ID is
//! `faction.<concept_key>`, checked by [`FactionDefinition::validate`]. There
//! is no `name` field to hold a name in. C9 authors a `displayName` marked
//! `needs decision`; nothing in Rust reads it.
//!
//! *Resource categories are Open* (brief section 20). There is deliberately no
//! resource enum here. `resources` is `BTreeMap<String, u32>` keyed by
//! whatever the faction records supply, and `resource_priorities` weights the
//! same open key space. The character-scale currency -- A3's rations, medicine
//! and coin in [`SupplyState`](crate::expedition::SupplyState), and `Habitats`'
//! drop tables -- is a separate economy and is not mirrored here.
//!
//! *Provisional doctrines are not behaviour.* Brief sections 6.1-6.4 are
//! Provisional, so `doctrine`, `board_position_behavior`, `recovery_rules`,
//! `elimination_rules` and their neighbours are authored notes this module
//! stores, validates the shape of, and never acts on. S15 turns approved
//! doctrines into behaviour; until then no code in the strategy tree may
//! branch on their contents.
//!
//! Nothing here ticks, scores, or draws randomly. S4 owns the tick, S5 the
//! utility scoring, S6 the relationship arithmetic.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::expedition::{ExpeditionError, require_stable_id};
use crate::strategy::utility::Goal;

/// The prefix every faction's stable ID carries. IDs are
/// `faction.<concept_key>` and nothing else, so a proper name cannot enter the
/// ID space even by accident.
pub const FACTION_ID_PREFIX: &str = "faction.";

/// The six faction concepts brief section 4 accepts: five established
/// autonomous factions plus the one Captain Michael founds.
///
/// These are *concept keys*, not names. The brief says outright: "Do not
/// invent proper faction names without explicit approval." This enum is
/// therefore closed -- a seventh faction is a design decision, not a content
/// edit -- and content that names a key outside it fails to load rather than
/// being admitted as an unknown string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConceptKey {
    FoxPeople,
    ColonialPowers,
    Pirates,
    Elves,
    Cthulhu,
    /// Captain Michael's faction. Distinct from the other five in the rules
    /// S3 and S13 will enforce (its buildings produce machines, capacity and
    /// services -- never people); distinct in nothing S1 stores.
    Michael,
}

impl ConceptKey {
    /// Every concept the brief accepts, in the brief's own order. The one
    /// place the list exists; iterate this rather than restating it.
    pub const ALL: [ConceptKey; 6] = [
        ConceptKey::FoxPeople,
        ConceptKey::ColonialPowers,
        ConceptKey::Pirates,
        ConceptKey::Elves,
        ConceptKey::Cthulhu,
        ConceptKey::Michael,
    ];

    /// The authored key exactly as it appears in content and in the stable ID.
    pub fn as_key(self) -> &'static str {
        match self {
            ConceptKey::FoxPeople => "fox_people",
            ConceptKey::ColonialPowers => "colonial_powers",
            ConceptKey::Pirates => "pirates",
            ConceptKey::Elves => "elves",
            ConceptKey::Cthulhu => "cthulhu",
            ConceptKey::Michael => "michael",
        }
    }

    /// The inverse of [`ConceptKey::as_key`]. `None` for anything outside the
    /// six -- the caller refuses the record, it does not invent a faction.
    pub fn from_key(key: &str) -> Option<Self> {
        ConceptKey::ALL
            .into_iter()
            .find(|concept| concept.as_key() == key)
    }

    /// The stable ID this concept must be authored under: `faction.<key>`.
    pub fn faction_id(self) -> String {
        format!("{FACTION_ID_PREFIX}{}", self.as_key())
    }
}

/// What went wrong loading or registering a faction record.
///
/// A module-local error enum, following `BattleError` and `ExpeditionError`:
/// one owner per failure vocabulary. ID *shape* is not re-implemented here --
/// [`FactionError::MalformedId`] carries `ExpeditionError`'s verdict from the
/// crate's single `require_stable_id` rule.
#[derive(Clone, Debug, PartialEq)]
pub enum FactionError {
    /// Not a stable ID at all (empty, uppercase, no dot, stray punctuation).
    MalformedId(ExpeditionError),
    /// Well-shaped, but not the `faction.<concept_key>` this record's
    /// `concept_key` demands. The trap S1's card names: an ID that drifted
    /// away from its concept is the first place a proper name would appear.
    IdDoesNotMatchConceptKey { found: String, expected: String },
    /// Two records claim the same faction. One concept, one record.
    DuplicateFaction { id: String },
    /// A relationship, priority or state names a faction no record defines.
    UnknownFaction { id: String },
}

/// A faction's authored design: brief section 19's "Faction definition" block,
/// field for field and in order.
///
/// Everything the brief leaves Provisional or Open is stored as authored text
/// or as an open-keyed map, never as an invented enum. That is why so many
/// fields are `String`: the brief's answer for them is "not decided yet", and
/// a premature enum would be this repository claiming a decision it was not
/// given. Each such field says which brief section owns the decision.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionDefinition {
    /// `faction.<concept_key>`. See [`FactionDefinition::validate`].
    pub id: String,
    pub concept_key: ConceptKey,
    /// Brief sections 6.1-6.4, Provisional. The faction's strategic doctrine
    /// as authored prose. **Not behaviour until S15** -- no code may branch on
    /// this string.
    #[serde(default)]
    pub doctrine: String,
    /// How much this faction wants each resource category, keyed by the open
    /// resource-category strings its record supplies (brief section 20:
    /// "Exact resource list" is Open, so there is no enum). Higher wants more;
    /// S5 reads these weights, S1 only stores them.
    #[serde(default)]
    pub resource_priorities: BTreeMap<String, i16>,
    /// The same shape over S3's `building.<...>` definition IDs.
    #[serde(default)]
    pub building_priorities: BTreeMap<String, i16>,
    /// Where new members come from (brief section 4 "Population or recruitment
    /// source"). Authored prose: the population model for the five established
    /// factions is Provisional, and Michael's is S12's and S13's. Michael's
    /// faction never grows people from building timers -- brief section 20
    /// rejects that outright -- and S3 enforces it at load, not this field.
    #[serde(default)]
    pub recruitment_or_population_rules: String,
    /// Route and terrain appetite, keyed by the authored terrain / travel-mode
    /// strings on the world graph. Higher prefers.
    #[serde(default)]
    pub movement_preferences: BTreeMap<String, i16>,
    /// The relationship this faction *starts* holding toward each other
    /// faction, keyed by `faction.<concept_key>`. Brief section 7's pairwise
    /// pressures are Provisional, so this is an authored opening position and
    /// nothing more; the live values live in [`FactionState::relationships`]
    /// and only S6 moves them. Reusing [`Relationship`] rather than defining a
    /// second "tendency" shape keeps one struct per concept.
    #[serde(default)]
    pub relationship_tendencies: BTreeMap<String, Relationship>,
    /// What the faction does from each board position (brief section 9).
    /// Authored prose per [`StrategicState`]: "Strategic board-position
    /// categories" is Provisional, so the *categories* are typed and their
    /// *behaviour* is a note S5 will replace.
    #[serde(default)]
    pub board_position_behavior: BTreeMap<StrategicState, String>,
    /// Brief section 20 Open: "Exact ordinary-faction victory conditions."
    /// Authored notes, in authored order, until that decision lands.
    #[serde(default)]
    pub victory_conditions: Vec<String>,
    /// How the faction climbs back out of `Desperate` (brief section 4
    /// "Recovery behavior"). S10 owns the recovery chain; prose until then.
    #[serde(default)]
    pub recovery_rules: String,
    /// What its removal does to the island (brief section 4 "Elimination
    /// consequences"). S10's; prose until then.
    #[serde(default)]
    pub elimination_rules: String,
    /// Weather appetite, keyed by the authored weather strings.
    #[serde(default)]
    pub weather_preferences: BTreeMap<String, i16>,
    /// Terrain appetite, keyed by the authored terrain strings. Distinct from
    /// `movement_preferences`: this is where the faction wants to *be*, that
    /// is how it wants to *travel*.
    #[serde(default)]
    pub terrain_influence: BTreeMap<String, i16>,
    /// How the faction meets corruption (brief section 11). Which corruption
    /// effects are reversible is Open, so prose.
    #[serde(default)]
    pub corruption_interactions: String,
    /// The `building.<...>` definition IDs this faction may raise (S3, C10).
    #[serde(default)]
    pub building_kit: BTreeSet<String>,
    /// The actor / unit definition IDs this faction may field.
    #[serde(default)]
    pub actor_kit: BTreeSet<String>,
    /// The faction-flavoured dungeon vocabulary (brief section 4 "Dungeon
    /// grammar"). Prose until a dungeon generator exists to consume it.
    #[serde(default)]
    pub dungeon_grammar: String,
    /// The faction-flavoured loot vocabulary (brief section 4 "Loot grammar").
    /// The character-scale drop tables in `habitat.rs` stay separate.
    #[serde(default)]
    pub loot_grammar: String,
}

impl Default for ConceptKey {
    /// Only so `FactionDefinition` can derive `Default` for
    /// `#[serde(default)]`; a record always states its own key, and
    /// [`FactionDefinition::validate`] rejects one whose ID disagrees.
    fn default() -> Self {
        ConceptKey::Michael
    }
}

impl FactionDefinition {
    /// The one gate between authored content and the simulation.
    ///
    /// Checks exactly two things, because they are the two that let a proper
    /// name or a stray faction into the game: the ID is a well-shaped stable
    /// ID, and it is precisely `faction.<concept_key>` for this record's own
    /// concept. Everything else in the record is Provisional or Open prose
    /// that this module has no authority to judge.
    pub fn validate(&self) -> Result<(), FactionError> {
        require_stable_id("faction.id", &self.id).map_err(FactionError::MalformedId)?;
        let expected = self.concept_key.faction_id();
        if self.id != expected {
            return Err(FactionError::IdDoesNotMatchConceptKey {
                found: self.id.clone(),
                expected,
            });
        }
        Ok(())
    }
}

/// The loaded faction records, keyed by `faction.<concept_key>`.
///
/// The registry S4's `strategic_tick(&mut self, geography, factions:
/// &FactionDefinitions)` takes. It exists in S1 because loading C9's records
/// is S1's half of that contract; giving S4 its own loader would be a second
/// owner for the same job.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionDefinitions {
    by_id: BTreeMap<String, FactionDefinition>,
}

impl FactionDefinitions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a record and admit it. Refuses a second record for a concept
    /// rather than overwriting: one concept, one authored faction.
    pub fn insert(&mut self, definition: FactionDefinition) -> Result<(), FactionError> {
        definition.validate()?;
        if self.by_id.contains_key(&definition.id) {
            return Err(FactionError::DuplicateFaction {
                id: definition.id.clone(),
            });
        }
        self.by_id.insert(definition.id.clone(), definition);
        Ok(())
    }

    pub fn get(&self, faction_id: &str) -> Option<&FactionDefinition> {
        self.by_id.get(faction_id)
    }

    pub fn by_concept(&self, concept: ConceptKey) -> Option<&FactionDefinition> {
        self.by_id.get(&concept.faction_id())
    }

    /// Every loaded faction ID, ascending. Deterministic, like every other
    /// iteration order in this crate.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.by_id.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Every ID a state's relationships (or any other cross-reference) may
    /// name resolves to a loaded record.
    pub fn require_known(&self, faction_id: &str) -> Result<(), FactionError> {
        if self.by_id.contains_key(faction_id) {
            Ok(())
        } else {
            Err(FactionError::UnknownFaction {
                id: faction_id.to_owned(),
            })
        }
    }
}

/// Where a faction stands on the board (brief section 9). Simulation state,
/// not a player-facing label -- the brief is explicit that these categories
/// need not be surfaced.
///
/// S5 computes which one a faction is in. S1 only stores and serializes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategicState {
    Desperate,
    Recovering,
    Contesting,
    Advantaged,
    Closing,
}

impl Default for StrategicState {
    /// The neutral position -- neither losing nor winning -- and it exists
    /// only so a save written before this field existed loads. It is not a
    /// starting doctrine: S5 recomputes the real state on the first tick.
    fn default() -> Self {
        StrategicState::Contesting
    }
}

/// One faction's live half of a pairwise relationship (brief section 7).
///
/// Independent axes, not a single good-versus-evil alignment: a faction can
/// fear and depend on the same neighbour at once, and temporary cooperation
/// does not silently become permanent alliance. Every axis is `i16` per S1's
/// card, signed because each runs both ways from zero, and zero throughout is
/// "no history yet" -- which is what [`Default`] gives.
///
/// The relationship is directional: `a.relationships["faction.b"]` is what A
/// holds about B, and B holds its own, separate view of A.
///
/// S6 owns how these move. Nothing in S1 changes them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub trust: i16,
    pub fear: i16,
    pub hatred: i16,
    pub grievance: i16,
    pub dependence: i16,
    pub trade_value: i16,
    pub territorial_conflict: i16,
    pub ideological_incompatibility: i16,
    pub recent_aid: i16,
    pub recent_aggression: i16,
    /// How far formal commitment has gone, as a magnitude rather than an enum:
    /// the treaty vocabulary is Provisional (brief section 20, "Pairwise
    /// relationship pressures"), and S1's card specifies every section 7 field
    /// as `i16`. S6 or S15 may promote it once the vocabulary is approved.
    pub treaty_state: i16,
    pub known_betrayal: i16,
    pub perceived_strength: i16,
    pub perceived_opportunity: i16,
}

/// One faction's mutable half: what it holds, where it stands, whether it is
/// still playing, and what it thinks of everyone else.
///
/// This is the only faction data a save carries and the only faction data a
/// strategic tick may write. The authored [`FactionDefinition`] beside it is
/// content and is reloaded, never saved.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionState {
    /// Open-keyed strategic stockpile (brief section 20: the exact resource
    /// list is Open). Content supplies the category strings; this module
    /// defines none, and no enum exists to tempt one into being defined.
    /// Separate from A3's rations / medicine / coin, which are the party's
    /// character-scale purse, not a faction's economy.
    #[serde(default)]
    pub resources: BTreeMap<String, u32>,
    #[serde(default)]
    pub strategic_state: StrategicState,
    /// Eliminated factions stay in the map. S10's recovery chain and brief
    /// section 4's "Elimination consequences" both need the record that used
    /// to be here, so removal is a flag, never a delete.
    #[serde(default)]
    pub eliminated: bool,
    /// What this faction currently holds about each other faction, keyed by
    /// `faction.<concept_key>`. A missing entry is "no relationship formed",
    /// not zeroed history.
    #[serde(default)]
    pub relationships: BTreeMap<String, Relationship>,
    /// S5: what this faction is trying to do right now, best first.
    ///
    /// Appended, `#[serde(default)]`, per the `ExpeditionState` rule
    /// (`docs/SHIP_PLAN.md` section 9): a save written before S5 loads with an
    /// empty list and the first strategic hour fills it.
    ///
    /// Recomputed every strategic hour by
    /// [`crate::strategy::utility::choose_goals`], exactly like
    /// [`FactionState::strategic_state`] beside it, and never set by hand.
    /// It is the *verdict* of S5's utility scoring and carries no scores:
    /// brief section 9 forbids exposing raw utility arithmetic, and
    /// `strategy/utility.rs` holds a test that this struct serializes no
    /// number beyond the ones S1 declared.
    #[serde(default)]
    pub current_goals: Vec<Goal>,
}

impl FactionState {
    /// A faction as it enters the simulation: nothing stockpiled, no history
    /// with anyone, contesting the board.
    pub fn new() -> Self {
        Self::default()
    }

    /// What this faction holds about `other_faction_id` right now, or the
    /// zeroed "no history" view. Reading never inserts.
    pub fn relationship_toward(&self, other_faction_id: &str) -> Relationship {
        self.relationships
            .get(other_faction_id)
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expedition::ExpeditionState;

    /// The rule the repository learned in `battle.rs` and `geography.rs`: when
    /// Rust mirrors authored content, a test holds the two equal.
    ///
    /// S1 has no authored content to mirror -- `content/factions/*.json` is
    /// C9's card and does not exist yet. What S1 *does* own is the closed list
    /// of concepts and the ID shape built from it, so this is the equivalent
    /// test against the brief itself: the six keys are brief section 4's six,
    /// spelled exactly as content must spell them, and every ID is
    /// `faction.<key>` with no room for a proper name. When C9 lands, that
    /// lane adds the fixture-versus-records test this one stands in for.
    #[test]
    fn the_six_concept_keys_are_the_briefs_and_every_id_is_faction_dot_key() {
        let keys: Vec<&str> = ConceptKey::ALL.iter().map(|c| c.as_key()).collect();
        assert_eq!(
            keys,
            vec![
                "fox_people",
                "colonial_powers",
                "pirates",
                "elves",
                "cthulhu",
                "michael"
            ],
            "brief section 4 accepts exactly these six concepts, and refuses proper names"
        );

        for concept in ConceptKey::ALL {
            assert_eq!(
                concept.faction_id(),
                format!("faction.{}", concept.as_key())
            );
            assert_eq!(ConceptKey::from_key(concept.as_key()), Some(concept));
            // The key survives serialization as itself, so content and code
            // spell it the same way.
            let json = serde_json::to_string(&concept).expect("concept serializes");
            assert_eq!(json, format!("\"{}\"", concept.as_key()));
        }

        assert_eq!(ConceptKey::from_key("colonial_power"), None);
        assert_eq!(ConceptKey::from_key("FoxPeople"), None);
    }

    /// C9's records are the owner; this module carries them. The standing rule
    /// from `fixture_matches_the_authored_loot_tables` and
    /// `fixture_matches_the_authored_world_cells`: when Rust mirrors authored
    /// content, a test holds the two equal.
    ///
    /// Every file in `content/factions/` must deserialize into a
    /// `FactionDefinition`, pass `validate`, and load into the registry S4
    /// takes -- and the set of concepts authored must be exactly
    /// `ConceptKey::ALL`, no more and no fewer. A record whose `id` drifted
    /// from its `concept_key` fails here, and a seventh faction cannot be
    /// added by writing a file.
    #[test]
    fn every_authored_faction_record_loads_and_validates() {
        let faction_directory = concat!(env!("CARGO_MANIFEST_DIR"), "/../content/factions/");
        let mut definitions = FactionDefinitions::new();
        let mut authored: BTreeSet<ConceptKey> = BTreeSet::new();
        for entry in std::fs::read_dir(faction_directory).expect("content/factions/ is readable") {
            let path = entry.expect("a readable directory entry").path();
            if path.extension().and_then(|name| name.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("the faction file is readable");
            let record: FactionDefinition = serde_json::from_str(&text).unwrap_or_else(|error| {
                panic!("{} is a FactionDefinition: {error}", path.display())
            });
            record
                .validate()
                .unwrap_or_else(|error| panic!("{} fails validate: {error:?}", path.display()));
            assert!(
                authored.insert(record.concept_key),
                "{} is a second record for concept {}",
                path.display(),
                record.concept_key.as_key()
            );
            definitions
                .insert(record)
                .unwrap_or_else(|error| panic!("{} does not load: {error:?}", path.display()));
        }

        assert_eq!(
            authored,
            ConceptKey::ALL.into_iter().collect::<BTreeSet<_>>(),
            "content/factions/ must author exactly the brief's six concepts"
        );
        assert_eq!(definitions.len(), 6);
        for concept in ConceptKey::ALL {
            assert!(
                definitions
                    .by_concept(concept)
                    .is_some_and(|record| record.id == concept.faction_id()),
                "no authored record for {}",
                concept.faction_id()
            );
        }
    }

    fn a_definition(concept: ConceptKey) -> FactionDefinition {
        FactionDefinition {
            id: concept.faction_id(),
            concept_key: concept,
            ..FactionDefinition::default()
        }
    }

    #[test]
    fn a_record_whose_id_drifted_from_its_concept_is_refused() {
        let mut wrong = a_definition(ConceptKey::Pirates);
        wrong.id = "faction.elves".into();
        assert_eq!(
            wrong.validate(),
            Err(FactionError::IdDoesNotMatchConceptKey {
                found: "faction.elves".into(),
                expected: "faction.pirates".into(),
            })
        );

        let mut unshaped = a_definition(ConceptKey::Pirates);
        unshaped.id = "Pirates".into();
        assert!(matches!(
            unshaped.validate(),
            Err(FactionError::MalformedId(_))
        ));

        assert_eq!(a_definition(ConceptKey::Pirates).validate(), Ok(()));
    }

    #[test]
    fn one_concept_gets_one_record() {
        let mut definitions = FactionDefinitions::new();
        definitions
            .insert(a_definition(ConceptKey::Elves))
            .expect("first record loads");
        assert_eq!(
            definitions.insert(a_definition(ConceptKey::Elves)),
            Err(FactionError::DuplicateFaction {
                id: "faction.elves".into()
            })
        );

        definitions
            .insert(a_definition(ConceptKey::Cthulhu))
            .expect("a second concept loads");
        assert_eq!(
            definitions.ids().collect::<Vec<_>>(),
            vec!["faction.cthulhu", "faction.elves"],
            "iteration is BTreeMap order, so it is the same on every machine"
        );
        assert_eq!(definitions.len(), 2);
        assert!(
            definitions
                .by_concept(ConceptKey::Elves)
                .is_some_and(|record| record.id == "faction.elves")
        );
        assert_eq!(
            definitions.require_known("faction.michael"),
            Err(FactionError::UnknownFaction {
                id: "faction.michael".into()
            })
        );
        assert_eq!(definitions.require_known("faction.elves"), Ok(()));
    }

    #[test]
    fn a_relationship_starts_at_no_history_and_is_read_without_inserting() {
        let state = FactionState::new();
        assert_eq!(state.strategic_state, StrategicState::Contesting);
        assert!(!state.eliminated);
        assert!(
            state.current_goals.is_empty(),
            "a faction wants nothing until its first strategic hour scores the board"
        );
        assert!(state.resources.is_empty());
        assert_eq!(
            state.relationship_toward("faction.pirates"),
            Relationship::default()
        );
        assert!(
            state.relationships.is_empty(),
            "reading a relationship must not create one"
        );
        assert_eq!(Relationship::default().trust, 0);
        assert_eq!(Relationship::default().treaty_state, 0);
    }

    /// The card's Done-when: a two-faction `ExpeditionState` survives
    /// serialize -> deserialize -> compare unchanged.
    ///
    /// This is the test that bites if `factions` loses `#[serde(default)]`,
    /// gains a `skip_serializing`, or has a field renamed under it.
    #[test]
    fn a_two_faction_expedition_state_round_trips_unchanged() {
        let mut state = ExpeditionState::new(
            7,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");

        let mut first = FactionState::new();
        // `resource.example` is a deliberate abstract placeholder: brief
        // section 20 leaves the resource list Open, so a fixture here must not
        // pretend to know a category name.
        first.resources.insert("resource.example".into(), 12);
        first.resources.insert("resource.example_other".into(), 3);
        first.strategic_state = StrategicState::Desperate;
        first.relationships.insert(
            ConceptKey::Pirates.faction_id(),
            Relationship {
                fear: 40,
                hatred: -7,
                dependence: 5,
                treaty_state: 1,
                perceived_opportunity: -12,
                ..Relationship::default()
            },
        );

        first.current_goals = vec![Goal::Recover, Goal::Consolidate];

        let mut second = FactionState::new();
        second.strategic_state = StrategicState::Closing;
        second.eliminated = true;
        second.current_goals = vec![Goal::Pressure];
        second.relationships.insert(
            ConceptKey::Elves.faction_id(),
            Relationship {
                trust: 3,
                known_betrayal: 1,
                ..Relationship::default()
            },
        );

        state
            .factions
            .insert(ConceptKey::Elves.faction_id(), first.clone());
        state
            .factions
            .insert(ConceptKey::Pirates.faction_id(), second.clone());

        let json = state.to_json();
        let reloaded = ExpeditionState::from_json(&json).expect("the save reloads");

        assert_eq!(reloaded, state, "the whole state survives the round trip");
        assert_eq!(reloaded.factions.len(), 2);
        assert_eq!(reloaded.factions.get("faction.elves"), Some(&first));
        assert_eq!(reloaded.factions.get("faction.pirates"), Some(&second));
        assert_eq!(
            reloaded
                .factions
                .get("faction.elves")
                .expect("the first faction reloaded")
                .relationship_toward("faction.pirates")
                .fear,
            40
        );
        assert_eq!(
            reloaded.factions.keys().collect::<Vec<_>>(),
            vec!["faction.elves", "faction.pirates"]
        );
        // Byte-stable: serializing the reloaded state reproduces the same
        // bytes, which is what makes a save hash comparable across a reload.
        assert_eq!(reloaded.to_json(), json);
    }

    /// A save written before `factions` existed still loads, and the strategic
    /// layer starts empty rather than the load failing.
    #[test]
    fn a_save_without_any_factions_still_loads() {
        let state = ExpeditionState::new(
            1,
            vec!["character.protagonist.captain".into()],
            "world.cell.black_beach",
        )
        .expect("a fresh campaign constructs");
        let mut object: serde_json::Value =
            serde_json::from_str(&state.to_json()).expect("a save is JSON");
        object
            .as_object_mut()
            .expect("a save is an object")
            .remove("factions")
            .expect("today's save carries the field this test then removes");

        let older = ExpeditionState::from_json(&object.to_string())
            .expect("a save predating the strategic layer still loads");
        assert!(older.factions.is_empty());
    }
}
