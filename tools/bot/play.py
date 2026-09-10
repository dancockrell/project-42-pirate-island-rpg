#!/usr/bin/env python3
"""A bot that plays the island and complains about it.

It drives the same verbs a player has, through `island_cli`, and keeps a log of
what it tried and what the game did back. Two things come out of a run:

  facts      - what actually happened: actions refused, goals never reached,
               content that never appeared, time spent doing nothing
  complaints - the facts that look like design problems, each with the evidence
               attached so nobody has to take the bot's word for it

The mechanical complaints need no API key. With one, `--judge` also asks Claude
to read the session and say what it was like to play, which catches the things
a rule cannot see: whether a choice mattered, whether the story landed.

  python3 tools/bot/play.py --days 20
  python3 tools/bot/play.py --days 20 --judge
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
            [str(binary)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            bufsize=1,
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


@dataclass
class Journal:
    """What the bot did, what the game said, and what it wants to complain about."""

    actions: list[dict] = field(default_factory=list)
    refusals: list[dict] = field(default_factory=list)
    events: list[str] = field(default_factory=list)
    complaints: list[dict] = field(default_factory=list)

    def act(self, what: str, ok: bool, day: int, **detail):
        self.actions.append({"day": day, "what": what, "ok": ok, **detail})
        if not ok:
            self.refusals.append({"day": day, "what": what, **detail})

    def note(self, day: int, text: str):
        self.events.append(f"day {day}: {text}")

    def complain(self, about: str, severity: str, evidence: str):
        self.complaints.append(
            {"about": about, "severity": severity, "evidence": evidence}
        )


class Player:
    """Plays with intent: survive, get people, follow up on what they say.

    The policy is deliberately the obvious one a new player would try. A bot
    that only wins by knowing the source is no use for finding out whether the
    game explains itself.
    """

    def __init__(self, game: Game, journal: Journal):
        self.game = game
        self.journal = journal
        self.talked_to: set[str] = set()
        self.recruited: list[str] = []
        self.lead_answers: list[dict] = []
        self.idle_days = 0

    @property
    def state(self) -> dict:
        return self.game.state

    @property
    def day(self) -> int:
        return self.state.get("day", 0)

    def women_nearby(self) -> list[dict]:
        """Adult women not already with Michael, nearest first."""
        me = self.state.get("captain") or {"x": 0, "y": 0}
        candidates = [
            p
            for p in self.state.get("people", [])
            if p["sex"] == "female"
            and p["age"]
            and p["age"] >= 18
            and p["faction"] != "faction.michael"
            and p["alive"]
        ]
        candidates.sort(key=lambda p: abs(p["x"] - me["x"]) + abs(p["y"] - me["y"]))
        return candidates

    def try_recruit(self, person: dict) -> bool:
        """Walk to her, talk, ask. The whole loop a player would do."""
        day = self.day
        pid = person["id"]
        moved = self.game.send(cmd="move_captain", to={"x": person["x"], "y": person["y"]})
        self.journal.act("move to person", moved.get("ok", False), day, who=person["name"])

        for _ in range(60):
            self.game.send(cmd="tick", count=25)
            if self.game.send(cmd="approach", id=pid).get("ok"):
                break
        else:
            self.journal.act("approach", False, day, who=person["name"],
                             note="could not get close enough to speak")
            return False

        for _ in range(40):
            if self.game.send(cmd="can_talk", id=pid).get("ok"):
                break
            self.game.send(cmd="tick", count=10)
        else:
            self.journal.act("can_talk", False, day, who=person["name"],
                             note="approached her but never became able to talk")
            return False

        said = self.game.send(cmd="talk", id=pid)
        self.talked_to.add(pid)
        self.journal.act("talk", said.get("ok", False), day, who=person["name"],
                         said=(said.get("said") or "")[:200])

        got = self.game.send(cmd="recruit", id=pid)
        if got.get("ok"):
            self.recruited.append(person["name"])
            self.journal.note(day, f"recruited {person['name']}")
            slot = len(self.recruited) - 1
            if slot < 4:
                self.game.send(cmd="assign", id=pid, slot=slot)
            return True

        self.journal.act("recruit", False, day, who=person["name"],
                         note="talked to her, then the game refused to let me ask")
        return False


    def work_the_wreck(self):
        """Walk to the nearest cache with anything left in it, then strip it."""
        caches = [c for c in self.state.get("salvage_caches", []) if c["remaining"] > 0]
        if not caches:
            return
        me = self.state.get("captain") or {"x": 0, "y": 0}
        caches.sort(key=lambda c: abs(c["x"] - me["x"]) + abs(c["y"] - me["y"]))
        target = caches[0]

        got = self.game.send(cmd="salvage")
        if got.get("ok"):
            self.journal.act("salvage", True, self.day, cache=target["id"])
            return

        self.game.send(cmd="move_captain", to={"x": target["x"], "y": target["y"]})
        for _ in range(50):
            self.game.send(cmd="tick", count=20)
            got = self.game.send(cmd="salvage")
            if got.get("ok"):
                self.journal.act("salvage", True, self.day, cache=target["id"])
                return
        self.journal.act("salvage", False, self.day, cache=target["id"],
                         note=got.get("refused", "walked to the cache and still could not take anything"))

    def answer_leads(self):
        """Answer anything a companion brings, and record what it changed."""
        for lead in list(self.state.get("leads", [])):
            before = dict(self.state)
            choice = lead["interpretations"][0]
            reply = self.game.send(
                cmd="resolve_lead", lead=lead["id"], interpretation=choice["id"]
            )
            self.journal.act("answer lead", reply.get("ok", False), self.day,
                             lead=lead["id"], chose=choice["id"])
            if reply.get("ok"):
                after = self.state
                changed = [
                    k for k in ("provisions", "salvage", "flags", "quests")
                    if before.get(k) != after.get(k)
                ]
                self.lead_answers.append({
                    "lead": lead["id"],
                    "companion": lead["companion"],
                    "observation": lead["observation"],
                    "chose": choice["claim"],
                    "changed": changed,
                })
                self.journal.note(self.day, f"answered {lead['id']}, changed: {changed or 'nothing visible'}")

    def play_day(self):
        """One day: chase salvage, chase people, answer anything asked of you."""
        start = dict(self.state)

        self.answer_leads()

        self.work_the_wreck()

        if len(self.recruited) < 4:
            for person in self.women_nearby()[:2]:
                if person["id"] in self.talked_to:
                    continue
                if self.try_recruit(person):
                    break

        remaining = TICKS_PER_DAY - (self.state["tick"] % TICKS_PER_DAY)
        self.game.send(cmd="tick", count=max(1, remaining))
        self.answer_leads()

        # Did anything the player can see actually change today?
        after = self.state
        visible = ("provisions", "salvage", "party_size", "loyal_companions", "flags")
        if all(start.get(k) == after.get(k) for k in visible) and not self.state.get("leads"):
            self.idle_days += 1


def evaluate(player: Player, journal: Journal, days: int):
    """Turn what happened into complaints, with the evidence attached."""
    state = player.state

    if player.idle_days >= max(3, days // 3):
        journal.complain(
            "Most days nothing the player can see changes",
            "high",
            f"{player.idle_days} of {days} days ended with identical visible state: "
            "no resource change, no companion change, no flag, no lead. The war is "
            "running but the player has no evidence of it.",
        )

    if not player.recruited:
        journal.complain(
            "Could not recruit anyone at all in a full run",
            "high",
            f"Tried {len([a for a in journal.actions if a['what'] == 'talk'])} conversations "
            f"over {days} days and ended alone. Refusals: "
            f"{[r.get('note') for r in journal.refusals if r['what'] == 'recruit'][:3]}",
        )

    talk_fails = [r for r in journal.refusals if r["what"] in ("approach", "can_talk")]
    if len(talk_fails) >= 3:
        journal.complain(
            "Reaching someone to talk to them fails often",
            "medium",
            f"{len(talk_fails)} attempts ended without a conversation. "
            "The player is given no reason why.",
        )

    if not player.lead_answers and state.get("loyal_companions", 0) > 0:
        journal.complain(
            "Recruited companions but was never asked anything",
            "medium",
            f"{state.get('loyal_companions')} companion(s) recruited and no lead opened "
            f"in {days} days. The thing that makes a companion interesting never fired.",
        )

    for answer in player.lead_answers:
        if not answer["changed"]:
            journal.complain(
                f"Answering {answer['lead']} changed nothing the player can see",
                "high",
                f"Backed: {answer['chose'][:120]!r} -- afterwards no resource, flag or "
                "quest state differed. A judgment call with no visible consequence "
                "reads as a fake choice.",
            )

    unfinished = [q for q in state.get("quests", []) if not q["done"]]
    if len(unfinished) == len(state.get("quests", [])) and state.get("quests"):
        journal.complain(
            "No quest resolved in the whole run",
            "medium",
            f"All {len(unfinished)} quests still on their opening stage after {days} days: "
            + ", ".join(q["id"] for q in unfinished[:4]),
        )

    if state.get("salvage", 0) == 0 and any(
        a["what"] == "salvage" and not a["ok"] for a in journal.actions
    ):
        note = next(
            (a.get("note") for a in journal.actions if a["what"] == "salvage" and not a["ok"]),
            "",
        )
        journal.complain(
            "Salvage, the one economic action available early, always refused",
            "medium",
            f"Every attempt refused with: {note!r}. Nothing tells the player what to do instead.",
        )

    dead = [p for p in state.get("people", []) if not p["alive"]]
    if len(dead) > 12:
        journal.complain(
            "The island depopulates while the player watches",
            "low",
            f"{len(dead)} dead by day {state.get('day')}. If the factions grind each "
            "other down this fast, a long campaign has nobody left in it.",
        )


def ask_claude(journal: Journal, player: Player, model: str) -> str | None:
    """Ask Claude what the session was actually like. Optional; needs a key."""
    try:
        import anthropic
    except ImportError:
        return "anthropic SDK not installed (pip install anthropic)"
    if not (os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN")):
        return "no ANTHROPIC_API_KEY set, skipping the judgment pass"

    digest = {
        "days_played": player.day,
        "recruited": player.recruited,
        "leads_answered": player.lead_answers,
        "what_happened": journal.events[-40:],
        "refused_actions": journal.refusals[-25:],
        "final_state": {
            k: player.state.get(k)
            for k in ("day", "provisions", "salvage", "party_size",
                      "loyal_companions", "flags", "quests")
        },
        "mechanical_complaints": journal.complaints,
    }

    try:
        client = anthropic.Anthropic()
        response = client.messages.create(
            model=model,
            max_tokens=4000,
            thinking={"type": "adaptive"},
            output_config={"effort": "medium"},
            system=(
                "You are playtesting an early prototype of a strategy RPG and reporting "
                "back to its developer. You are given a log of one bot-played session.\n\n"
                "Say what would actually make this more interesting to play. Be specific "
                "and concrete: name the moment, say what it felt like, say what you would "
                "change. Do not be encouraging. Do not list what works. If the session log "
                "shows nothing happening, say the game is boring and say why.\n\n"
                "Rank your points by how much they hurt the experience. Prefer 'I recruited "
                "someone and then had no reason to care about her' over abstract notes "
                "about systems."
            ),
            messages=[{
                "role": "user",
                "content": "Here is the session:\n\n" + json.dumps(digest, indent=2),
            }],
        )
        return "".join(b.text for b in response.content if b.type == "text")
    except Exception as error:  # network, auth, rate limit -- the run still stands
        return f"judgment pass failed: {type(error).__name__}: {error}"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--days", type=int, default=15)
    parser.add_argument("--judge", action="store_true", help="also ask Claude to review the session")
    parser.add_argument("--model", default="claude-opus-5")
    parser.add_argument("--out", default=str(REPO / "tools" / "bot" / "last_session.json"))
    args = parser.parse_args()

    if not CLI.exists():
        sys.exit(f"build the driver first: cargo build --manifest-path godot-rust/Cargo.toml --bin island_cli\n(missing {CLI})")

    game = Game(CLI)
    journal = Journal()
    try:
        game.send(cmd="new")
        player = Player(game, journal)
        for _ in range(args.days):
            player.play_day()
            if not player.state.get("captain_alive", True):
                journal.note(player.day, "Michael died; the run ends here")
                break
        evaluate(player, journal, args.days)
    finally:
        game.close()

    verdict = ask_claude(journal, player, args.model) if args.judge else None

    report = {
        "days_played": player.day,
        "commands_sent": game.calls,
        "recruited": player.recruited,
        "leads_answered": player.lead_answers,
        "final_state": {
            k: player.state.get(k)
            for k in ("day", "provisions", "salvage", "party_size",
                      "loyal_companions", "flags")
        },
        "quests": player.state.get("quests", []),
        "complaints": journal.complaints,
        "events": journal.events,
        "refusals": journal.refusals,
        "claude_verdict": verdict,
    }
    Path(args.out).write_text(json.dumps(report, indent=2))

    print(f"played {player.day} days, {game.calls} commands")
    print(f"recruited: {player.recruited or 'nobody'}")
    print(f"leads answered: {len(player.lead_answers)}")
    print()
    if journal.complaints:
        print(f"{len(journal.complaints)} COMPLAINTS")
        for c in journal.complaints:
            print(f"\n  [{c['severity']}] {c['about']}")
            print(f"      {c['evidence']}")
    else:
        print("no complaints -- which for a prototype probably means the bot is not looking hard enough")
    if verdict:
        print("\n--- Claude's verdict ---")
        print(verdict)
    print(f"\nfull session: {args.out}")


if __name__ == "__main__":
    main()
