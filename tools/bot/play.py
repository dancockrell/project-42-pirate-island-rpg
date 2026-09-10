#!/usr/bin/env python3
"""Bots that play the island and complain about it.

Each bot drives the same verbs a player has, through `island_cli`. Several play
styles run the same island differently, because a problem only one kind of
player hits is still a problem.

The measurement that matters most is the gap between what the island does and
what any of it says to the man standing in it:

  world_delta   - what the simulation did that day (people born and killed,
                  buildings raised, damaged and destroyed, factions ending)
  player_delta  - what changed in anything the player can actually see

  news          - what a companion or a local actually told him about the war,
                  asked for with `news` and only ever heard standing beside
                  someone he has already talked to

A day where the world moved and neither the player's view nor anybody's mouth
moved with it is the game being busy at the player rather than with them.
Counting them separately is the only way to tell "nothing happened" from
"plenty happened and none of it reached me". There is no narration feed and
there is not going to be one, so `news` is the whole of the world's voice.

  python3 tools/bot/play.py --days 30
  python3 tools/bot/play.py --days 30 --style all
  python3 tools/bot/play.py --days 30 --judge
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CLI = REPO / "godot-rust" / "target" / "debug" / "island_cli"
TICKS_PER_DAY = 1440
CAPTAIN = "character.protagonist.captain"


class Game:
    """The game as a subprocess. One JSON command in, one JSON reply out."""

    def __init__(self, binary: Path):
        self.proc = subprocess.Popen(
            [str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            text=True, bufsize=1,
        )
        self.state: dict = {}
        self.calls = 0

    def send(self, **command) -> dict:
        self.calls += 1
        self.proc.stdin.write(json.dumps(command) + "\n")
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError("the game stopped responding")
        reply = json.loads(line)
        if "state" in reply:
            self.state = reply["state"]
        return reply

    def close(self):
        try:
            self.send(cmd="quit")
        except Exception:
            pass
        self.proc.terminate()


def world_facts(state: dict) -> dict:
    """Everything true of the island, whether or not the player is looking."""
    people = state.get("people", [])
    buildings = state.get("buildings", [])
    return {
        "alive": sum(1 for p in people if p["alive"]),
        "dead": sum(1 for p in people if not p["alive"]),
        "undead": sum(1 for p in people if p.get("undead")),
        "buildings": len(buildings),
        "building_levels": sum(b["level"] for b in buildings),
        "wrecked": sum(1 for b in buildings if not b["operational"]),
        "factions_alive": len({p["faction"] for p in people if p["alive"]}),
        "eliminated": len(state.get("eliminated_factions", [])),
    }


def player_facts(state: dict) -> dict:
    """Everything the player has actually been told or can act on."""
    return {
        "provisions": state.get("provisions", 0),
        "salvage": state.get("salvage", 0),
        "party_size": state.get("party_size", 0),
        "companions": state.get("loyal_companions", 0),
        "flags": sorted(state.get("flags", [])),
        "open_leads": sorted(l["id"] for l in state.get("leads", [])),
        "quest_stages": sorted(f"{q['id']}:{q['stage']}" for q in state.get("quests", [])),
        "confrontation": state.get("campaign", {}).get("confrontation"),
        "heat_signals": sorted(state.get("campaign", {}).get("heat_signals", [])),
    }


def changed(before: dict, after: dict) -> dict:
    return {k: [before[k], after[k]] for k in after if before.get(k) != after[k]}



def find_bugs(state: dict, game: "Game", day: int) -> list[dict]:
    """Contradictions in what the game says about itself.

    These are correctness bugs, not opinions: two living people with one name,
    a party slot holding someone dead, a count that disagrees with the thing it
    counts. Cheap to check, so it runs every day rather than once at the end.
    """
    bugs: list[dict] = []

    def bug(what: str, detail: str):
        bugs.append({"day": day, "what": what, "detail": detail})

    people = state.get("people", [])
    living = [p for p in people if p["alive"]]

    names: dict[str, list[str]] = {}
    for person in living:
        names.setdefault(person["name"], []).append(person["id"])
    for name, ids in names.items():
        if len(ids) > 1:
            bug("two living people share a name", f"{name!r} is {len(ids)} different people: {ids[:3]}")

    loyal = [p for p in living if p["loyal"]]
    if state.get("loyal_companions", 0) != len(loyal):
        bug("loyal_companions disagrees with the people it counts",
            f"reported {state.get('loyal_companions')}, actually {len(loyal)} living loyal")

    by_id = {p["id"]: p for p in people}
    for slot, occupant in enumerate(state.get("party", [])):
        if not occupant:
            continue
        who = by_id.get(occupant)
        if who is None:
            bug("party slot holds someone who is not on the island",
                f"slot {slot} holds {occupant!r}")
        elif not who["loyal"]:
            bug("party slot holds someone not loyal to Michael",
                f"slot {slot} holds {who['name']}")

    for key in ("salvage", "provisions"):
        if state.get(key, 0) < 0:
            bug("negative resource", f"{key} is {state[key]}")

    for faction in state.get("eliminated_factions", []):
        survivors = [p for p in living if p["faction"] == faction]
        if survivors:
            bug("an eliminated faction still has living people",
                f"{faction} has {len(survivors)}, e.g. {survivors[0]['name']}")

    for lead in state.get("leads", []):
        if len(lead["interpretations"]) != 2:
            bug("a lead does not offer exactly two readings",
                f"{lead['id']} offers {len(lead['interpretations'])}")
        if lead["companion"] and lead["companion"] not in by_id:
            bug("an open lead belongs to someone not on the island",
                f"{lead['id']} belongs to {lead['companion']}")
        if not loyal:
            bug("a lead is open with no companion to have raised it", lead["id"])

    for building in state.get("buildings", []):
        if building["level"] < 1:
            bug("a building has no level", f"{building['id']} level {building['level']}")

    return bugs


def check_save_round_trip(game: "Game", day: int) -> list[dict]:
    """Saving must not change the game, and loading the save back must land on
    exactly the game that was saved -- workshop, dogs, aimed carbine and all.
    A field that survives play but not a save is a real bug."""
    before = json.dumps(game.state, sort_keys=True)
    reply = game.send(cmd="save")
    after = json.dumps(game.state, sort_keys=True)
    if not reply.get("ok"):
        return [{"day": day, "what": "saving failed", "detail": str(reply)[:200]}]
    if before != after:
        return [{"day": day, "what": "saving the game changed the game",
                 "detail": "state differed before and after a save"}]

    if not game.send(cmd="load", save=reply["save"]).get("ok"):
        return [{"day": day, "what": "a save the game just wrote would not load back",
                 "detail": "the game refused its own save"}]
    reloaded = json.dumps(game.state, sort_keys=True)
    if reloaded != before:
        was, now = json.loads(before), json.loads(reloaded)
        differing = [k for k in was if was.get(k) != now.get(k)]
        return [{"day": day, "what": "loading a save does not restore the game that was saved",
                 "detail": f"these differ after an immediate save/load: {differing[:8]}"}]
    return []



def check_determinism(game: "Game", day: int, ticks: int = 120) -> list[dict]:
    """Save, run, reload, run the same again. A deterministic simulation must
    land in exactly the same place both times. This is the strongest test the
    harness has: it exercises every system at once and needs no knowledge of
    what any of them do."""
    saved = game.send(cmd="save")
    if not saved.get("ok"):
        return [{"day": day, "what": "saving failed", "detail": str(saved)[:200]}]
    payload = saved["save"]

    # The same script both times, workshop verbs and carbine included, so a
    # divergence in the new systems is caught the same way as one in the old.
    who = sorted(p["id"] for p in game.state.get("people", []) if p["alive"])
    aimed = game.state.get("aimed_at")

    def run_the_same_day() -> str:
        game.send(cmd="tick", count=ticks // 2)
        game.send(cmd="develop")
        game.send(cmd="queue_machine")
        game.send(cmd="repair")
        if who:
            game.send(cmd="news", id=who[0])
        if aimed:
            game.send(cmd="aim", id=aimed)
        game.send(cmd="tick", count=ticks - ticks // 2)
        return json.dumps(game.state, sort_keys=True)

    first = run_the_same_day()

    if not game.send(cmd="load", save=payload).get("ok"):
        return [{"day": day, "what": "a save the game just wrote would not load back",
                 "detail": "save/load round trip refused its own output"}]
    second = run_the_same_day()
    # Put the world back where the check found it: the probe is allowed to
    # spend the player's salvage, but only inside the check.
    game.send(cmd="load", save=payload)

    if first != second:
        a, b = json.loads(first), json.loads(second)
        differing = [k for k in a if a.get(k) != b.get(k)]
        return [{"day": day, "what": "the simulation is not deterministic across a save",
                 "detail": f"after reloading and running the same {ticks} ticks, these "
                           f"differ: {differing[:6]}"}]
    return []


def check_refusals_are_pure(game: "Game", day: int) -> list[dict]:
    """A refused command must change nothing. A refusal with a side effect is
    worse than a crash, because the game keeps running and quietly lies."""
    bugs = []
    probes = [
        ("recruit", {"cmd": "recruit", "id": "actor.does.not.exist"}),
        ("assign", {"cmd": "assign", "id": "actor.does.not.exist", "slot": 0}),
        ("approach", {"cmd": "approach", "id": ""}),
        ("build_foothold", {"cmd": "build_foothold", "at": {"x": -5, "y": -5}}),
        ("resolve_lead", {"cmd": "resolve_lead", "lead": "lead.nope",
                          "interpretation": "interpretation.nope"}),
        ("move_captain", {"cmd": "move_captain", "to": {"x": 9999, "y": 9999}}),
        # The workshop verbs and the carbine. Each of these spends salvage or
        # starts a war when it works, so a refusal that still changed something
        # is the most expensive kind of lie the game can tell.
        ("restore", {"cmd": "restore", "id": "actor.does.not.exist"}),
        ("aim", {"cmd": "aim", "id": "actor.does.not.exist"}),
        ("aim at himself", {"cmd": "aim", "id": CAPTAIN}),
        ("dismiss", {"cmd": "dismiss", "slot": 99}),
        ("dismiss", {"cmd": "dismiss", "slot": "nonsense"}),
    ]

    # develop, repair and queue_machine are legitimate moves when the workshop
    # is ready and paid for, so they are only probed in a state where the game
    # has no honest way to accept them.
    state = game.state
    salvage = state.get("salvage", 0)
    machine_cost = (list(state.get("machine_costs") or []) + [0, 0, 0])[0]
    berths = (list(state.get("machine_costs") or []) + [0, 0, 0])[2]
    mine = [b for b in state.get("buildings", []) if b["faction"] == "faction.michael"]
    develop_cost = state.get("develop_cost", 0)
    if not mine or not develop_cost or salvage < develop_cost:
        probes.append(("develop", {"cmd": "develop"}))
    if not mine or salvage < state.get("repair_cost", 0):
        probes.append(("repair", {"cmd": "repair"}))
    if not mine or salvage < machine_cost or len(state.get("dogs", [])) >= berths:
        probes.append(("queue_machine", {"cmd": "queue_machine"}))

    for label, probe in probes:
        before = json.dumps(game.state, sort_keys=True)
        reply = game.send(**probe)
        after = json.dumps(game.state, sort_keys=True)
        if reply.get("ok"):
            bugs.append({"day": day, "what": f"the game accepted a nonsense {label}",
                         "detail": json.dumps(probe)})
        elif before != after:
            bugs.append({"day": day, "what": f"a refused {label} still changed the game",
                         "detail": json.dumps(probe)})

    # Asking what someone says is a question, not a move: it must never change
    # the island, and it must never invent an answer from a stranger.
    living = sorted(p["id"] for p in state.get("people", []) if p["alive"])
    for target in ["actor.does.not.exist"] + living[:1]:
        before = json.dumps(game.state, sort_keys=True)
        reply = game.send(cmd="news", id=target)
        after = json.dumps(game.state, sort_keys=True)
        if before != after:
            bugs.append({"day": day, "what": "asking someone for news changed the game",
                         "detail": f"news about {target!r} moved the world"})
        if target == "actor.does.not.exist" and (reply.get("news") or ""):
            bugs.append({"day": day, "what": "somebody who does not exist had news",
                         "detail": str(reply.get("news"))[:160]})
    return bugs


def check_idempotency(game: "Game", day: int, state: dict) -> list[dict]:
    """Doing a done thing twice must be refused, not applied twice. Answering
    the same lead or recruiting the same woman again are the cases a real
    player hits by double-clicking."""
    bugs = []
    loyal = [p for p in state.get("people", []) if p["loyal"] and p["alive"]]
    if loyal:
        target = loyal[0]
        if game.send(cmd="recruit", id=target["id"]).get("ok"):
            bugs.append({"day": day, "what": "recruited someone who had already joined",
                         "detail": f"{target['name']} was recruited a second time"})
        before = state.get("party_size", 0)
        game.send(cmd="assign", id=target["id"], slot=0)
        game.send(cmd="assign", id=target["id"], slot=1)
        after = game.state.get("party_size", 0)
        holders = [i for i, occupant in enumerate(game.state.get("party", []))
                   if occupant == target["id"]]
        if len(holders) > 1:
            bugs.append({"day": day, "what": "one person occupies two party slots",
                         "detail": f"{target['name']} is in slots {holders}"})
        if after > 4:
            bugs.append({"day": day, "what": "party grew past four slots",
                         "detail": f"party_size {before} -> {after}"})

        # Dismissing an empty slot is the double-click of walking away twice.
        held = [i for i, occupant in enumerate(game.state.get("party", []))
                if occupant == target["id"]]
        if held:
            slot = held[0]
            if game.send(cmd="dismiss", slot=slot).get("ok"):
                if game.send(cmd="dismiss", slot=slot).get("ok"):
                    bugs.append({"day": day, "what": "dismissed an empty party slot",
                                 "detail": f"slot {slot} was cleared twice"})
                if game.state.get("party", [])[slot]:
                    bugs.append({"day": day, "what": "a dismissed companion is still in her slot",
                                 "detail": f"slot {slot} still holds {game.state['party'][slot]}"})
                # Put her back: the check is not allowed to cost the run a
                # companion.
                game.send(cmd="assign", id=target["id"], slot=slot)

    # Every workshop verb charges salvage or starts a job. Asking twice must
    # buy one thing, not two: the second call has to be refused.
    for what, command in (("develop the workshop", {"cmd": "develop"}),
                          ("build a mechanical dog", {"cmd": "queue_machine"}),
                          ("repair the workshop", {"cmd": "repair"})):
        first = game.send(**command)
        before = json.dumps(game.state, sort_keys=True)
        second = game.send(**command)
        after = json.dumps(game.state, sort_keys=True)
        if first.get("ok") and second.get("ok"):
            bugs.append({"day": day, "what": f"asked twice to {what} and the game did it twice",
                         "detail": "both calls were accepted back to back"})
        if not second.get("ok") and before != after:
            bugs.append({"day": day, "what": f"a refused request to {what} still changed the game",
                         "detail": json.dumps(command)})

    # Re-aiming at the man already in the sights must be a no-op, not a second
    # war. Only probed when the player has already started one.
    aimed = state.get("aimed_at")
    if aimed:
        game.send(cmd="aim", id=aimed)
        if game.state.get("aimed_at") != aimed:
            bugs.append({"day": day, "what": "aiming again at the same target moved the aim",
                         "detail": f"aimed_at went {aimed!r} -> {game.state.get('aimed_at')!r}"})
    return bugs


@dataclass
class Journal:
    style: str = "?"
    actions: list[dict] = field(default_factory=list)
    refusals: list[dict] = field(default_factory=list)
    events: list[str] = field(default_factory=list)
    complaints: list[dict] = field(default_factory=list)
    days: list[dict] = field(default_factory=list)
    bugs: list[dict] = field(default_factory=list)

    def act(self, what: str, ok: bool, day: int, **detail):
        self.actions.append({"day": day, "what": what, "ok": ok, **detail})
        if not ok:
            self.refusals.append({"day": day, "what": what, **detail})

    def note(self, day: int, text: str):
        self.events.append(f"day {day}: {text}")

    def complain(self, about: str, severity: str, evidence: str):
        self.complaints.append({"style": self.style, "about": about,
                                "severity": severity, "evidence": evidence})


class Player:
    """One play style. The policy is what a person of that temperament would try.

    A bot that only succeeds by knowing the source cannot tell us whether the
    game explains itself, so none of these read anything a player could not.
    """

    def __init__(self, game: Game, journal: Journal, style: str):
        self.game, self.journal, self.style = game, journal, style
        self.talked_to: set[str] = set()
        self.recruited: list[str] = []
        self.lead_answers: list[dict] = []
        self.built = False
        self.idle_because_nothing_to_do = 0
        # Where he put the workshop. The observation carries no building
        # positions, and a player would simply remember where he built it.
        self.workshop_at: dict | None = None
        # The best health each building has ever been seen at. There is no
        # max_health in the observation, so "damaged" means "worse than it was".
        self.building_peak: dict[str, int] = {}
        # The last thing each person Michael has talked to said about the war.
        self.news_heard: dict[str, str] = {}
        self.news_today: list[dict] = []
        self.days_the_world_spoke = 0
        self.machines_ordered = 0
        self.fights_started = 0
        self.restored: list[str] = []
        self.restore_refusals: dict[str, int] = {}
        self.build_refusals = 0
        self.machine_refusals = 0
        self.dogs_seen = 0

    state = property(lambda self: self.game.state)
    day = property(lambda self: self.game.state.get("day", 0))

    # -- the things a player can do -------------------------------------------

    def women_nearby(self) -> list[dict]:
        me = self.state.get("captain") or {"x": 0, "y": 0}
        near = [p for p in self.state.get("people", [])
                if p["sex"] == "female" and (p["age"] or 0) >= 18
                and p["faction"] != "faction.michael" and p["alive"]]
        near.sort(key=lambda p: abs(p["x"] - me["x"]) + abs(p["y"] - me["y"]))
        return near

    def here(self) -> dict:
        return self.state.get("captain") or {"x": 0, "y": 0}

    def walk_to(self, target: dict, tries: int = 12) -> bool:
        """Close distance, re-aiming as she moves. Ticks in proportion to how
        far away she is rather than polling blindly, which is both faster and
        closer to how a player actually chases someone across a board."""
        for _ in range(tries):
            me = self.here()
            gap = abs(target["x"] - me["x"]) + abs(target["y"] - me["y"])
            if gap <= 1:
                return True
            self.game.send(cmd="move_captain", to={"x": target["x"], "y": target["y"]})
            self.game.send(cmd="tick", count=max(8, min(gap * 12, 260)))
            fresh = next((p for p in self.state.get("people", [])
                          if p["id"] == target.get("id")), None)
            if fresh:
                target = fresh
            elif target.get("id"):
                return False  # she died or left the board while we walked
        return abs(target["x"] - self.here()["x"]) + abs(target["y"] - self.here()["y"]) <= 1

    def try_recruit(self, person: dict) -> bool:
        day, pid = self.day, person["id"]
        self.walk_to(person)
        for _ in range(12):
            if self.game.send(cmd="approach", id=pid).get("ok"):
                break
            self.game.send(cmd="tick", count=30)
        else:
            self.journal.act("approach", False, day, who=person["name"],
                             note="never got close enough to speak")
            return False
        for _ in range(20):
            if self.game.send(cmd="can_talk", id=pid).get("ok"):
                break
            self.game.send(cmd="tick", count=20)
        else:
            self.journal.act("can_talk", False, day, who=person["name"],
                             note="reached her but could never start talking")
            return False
        said = self.game.send(cmd="talk", id=pid)
        self.talked_to.add(pid)
        self.journal.act("talk", said.get("ok", False), day, who=person["name"],
                         said=(said.get("said") or "")[:160])
        if self.game.send(cmd="recruit", id=pid).get("ok"):
            self.recruited.append(person["name"])
            self.journal.note(day, f"recruited {person['name']}")
            if len(self.recruited) <= 4:
                self.game.send(cmd="assign", id=pid, slot=len(self.recruited) - 1)
            return True
        self.journal.act("recruit", False, day, who=person["name"],
                         note="talked to her and was then refused")
        return False

    def work_the_wreck(self):
        caches = [c for c in self.state.get("salvage_caches", []) if c["remaining"] > 0]
        if not caches:
            return
        me = self.state.get("captain") or {"x": 0, "y": 0}
        caches.sort(key=lambda c: abs(c["x"] - me["x"]) + abs(c["y"] - me["y"]))
        target = caches[0]
        if self.game.send(cmd="salvage").get("ok"):
            self.journal.act("salvage", True, self.day, cache=target["id"])
            return
        self.walk_to(target)
        for _ in range(6):
            if self.game.send(cmd="salvage").get("ok"):
                self.journal.act("salvage", True, self.day, cache=target["id"])
                return
            self.game.send(cmd="tick", count=30)

    def build(self):
        """The shed wants clear ground, and where Michael happens to be
        standing after a day of talking to people usually is not. So try where
        he is, then the wreck he already knows, then a few steps off."""
        me = self.state.get("captain")
        if not me or self.state.get("salvage", 0) < 4:
            return
        spots = [dict(me)]
        for cache in sorted(self.state.get("salvage_caches", []), key=lambda c: c["id"]):
            spots.append({"x": cache["x"], "y": cache["y"]})
        spots += [{"x": me["x"] + dx, "y": me["y"] + dy}
                  for dx, dy in ((4, 0), (-4, 0), (0, 4), (0, -4))]

        for spot in spots[:4]:
            if spot != self.here():
                self.walk_to(spot, tries=3)
            at = self.here()
            built = self.game.send(cmd="build_foothold", at=at)
            self.journal.act("build foothold", built.get("ok", False), self.day,
                             at=at, note=built.get("refused", ""))
            if built.get("ok"):
                self.built = True
                self.workshop_at = {"x": at["x"], "y": at["y"]}
                self.journal.note(self.day, "built a foothold")
                return
        self.build_refusals += 1

    # -- the workshop ---------------------------------------------------------

    def workshop(self) -> dict | None:
        """The one building Michael owns, as the player sees it."""
        for building in self.state.get("buildings", []):
            if building["faction"] == "faction.michael":
                return building
        return None

    def remember_buildings(self):
        for building in self.state.get("buildings", []):
            self.building_peak[building["id"]] = max(
                self.building_peak.get(building["id"], 0), building["health"])

    def workshop_ready(self) -> bool:
        shed = self.workshop()
        return bool(shed) and shed["operational"]

    def workshop_damaged(self) -> bool:
        shed = self.workshop()
        if not shed:
            return False
        return shed["health"] < self.building_peak.get(shed["id"], shed["health"])

    def go_to_workshop(self) -> bool:
        """Every workshop verb wants Michael standing beside it. Since
        move_captain moves the household, this brings the companions and the
        dogs along too, which is what restoring someone needs."""
        if not self.workshop_at:
            return False
        return self.walk_to(dict(self.workshop_at), tries=6)

    def workshop_command(self, what: str, **command):
        self.go_to_workshop()
        reply = self.game.send(**command)
        self.journal.act(what, reply.get("ok", False), self.day,
                         note=reply.get("refused", ""))
        return reply

    def mind_the_workshop(self):
        """Raising the shed and repairing it only advance while Michael is
        standing on the site, so 'finish the workshop' is a thing he spends a
        day doing rather than a thing he orders and walks away from."""
        if not self.go_to_workshop():
            self.journal.act("mind the workshop", False, self.day,
                             note="could not get back to the workshop site")
            return
        for _ in range(6):
            self.game.send(cmd="tick", count=120)
            shed = self.workshop()
            self.remember_buildings()
            if not shed or (shed["operational"] and not self.workshop_damaged()):
                break
            if not self.go_to_workshop():
                break
        self.journal.act("mind the workshop", self.workshop_ready(), self.day)

    def blamed_salvage_he_had(self, what: str, reply: dict, cost: int):
        """A refusal that names the wrong reason is worse than a refusal. If
        the game says salvage while the player is looking at enough salvage,
        the real reason is something it never told him."""
        note = reply.get("refused") or ""
        salvage = self.state.get("salvage", 0)
        if not reply.get("ok") and cost and salvage >= cost and "salvage" in note.lower():
            self.journal.bugs.append({
                "day": self.day,
                "what": f"a refusal to {what} blamed salvage the player had in hand",
                "detail": f"{note!r} with {salvage} salvage stored and a cost of {cost}",
            })

    def develop_workshop(self):
        reply = self.workshop_command("develop workshop", cmd="develop")
        # The cost is read back out of the same reply that carried the refusal,
        # so a price that moved while Michael walked over cannot be mistaken
        # for the game lying about why it said no.
        self.blamed_salvage_he_had("build the workshop out", reply,
                                   self.state.get("develop_cost", 0))
        if reply.get("ok"):
            self.journal.note(self.day, "started building the workshop out")

    def repair_workshop(self):
        reply = self.workshop_command("repair workshop", cmd="repair")
        self.blamed_salvage_he_had("repair the workshop", reply,
                                   self.state.get("repair_cost", 0))
        if reply.get("ok"):
            self.journal.note(self.day, "started repairing the workshop")

    def order_machine(self):
        reply = self.workshop_command("build a mechanical dog", cmd="queue_machine")
        cost, _ticks, berths = (list(self.state.get("machine_costs") or []) + [0, 0, 0])[:3]
        dogs = len(self.state.get("dogs", []))
        self.blamed_salvage_he_had("build a mechanical dog", reply, cost)
        if not reply.get("ok") and dogs < berths and "berth" in (reply.get("refused") or "").lower():
            self.journal.bugs.append({
                "day": self.day,
                "what": "the workshop reports a free dog berth and then refuses to use it",
                "detail": f"{reply.get('refused')!r} with {dogs} dogs and {berths} berths",
            })
        if reply.get("ok"):
            self.machine_refusals = 0
            self.machines_ordered += 1
            self.journal.note(self.day, "put a mechanical dog on the workshop bench")
        else:
            # Three identical noes in a row is the game saying no, whatever the
            # observation advertises. Stop calling it an available action.
            self.machine_refusals += 1

    def drowned_companions(self) -> list[dict]:
        """His own dead, still walking. Anyone the workshop has refused three
        times is dropped, so one impossible case cannot swallow every day."""
        return sorted(
            (p for p in self.state.get("people", [])
             if p.get("undead") and p["alive"] and p["faction"] == "faction.michael"
             and self.restore_refusals.get(p["id"], 0) < 3),
            key=lambda p: p["id"])

    def restore_companion(self):
        drowned = self.drowned_companions()
        if not drowned:
            return
        self.go_to_workshop()
        for person in drowned[:3]:
            reply = self.game.send(cmd="restore", id=person["id"])
            self.journal.act("restore a companion", reply.get("ok", False), self.day,
                             who=person["name"], note=reply.get("refused", ""))
            if reply.get("ok"):
                self.restored.append(person["name"])
                self.journal.note(self.day, f"brought {person['name']} back")
                return
            self.restore_refusals[person["id"]] = \
                self.restore_refusals.get(person["id"], 0) + 1

    # -- the carbine ----------------------------------------------------------

    def strangers(self) -> list[dict]:
        me = self.here()
        others = [p for p in self.state.get("people", [])
                  if p["alive"] and p["faction"] != "faction.michael"]
        others.sort(key=lambda p: (abs(p["x"] - me["x"]) + abs(p["y"] - me["y"]), p["id"]))
        return others

    def pick_a_fight(self):
        """Aiming the carbine is the only way the player starts anything, so a
        style that never aims never sees the half of the game that shoots back."""
        for person in self.strangers()[:2]:
            self.walk_to(person, tries=4)
            reply = self.game.send(cmd="aim", id=person["id"])
            self.journal.act("aim the carbine", reply.get("ok", False), self.day,
                             who=person["name"], faction=person["faction"])
            if reply.get("ok"):
                self.fights_started += 1
                self.journal.note(self.day, f"aimed at {person['name']} of {person['faction']}")
                return

    def regroup(self):
        """Clear a slot whose occupant is dead, gone or no longer his."""
        by_id = {p["id"]: p for p in self.state.get("people", [])}
        for slot, occupant in enumerate(self.state.get("party", [])):
            if not occupant:
                continue
            who = by_id.get(occupant)
            if who and who["alive"] and who["loyal"]:
                continue
            reply = self.game.send(cmd="dismiss", slot=slot)
            self.journal.act("dismiss a party slot", reply.get("ok", False), self.day,
                             slot=slot, who=(who or {}).get("name", occupant))
            if reply.get("ok"):
                self.journal.note(self.day, f"cleared party slot {slot}")
                return

    def stale_party_slot(self) -> bool:
        by_id = {p["id"]: p for p in self.state.get("people", [])}
        for occupant in self.state.get("party", []):
            if not occupant:
                continue
            who = by_id.get(occupant)
            if not who or not who["alive"] or not who["loyal"]:
                return True
        return False

    # -- what the world tells him ---------------------------------------------

    def ask_the_news(self):
        """`news` is the game's only channel for what the war is doing, and it
        only opens for someone Michael has talked to and is standing beside.
        A day where somebody told him something new is a day the world reached
        him, whether or not any number on his own sheet moved."""
        self.news_today = []
        for pid in sorted(self.talked_to):
            person = next((p for p in self.state.get("people", [])
                           if p["id"] == pid and p["alive"]), None)
            if not person:
                continue
            if not self.game.send(cmd="can_talk", id=pid).get("ok"):
                continue
            reply = self.game.send(cmd="news", id=pid)
            said = (reply.get("news") or "").strip()
            if not said:
                continue
            if self.news_heard.get(pid) != said:
                self.news_heard[pid] = said
                self.news_today.append({"who": person["name"], "said": said[:200]})
                self.journal.note(self.day, f"{person['name']}: {said[:120]}")
        if self.news_today:
            self.days_the_world_spoke += 1

    def answer_leads(self, prefer: int = 0):
        for lead in list(self.state.get("leads", [])):
            before = player_facts(self.state)
            choice = lead["interpretations"][prefer % len(lead["interpretations"])]
            reply = self.game.send(cmd="resolve_lead", lead=lead["id"],
                                   interpretation=choice["id"])
            self.journal.act("answer lead", reply.get("ok", False), self.day,
                             lead=lead["id"], chose=choice["id"])
            if reply.get("ok"):
                self.lead_answers.append({
                    "lead": lead["id"], "companion": lead["companion"],
                    "observation": lead["observation"], "chose": choice["claim"],
                    "changed": changed(before, player_facts(self.state)),
                })
                self.journal.note(self.day, f"answered {lead['id']}")

    # -- a day ----------------------------------------------------------------

    def goals(self) -> list[tuple[int, str]]:
        """What is worth doing right now, best first.

        Scored rather than scripted, so the bot spends its day on whatever the
        game has actually made available -- which is also how it notices when
        the game has made nothing available.
        """
        state = self.state
        wants: list[tuple[int, str]] = []

        if state.get("leads"):
            wants.append((100, "answer"))

        if len(self.recruited) < 4 and self.women_nearby():
            # Wanting company more when he has none is what a person would do.
            wants.append((80 - 12 * len(self.recruited), "recruit"))

        caches = [c for c in state.get("salvage_caches", []) if c["remaining"] > 0]
        if caches and self.style != "drifter":
            # Salvage matters until there is enough to build with.
            wants.append((70 if state.get("salvage", 0) < 8 else 30, "salvage"))

        salvage = state.get("salvage", 0)
        shed = self.workshop()
        builder = self.style == "builder"

        # The workshop and everything it makes possible. Michael cannot raise
        # soldiers, so the shed is the only thing he can actually grow, and a
        # bot that could not touch it reported an empty day whenever it had
        # salvage in hand and a shed to spend it on.
        if not shed and salvage >= 4 and self.style != "drifter" \
                and self.build_refusals < 3:
            wants.append((90 if builder else 65, "build"))

        if shed and (not shed["operational"] or self.workshop_damaged()):
            # Building work only advances with Michael standing on the site, so
            # an unfinished shed is a day's work rather than an order.
            wants.append((93 if builder else 60, "finish"))

        if self.workshop_ready() and self.workshop_damaged() \
                and salvage >= state.get("repair_cost", 0):
            # A wrecked workshop stops everything else, whoever you are.
            wants.append((95, "repair"))

        develop_cost = state.get("develop_cost", 0)
        if self.workshop_ready() and develop_cost and salvage >= develop_cost:
            wants.append((92 if builder else 55, "develop"))

        machine_cost, _ticks, berths = (list(state.get("machine_costs") or []) + [0, 0, 0])[:3]
        if self.workshop_ready() and machine_cost and salvage >= machine_cost \
                and len(state.get("dogs", [])) < berths and self.machine_refusals < 3:
            wants.append((88 if builder else 50, "machine"))

        if self.workshop_ready() and salvage > 0 and self.drowned_companions():
            # Getting a drowned companion back beats almost anything else.
            wants.append((94, "restore"))

        if self.style == "contrarian" and not state.get("aimed_at") and self.strangers() \
                and state.get("party_size", 0) >= 2:
            # Only the contrarian starts fights, and the carbine is the only
            # way anyone starts one at all. Scored below the day's honest work
            # so he picks the fight when he has run out of better ideas and has
            # somebody standing with him, rather than on the second morning.
            wants.append((35, "fight"))

        if self.stale_party_slot():
            wants.append((45, "regroup"))

        if self.style == "drifter":
            wants.append((40, "wander"))

        wants.sort(reverse=True)
        return wants

    def wander(self):
        """Walk somewhere the player has not been. A game that only rewards
        standing still is worth knowing about."""
        buildings = self.state.get("buildings", [])
        if not buildings:
            return
        target = buildings[self.day % len(buildings)]
        spot = next((p for p in self.state.get("people", [])
                     if p["faction"] == target["faction"]), None)
        if spot:
            self.walk_to(spot, tries=4)

    def play_day(self):
        before_world = world_facts(self.state)
        before_player = player_facts(self.state)
        self.remember_buildings()
        dogs = len(self.state.get("dogs", []))
        if dogs != self.dogs_seen:
            # The bench moved: whatever it refused last time is worth one more
            # honest try.
            self.dogs_seen, self.machine_refusals = dogs, 0

        # Two actions a day: enough to make progress, few enough that a day
        # with nothing worth doing is visible as exactly that.
        for _, goal in self.goals()[:2]:
            if goal == "answer":
                self.answer_leads(prefer=1 if self.style == "contrarian" else 0)
            elif goal == "recruit":
                for person in self.women_nearby()[:3]:
                    if person["id"] in self.talked_to:
                        continue
                    if self.try_recruit(person):
                        break
            elif goal == "salvage":
                self.work_the_wreck()
            elif goal == "build":
                self.build()
            elif goal == "finish":
                self.mind_the_workshop()
            elif goal == "develop":
                self.develop_workshop()
            elif goal == "repair":
                self.repair_workshop()
            elif goal == "machine":
                self.order_machine()
            elif goal == "restore":
                self.restore_companion()
            elif goal == "fight":
                self.pick_a_fight()
            elif goal == "regroup":
                self.regroup()
            elif goal == "wander":
                self.wander()
        else:
            if not self.goals():
                self.idle_because_nothing_to_do += 1

        remaining = TICKS_PER_DAY - (self.state["tick"] % TICKS_PER_DAY)
        self.game.send(cmd="tick", count=max(1, remaining))
        self.answer_leads(prefer=1 if self.style == "contrarian" else 0)
        self.remember_buildings()
        self.ask_the_news()

        self.journal.bugs.extend(find_bugs(self.state, self.game, self.day))
        if self.day % 4 == 0:
            self.journal.bugs.extend(check_save_round_trip(self.game, self.day))
            self.journal.bugs.extend(check_refusals_are_pure(self.game, self.day))
        if self.day % 7 == 0:
            self.journal.bugs.extend(check_idempotency(self.game, self.day, self.state))
        if self.day % 9 == 0:
            self.journal.bugs.extend(check_determinism(self.game, self.day))

        self.journal.days.append({
            "day": self.day,
            "world_delta": changed(before_world, world_facts(self.state)),
            "player_delta": changed(before_player, player_facts(self.state)),
            # What a companion or a local actually told him today. The owner
            # has vetoed a narration feed, so this is the whole of the world's
            # voice: a day with news in it is a day that reached the player.
            "news": self.news_today,
        })


def evaluate(player: Player, journal: Journal, days: int):
    state = player.state
    busy_but_silent = [d for d in journal.days
                       if d["world_delta"] and not d["player_delta"] and not d.get("news")]
    truly_still = [d for d in journal.days if not d["world_delta"] and not d["player_delta"]]

    if len(busy_but_silent) >= max(3, days // 3):
        sample = busy_but_silent[len(busy_but_silent) // 2]
        moved = ", ".join(f"{k} {v[0]}->{v[1]}" for k, v in list(sample["world_delta"].items())[:4])
        journal.complain(
            "The island changes constantly and the player is told none of it",
            "high",
            f"{len(busy_but_silent)} of {days} days moved the world without moving one "
            f"thing the player can see and without one person telling him anything new. "
            f"Day {sample['day']} for instance: {moved}. "
            "A player watching this has no way to know any of it happened.",
        )

    if len(truly_still) >= max(3, days // 3):
        journal.complain(
            "Whole days where the island itself does nothing",
            "medium",
            f"{len(truly_still)} of {days} days changed nothing anywhere, not even "
            "faction population or buildings.",
        )

    if player.idle_because_nothing_to_do >= max(3, days // 4):
        journal.complain(
            "The game regularly offers the player nothing to do", "high",
            f"On {player.idle_because_nothing_to_do} of {days} days there was no lead to "
            "answer, nobody left to recruit, no salvage to collect, nothing to build, "
            "no workshop work he could pay for, nobody to bring back and no party slot "
            "to sort out. Not 'the player chose to wait' -- the game had no available "
            "action.",
        )

    spoke = [d for d in journal.days if d.get("news")]
    if len(spoke) <= days // 10:
        journal.complain(
            "Nobody ever tells the player what the war is doing", "high",
            f"On {len(spoke)} of {days} days did a companion or a local say anything new "
            "about the fighting, and that is the game's only channel for it.",
        )

    if not player.recruited:
        journal.complain(
            "Ended the run alone", "high",
            f"{len([a for a in journal.actions if a['what'] == 'talk'])} conversations, "
            f"nobody joined. Refusals: {[r.get('note') for r in journal.refusals][:3]}",
        )

    for answer in player.lead_answers:
        if not answer["changed"]:
            journal.complain(
                f"Answering {answer['lead']} changed nothing visible", "high",
                f"Backed {answer['chose'][:100]!r} and no flag, resource or quest moved.",
            )

    unfinished = [q for q in state.get("quests", []) if not q["done"]]
    if state.get("quests") and len(unfinished) == len(state["quests"]):
        journal.complain(
            "Not one quest resolved", "medium",
            f"All {len(unfinished)} still on their opening stage after {days} days.",
        )

    deaths = [d for d in journal.days if "dead" in d["world_delta"]]
    if len(deaths) > days * 0.6:
        last = deaths[-1]["world_delta"]["dead"]
        journal.complain(
            "The factions annihilate each other while the player watches", "medium",
            f"People died on {len(deaths)} of {days} days, ending at {last[1]} dead. "
            "Nothing asks the player to care or lets them intervene.",
        )

    talk_fails = [r for r in journal.refusals if r["what"] in ("approach", "can_talk")]
    if len(talk_fails) >= 3:
        journal.complain(
            "Getting close enough to talk to someone often just fails", "medium",
            f"{len(talk_fails)} attempts ended with no conversation and no reason given.",
        )


def ask_claude(reports: list[dict], model: str) -> str | None:
    try:
        import anthropic
    except ImportError:
        return "anthropic SDK not installed"
    if not (os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN")):
        return "no ANTHROPIC_API_KEY set, skipping the judgment pass"
    try:
        client = anthropic.Anthropic()
        response = client.messages.create(
            model=model, max_tokens=4000,
            thinking={"type": "adaptive"},
            output_config={"effort": "medium"},
            system=(
                "You are playtesting an early prototype of a strategy RPG and reporting to "
                "its developer. You get logs of several bot-played sessions in different "
                "styles.\n\n"
                "Say what would make this more interesting to play. Be concrete: name the "
                "moment, say what it felt like, say what you would change. Do not be "
                "encouraging and do not list what works. If the logs show a player with "
                "nothing to react to, say the game is boring and say exactly why.\n\n"
                "Rank by how much each hurts. Prefer 'I recruited four women and none of "
                "them ever mattered' over abstract notes about systems."
            ),
            messages=[{"role": "user",
                       "content": "Sessions:\n\n" + json.dumps(reports, indent=2)[:120000]}],
        )
        return "".join(b.text for b in response.content if b.type == "text")
    except Exception as error:
        return f"judgment pass failed: {type(error).__name__}: {error}"


STYLES = ["recruiter", "builder", "drifter", "contrarian"]


def run_style(style: str, days: int) -> dict:
    game, journal = Game(CLI), Journal(style=style)
    try:
        game.send(cmd="new")
        player = Player(game, journal, style)
        for _ in range(days):
            player.play_day()
            if not player.state.get("captain_alive", True):
                journal.note(player.day, "Michael died; run over")
                break
        evaluate(player, journal, days)
    finally:
        game.close()
    return {
        "style": style,
        "days_played": player.day,
        "commands": game.calls,
        "recruited": player.recruited,
        "restored": player.restored,
        "machines_ordered": player.machines_ordered,
        "dogs": player.state.get("dogs", []),
        "fights_started": player.fights_started,
        "news_heard": player.news_heard,
        "days_the_world_spoke": player.days_the_world_spoke,
        "leads_answered": player.lead_answers,
        "days_with_nothing_worth_doing": player.idle_because_nothing_to_do,
        "final_player_view": player_facts(player.state),
        "final_world": world_facts(player.state),
        "days": journal.days,
        "complaints": journal.complaints,
        "events": journal.events,
        "refusals": journal.refusals[:30],
        "bugs": journal.bugs,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--days", type=int, default=20)
    parser.add_argument("--style", default="all", choices=STYLES + ["all"])
    parser.add_argument("--judge", action="store_true")
    parser.add_argument("--model", default="claude-opus-5")
    parser.add_argument("--out", default=str(REPO / "tools" / "bot" / "last_session.json"))
    args = parser.parse_args()

    if not CLI.exists():
        sys.exit(f"build it first: cargo build --manifest-path godot-rust/Cargo.toml --bin island_cli")

    styles = STYLES if args.style == "all" else [args.style]
    reports = [run_style(style, args.days) for style in styles]

    for report in reports:
        busy_silent = sum(1 for d in report["days"]
                          if d["world_delta"] and not d["player_delta"] and not d.get("news"))
        loud = sum(1 for d in report["days"] if d["player_delta"] or d.get("news"))
        print(f"\n=== {report['style']}: {report['days_played']} days, "
              f"{report['commands']} commands ===")
        print(f"  recruited        {report['recruited'] or 'nobody'}")
        print(f"  leads answered   {len(report['leads_answered'])}")
        print(f"  workshop         {len(report['dogs'])} dog(s), "
              f"{report['machines_ordered']} ordered, "
              f"{len(report['restored'])} restored, "
              f"{report['fights_started']} fight(s) started")
        print(f"  days with nothing to do: {report['days_with_nothing_worth_doing']}")
        print(f"  days somebody told them something new: {report['days_the_world_spoke']}")
        print(f"  days the player saw something happen: {loud}/{len(report['days'])}")
        print(f"  days the world moved and they saw none of it: {busy_silent}")

    all_bugs = [b for r in reports for b in r.get("bugs", [])]
    if all_bugs:
        seen = {}
        for b in all_bugs:
            seen.setdefault(b["what"], []).append(b)
        print(f"\n{len(all_bugs)} BUG SIGHTING(S), {len(seen)} distinct")
        for what, hits in seen.items():
            print(f"\n  ! {what}  (x{len(hits)}, first day {hits[0]['day']})")
            print(f"      {hits[0]['detail']}")
    else:
        print("\nno invariant violations seen")

    everything = [c for r in reports for c in r["complaints"]]
    print(f"\n{len(everything)} COMPLAINTS across {len(reports)} play style(s)")
    for c in sorted(everything, key=lambda c: {"high": 0, "medium": 1, "low": 2}[c["severity"]]):
        print(f"\n  [{c['severity']}] ({c['style']}) {c['about']}")
        print(f"      {c['evidence']}")

    verdict = ask_claude(reports, args.model) if args.judge else None
    if verdict:
        print("\n--- Claude's verdict ---")
        print(verdict)

    Path(args.out).write_text(json.dumps(
        {"reports": reports, "claude_verdict": verdict}, indent=2))
    print(f"\nfull sessions: {args.out}")


if __name__ == "__main__":
    main()
