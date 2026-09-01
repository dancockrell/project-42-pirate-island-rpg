use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct DeathMemory {
    pub killed_by_player_count: u32,
    pub last_death_day: Option<u32>,
    pub last_death_context_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NamedPerson {
    pub id: String,
    pub display_name: String,
    pub alive_today: bool,
    pub death_memory: DeathMemory,
}

impl NamedPerson {
    pub fn record_player_caused_death(&mut self, day: u32, context_id: &str) {
        self.alive_today = false;
        self.death_memory.killed_by_player_count += 1;
        self.death_memory.last_death_day = Some(day);
        self.death_memory.last_death_context_id = Some(context_id.to_owned());
    }

    pub fn return_at_midnight(&mut self) { self.alive_today = true; }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpawnedMonster {
    pub instance_id: String,
    pub definition_id: String,
    pub level: u8,
    pub behavior_tags: [String; 2],
    pub physical_variant: String,
    pub condition: String,
    pub patrol_purpose: String,
    pub loot_seed: u64,
}

#[derive(Clone, Debug)]
pub struct WorldClock {
    pub day: u32,
    pub minute_of_day: u16,
    pub named_people: BTreeMap<String, NamedPerson>,
}

impl WorldClock {
    pub fn advance_to_next_midnight(&mut self) {
        self.day += 1;
        self.minute_of_day = 0;
        for person in self.named_people.values_mut() { person.return_at_midnight(); }
    }

    pub fn level_for_daily_spawn(&self, region_base: u8, pressure: i8) -> u8 {
        let daily_growth = ((self.day.saturating_sub(1)) / 3).min(12) as i16;
        (i16::from(region_base) + daily_growth + i16::from(pressure)).clamp(1, 99) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_people_return_but_remember_player_caused_deaths() {
        let mut person = NamedPerson { id: "person.port.quartermaster".into(), display_name: "Quartermaster".into(),
            alive_today: true, death_memory: DeathMemory { killed_by_player_count: 0, last_death_day: None, last_death_context_id: None } };
        person.record_player_caused_death(4, "encounter.port.argument");
        let mut world = WorldClock { day: 4, minute_of_day: 1439,
            named_people: [(person.id.clone(), person)].into_iter().collect() };
        world.advance_to_next_midnight();
        let returned = world.named_people.get("person.port.quartermaster").unwrap();
        assert!(returned.alive_today);
        assert_eq!(returned.death_memory.killed_by_player_count, 1);
        assert_eq!(returned.death_memory.last_death_day, Some(4));
    }
}

