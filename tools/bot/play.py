#!/usr/bin/env python3
"""Bots that play the island and complain about it.

Each bot drives the same verbs a player has, through `island_cli`. Several play
styles run the same island differently, because a problem only one kind of
player hits is still a problem.

The measurement that matters most is the gap between two things:

  world_delta   - what the simulation did that day (people born and killed,
                  buildings raised, damaged and destroyed, factions ending)
  player_delta  - what changed in anything the player can actually see

A day where the world moved and the player's view did not is the game being
busy at the player rather than with them. Counting both separately is the only
way to tell "nothing happened" from "plenty happened and none of it reached me".

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
    """Saving must not change the game. A save that alters state is a real bug."""
    before = json.dumps(game.state, sort_keys=True)
    reply = game.send(cmd="save")
    after = json.dumps(game.state, sort_keys=True)
    if not reply.get("ok"):
        return [{"day": day, "what": "saving failed", "detail": str(reply)[:200]}]
    if before != after:
        return [{"day": day, "what": "saving the game changed the game",
                 "detail": "state differed before and after a save"}]
    return []


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
        me = self.state.get("captain")
        if not me or self.state.get("salvage", 0) < 4:
            return
        built = self.game.send(cmd="build_foothold", at=me)
        self.journal.act("build foothold", built.get("ok", False), self.day,
                         note=built.get("refused", ""))
        if built.get("ok"):
            self.journal.note(self.day, "built a foothold")

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

    def play_day(self):
        before_world = world_facts(self.state)
        before_player = player_facts(self.state)

        self.answer_leads(prefer=1 if self.style == "contrarian" else 0)

        if self.style != "drifter":
            self.work_the_wreck()
        if self.style == "builder":
            self.build()
        if self.style in ("recruiter", "builder", "contrarian") and len(self.recruited) < 4:
            for person in self.women_nearby()[:2]:
                if person["id"] in self.talked_to:
                    continue
                if self.try_recruit(person):
                    break

        remaining = TICKS_PER_DAY - (self.state["tick"] % TICKS_PER_DAY)
        self.game.send(cmd="tick", count=max(1, remaining))
        self.answer_leads(prefer=1 if self.style == "contrarian" else 0)

        self.journal.bugs.extend(find_bugs(self.state, self.game, self.day))
        if self.day % 5 == 0:
            self.journal.bugs.extend(check_save_round_trip(self.game, self.day))

        self.journal.days.append({
            "day": self.day,
            "world_delta": changed(before_world, world_facts(self.state)),
            "player_delta": changed(before_player, player_facts(self.state)),
        })


def evaluate(player: Player, journal: Journal, days: int):
    state = player.state
    busy_but_silent = [d for d in journal.days if d["world_delta"] and not d["player_delta"]]
    truly_still = [d for d in journal.days if not d["world_delta"] and not d["player_delta"]]

    if len(busy_but_silent) >= max(3, days // 3):
        sample = busy_but_silent[len(busy_but_silent) // 2]
        moved = ", ".join(f"{k} {v[0]}->{v[1]}" for k, v in list(sample["world_delta"].items())[:4])
        journal.complain(
            "The island changes constantly and the player is told none of it",
            "high",
            f"{len(busy_but_silent)} of {days} days moved the world without moving one "
            f"thing the player can see. Day {sample['day']} for instance: {moved}. "
            "A player watching this has no way to know any of it happened.",
        )

    if len(truly_still) >= max(3, days // 3):
        journal.complain(
            "Whole days where the island itself does nothing",
            "medium",
            f"{len(truly_still)} of {days} days changed nothing anywhere, not even "
            "faction population or buildings.",
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
        "leads_answered": player.lead_answers,
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
        busy_silent = sum(1 for d in report["days"] if d["world_delta"] and not d["player_delta"])
        loud = sum(1 for d in report["days"] if d["player_delta"])
        print(f"\n=== {report['style']}: {report['days_played']} days, "
              f"{report['commands']} commands ===")
        print(f"  recruited        {report['recruited'] or 'nobody'}")
        print(f"  leads answered   {len(report['leads_answered'])}")
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
