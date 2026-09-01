from pathlib import Path
from docx import Document
from docx.shared import Inches, Pt, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.enum.section import WD_SECTION
from docx.enum.style import WD_STYLE_TYPE
from docx.enum.table import WD_TABLE_ALIGNMENT, WD_CELL_VERTICAL_ALIGNMENT
from docx.oxml import OxmlElement
from docx.oxml.ns import qn

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "work" / "art"
OUT = ROOT / "outputs" / "Project_42_Pirate_Island_RPG_Design_Bible.docx"

INK = "231F1A"; BRONZE = "8B642F"; GOLD = "C59A52"; DEEP = "17201F"; TEAL = "295B58"
PALE = "F3EEE3"; MUTED = "6E665B"; WHITE = "FFFFFF"; RULE = "C9B58E"

def set_cell_shading(cell, fill):
    tcPr = cell._tc.get_or_add_tcPr(); shd = tcPr.find(qn('w:shd'))
    if shd is None: shd = OxmlElement('w:shd'); tcPr.append(shd)
    shd.set(qn('w:fill'), fill)

def set_cell_margins(cell, top=90, start=120, bottom=90, end=120):
    tc = cell._tc; tcPr = tc.get_or_add_tcPr(); tcMar = tcPr.first_child_found_in('w:tcMar')
    if tcMar is None: tcMar = OxmlElement('w:tcMar'); tcPr.append(tcMar)
    for m,v in [('top',top),('start',start),('bottom',bottom),('end',end)]:
        node = tcMar.find(qn('w:'+m))
        if node is None: node=OxmlElement('w:'+m); tcMar.append(node)
        node.set(qn('w:w'), str(v)); node.set(qn('w:type'),'dxa')

def font(run, size=10.5, bold=False, italic=False, color=INK, name="Aptos"):
    run.font.name=name; run._element.get_or_add_rPr().rFonts.set(qn('w:ascii'),name); run._element.rPr.rFonts.set(qn('w:hAnsi'),name)
    run.font.size=Pt(size); run.bold=bold; run.italic=italic; run.font.color.rgb=RGBColor.from_string(color)

def border_bottom(p, color=BRONZE, size=10):
    pPr=p._p.get_or_add_pPr(); pBdr=OxmlElement('w:pBdr'); bottom=OxmlElement('w:bottom')
    bottom.set(qn('w:val'),'single'); bottom.set(qn('w:sz'),str(size)); bottom.set(qn('w:space'),'5'); bottom.set(qn('w:color'),color)
    pBdr.append(bottom); pPr.append(pBdr)

def keep_with_next(p): p.paragraph_format.keep_with_next=True

def add_p(doc, text="", bold_lead=None, style=None, after=8, align=None, italic=False):
    p=doc.add_paragraph(style=style); p.paragraph_format.space_after=Pt(after); p.paragraph_format.line_spacing=1.22
    if align is not None: p.alignment=align
    if bold_lead and text.startswith(bold_lead):
        font(p.add_run(bold_lead), bold=True); font(p.add_run(text[len(bold_lead):]), italic=italic)
    else: font(p.add_run(text), italic=italic)
    return p

def add_h(doc, text, level=1):
    p=doc.add_paragraph(text, style=f'Heading {level}'); keep_with_next(p); return p

def add_bullets(doc, items):
    for item in items:
        p=doc.add_paragraph(style='List Bullet'); p.paragraph_format.space_after=Pt(4); p.paragraph_format.line_spacing=1.15; font(p.add_run(item))

def add_numbered(doc, items):
    numbering = doc.part.numbering_part.element
    abstract_ids = [int(x.get(qn('w:abstractNumId'))) for x in numbering.findall(qn('w:abstractNum'))]
    num_ids = [int(x.get(qn('w:numId'))) for x in numbering.findall(qn('w:num'))]
    abstract_id = max(abstract_ids, default=-1) + 1
    num_id = max(num_ids, default=0) + 1
    abstract = OxmlElement('w:abstractNum'); abstract.set(qn('w:abstractNumId'), str(abstract_id))
    multi = OxmlElement('w:multiLevelType'); multi.set(qn('w:val'), 'singleLevel'); abstract.append(multi)
    lvl = OxmlElement('w:lvl'); lvl.set(qn('w:ilvl'), '0')
    start = OxmlElement('w:start'); start.set(qn('w:val'), '1'); lvl.append(start)
    num_fmt = OxmlElement('w:numFmt'); num_fmt.set(qn('w:val'), 'decimal'); lvl.append(num_fmt)
    lvl_text = OxmlElement('w:lvlText'); lvl_text.set(qn('w:val'), '%1.'); lvl.append(lvl_text)
    suff = OxmlElement('w:suff'); suff.set(qn('w:val'), 'tab'); lvl.append(suff)
    ppr = OxmlElement('w:pPr'); tabs = OxmlElement('w:tabs'); tab = OxmlElement('w:tab'); tab.set(qn('w:val'),'num'); tab.set(qn('w:pos'),'720'); tabs.append(tab); ppr.append(tabs)
    ind = OxmlElement('w:ind'); ind.set(qn('w:left'),'720'); ind.set(qn('w:hanging'),'360'); ppr.append(ind); lvl.append(ppr); abstract.append(lvl); numbering.append(abstract)
    num = OxmlElement('w:num'); num.set(qn('w:numId'), str(num_id)); abs_id = OxmlElement('w:abstractNumId'); abs_id.set(qn('w:val'), str(abstract_id)); num.append(abs_id); numbering.append(num)
    for item in items:
        p=doc.add_paragraph(); p.paragraph_format.space_after=Pt(4); p.paragraph_format.line_spacing=1.15
        numPr=OxmlElement('w:numPr'); ilvl=OxmlElement('w:ilvl'); ilvl.set(qn('w:val'),'0'); n=OxmlElement('w:numId'); n.set(qn('w:val'),str(num_id)); numPr.append(ilvl); numPr.append(n); p._p.get_or_add_pPr().append(numPr)
        font(p.add_run(item))

def set_table_geometry(t, widths):
    dxa=[round(w*1440) for w in widths]
    total=sum(dxa)
    tblPr=t._tbl.tblPr
    tblW=tblPr.find(qn('w:tblW'))
    if tblW is None: tblW=OxmlElement('w:tblW'); tblPr.append(tblW)
    tblW.set(qn('w:type'),'dxa'); tblW.set(qn('w:w'),str(total))
    tblInd=tblPr.find(qn('w:tblInd'))
    if tblInd is None: tblInd=OxmlElement('w:tblInd'); tblPr.append(tblInd)
    tblInd.set(qn('w:type'),'dxa'); tblInd.set(qn('w:w'),'120')
    grid=t._tbl.tblGrid
    for child in list(grid): grid.remove(child)
    for width in dxa:
        col=OxmlElement('w:gridCol'); col.set(qn('w:w'),str(width)); grid.append(col)
    for row in t.rows:
        for cell,width in zip(row.cells,dxa):
            tcPr=cell._tc.get_or_add_tcPr(); tcW=tcPr.find(qn('w:tcW'))
            if tcW is None: tcW=OxmlElement('w:tcW'); tcPr.append(tcW)
            tcW.set(qn('w:type'),'dxa'); tcW.set(qn('w:w'),str(width))

def add_callout(doc, label, text):
    t=doc.add_table(rows=1, cols=1); t.alignment=WD_TABLE_ALIGNMENT.LEFT; t.autofit=False; t.columns[0].width=Inches(6.5)
    c=t.cell(0,0); set_cell_shading(c,'E9E2D3'); set_cell_margins(c,150,120,150,120)
    p=c.paragraphs[0]; p.paragraph_format.space_after=Pt(3); font(p.add_run(label.upper()+"  "),9,bold=True,color=BRONZE); font(p.add_run(text),10.2,color=INK)
    set_table_geometry(t,[6.5])
    doc.add_paragraph().paragraph_format.space_after=Pt(1)

def add_table(doc, headers, rows, widths=None):
    t=doc.add_table(rows=1, cols=len(headers)); t.alignment=WD_TABLE_ALIGNMENT.LEFT; t.autofit=False
    if widths is None: widths=[6.5/len(headers)]*len(headers)
    for i,(h,w) in enumerate(zip(headers,widths)):
        c=t.rows[0].cells[i]; c.width=Inches(w); set_cell_shading(c,DEEP); set_cell_margins(c); c.vertical_alignment=WD_CELL_VERTICAL_ALIGNMENT.CENTER
        p=c.paragraphs[0]; font(p.add_run(h),9,bold=True,color=WHITE)
    for row in rows:
        cells=t.add_row().cells
        for i,(value,w) in enumerate(zip(row,widths)):
            cells[i].width=Inches(w); set_cell_margins(cells[i]); cells[i].vertical_alignment=WD_CELL_VERTICAL_ALIGNMENT.TOP
            if len(t.rows)%2==1: set_cell_shading(cells[i],PALE)
            p=cells[i].paragraphs[0]; font(p.add_run(str(value)),9.2)
    set_table_geometry(t,widths)
    for row in t.rows:
        trPr=row._tr.get_or_add_trPr()
        trPr.append(OxmlElement('w:cantSplit'))
        for cell in row.cells:
            tcPr=cell._tc.get_or_add_tcPr(); tcW=tcPr.find(qn('w:tcW')); tcW.set(qn('w:type'),'dxa')
    doc.add_paragraph().paragraph_format.space_after=Pt(1)
    return t

def add_figure(doc, filename, caption, purpose):
    p=doc.add_paragraph(); p.alignment=WD_ALIGN_PARAGRAPH.CENTER; p.paragraph_format.space_before=Pt(5); p.paragraph_format.space_after=Pt(5)
    p.add_run().add_picture(str(ART/filename), width=Inches(6.5))
    cp=doc.add_paragraph(); cp.paragraph_format.keep_with_next=False; cp.paragraph_format.space_after=Pt(10)
    font(cp.add_run(caption+" "),9,bold=True,color=BRONZE); font(cp.add_run(purpose),9,italic=True,color=MUTED)

def page_break(doc): doc.add_page_break()

doc=Document(); sec=doc.sections[0]
sec.page_width=Inches(8.5); sec.page_height=Inches(11); sec.top_margin=Inches(.82); sec.bottom_margin=Inches(.78); sec.left_margin=Inches(1); sec.right_margin=Inches(1); sec.header_distance=Inches(.35); sec.footer_distance=Inches(.35)

styles=doc.styles
normal=styles['Normal']; normal.font.name='Aptos'; normal.font.size=Pt(10.5); normal.font.color.rgb=RGBColor.from_string(INK); normal.paragraph_format.space_after=Pt(8); normal.paragraph_format.line_spacing=1.22
for name,size,before,after,color in [('Title',31,0,8,DEEP),('Subtitle',13,0,10,MUTED),('Heading 1',18,18,8,DEEP),('Heading 2',14,14,6,TEAL),('Heading 3',11.5,10,4,BRONZE)]:
    s=styles[name]; s.font.name='Aptos Display' if name!='Normal' else 'Aptos'; s.font.size=Pt(size); s.font.color.rgb=RGBColor.from_string(color); s.font.bold=name not in ('Subtitle',)
    s.paragraph_format.space_before=Pt(before); s.paragraph_format.space_after=Pt(after); s.paragraph_format.keep_with_next=True
for lname in ['List Bullet','List Number']:
    s=styles[lname]; s.font.name='Aptos'; s.font.size=Pt(10.2); s.paragraph_format.left_indent=Inches(.38); s.paragraph_format.first_line_indent=Inches(-.18); s.paragraph_format.space_after=Pt(4)

# Header/footer
hp=sec.header.paragraphs[0]; hp.alignment=WD_ALIGN_PARAGRAPH.LEFT; border_bottom(hp,color=RULE,size=5); font(hp.add_run('PROJECT 42  /  PIRATE ISLAND RPG  /  DESIGN BIBLE'),8,bold=True,color=MUTED)
fp=sec.footer.paragraphs[0]; fp.alignment=WD_ALIGN_PARAGRAPH.CENTER; font(fp.add_run('Design pass: executable game contract  |  Godot 4  |  September 2026'),8,color=MUTED)

# Cover
add_p(doc,'PROJECT 42',after=10,align=WD_ALIGN_PARAGRAPH.CENTER).runs[0].font.color.rgb=RGBColor.from_string(BRONZE)
p=doc.add_paragraph(style='Title'); p.alignment=WD_ALIGN_PARAGRAPH.CENTER; p.paragraph_format.space_before=Pt(64); font(p.add_run('PIRATE ISLAND RPG'),31,bold=True,color=DEEP,name='Aptos Display')
p=doc.add_paragraph(); p.alignment=WD_ALIGN_PARAGRAPH.CENTER; font(p.add_run('DESIGN BIBLE'),18,bold=True,color=BRONZE,name='Aptos Display'); p.paragraph_format.space_after=Pt(16); border_bottom(p,color=BRONZE,size=14)
add_p(doc,'A modern 2D party RPG about dangerous exploration, an impossible tropical island, and the household that forms when people from incompatible histories choose to survive together.',after=20,align=WD_ALIGN_PARAGRAPH.CENTER,italic=True)
add_figure(doc,'battle-ui.png','Cover reference — the battle screen as a stage.','The party remains legible as cards until an actor is chosen; Betty rises into the scene at fighting-game scale while the background, enemies, command rail and bronze frame remain readable as a single composition.')
add_p(doc,'Pass: product coherence, system boundaries and development-risk prevention',after=0,align=WD_ALIGN_PARAGRAPH.CENTER)
page_break(doc)

add_h(doc,'How to use this bible',1)
add_p(doc,"This bible is the source specification for an AI or human team building Project 42. Read Part A first. It defines authority, identifiers, ownership, dependency order, change control and acceptance. Later chapters contain domain specifications. Descriptive paragraphs explain purpose and player experience. Normative requirements appear only in labeled contract tables, build steps, schemas, invariants and acceptance gates. If two normative records conflict, stop implementation, report both stable IDs and resolve the contradiction through the decision ledger. Never silently choose the later sentence, the more detailed sentence or the easier implementation.")
add_callout(doc,'Design promise',"The world may be obscure, dangerous, capricious, and occasionally unfair. The software must remain lucid. Mystery belongs to the island; confusion must not come from the interface.")
add_h(doc,'The document at a glance',2)
add_table(doc,['Part','Question it answers'],[
('I. The Island and the Player','What sort of place is this, and what does inhabiting it feel like?'),('II. The Play Structure','How do travel, danger, knowledge, combat and recovery reinforce one another?'),('III. The Household','How do romance, loyalty and party capability become one system without reducing characters to rewards?'),('IV. Sites and Antagonists','What lies beneath the island, and what is learning from the party?'),('V. Production','What is the smallest truthful prototype, and how is it built in Godot?')],[1.75,4.75])
add_h(doc,'Scope posture',2)
add_p(doc,"The complete game is intended to feel abundant, but abundance must come from reuse with memory rather than from indiscriminate volume. Thirty to forty meaningful overworld locations are more valuable than several hundred interchangeable nodes. Twelve major sites can each possess rules of their own. Six core heroines can sustain hundreds of combinations if their relationships with the protagonist, with one another, and with the house are allowed to develop in parallel. The visual pipeline should produce libraries of poses, effects, frames, and environmental states that recombine cleanly; it should not invite a stream of one-off images that cannot be animated, revised, or kept consistent.")

page_break(doc)
add_h(doc,'A. AI implementation protocol',1)
add_p(doc,"Part A controls how another AI reads, decomposes, implements and verifies the entire game. It contains process rules rather than setting prose. Every implementation task begins by loading the smallest relevant domain package plus its declared dependencies. The agent records the stable IDs it used, produces the requested artifact, runs the listed validator and writes evidence into the build ledger. This locality rule keeps the working context small while preserving exact relationships between systems.")

add_h(doc,'A.1 Normative language and precedence',2)
add_table(doc,['Token','Meaning','Required AI behavior'],[
('MUST','A build, data, content, accessibility or integrity requirement.','Implement exactly; fail validation when violated.'),
('MUST NOT','A prohibited state, dependency or behavior.','Reject content or code that creates it.'),
('DOES','A locked shipped behavior visible to the player.','Preserve until a decision record explicitly replaces it.'),
('TARGET','A measured performance, pacing, quantity or quality goal.','Instrument and report the measured result.'),
('CANDIDATE','A bounded option awaiting proof.','Keep outside production content until its gate passes.'),
('RATIONALE','Explanation of why a contract exists.','Use for design judgment; never treat as an additional hidden rule.'),
('EXAMPLE','One valid instance.','Do not generalize beyond the governing schema or invariant.')],[.9,2.05,3.55])
add_numbered(doc,[
"Load the authority record for the requested domain.",
"Load every dependency listed by stable ID. Do not load unrelated chapters as precautionary context.",
"Translate the request into requirement IDs, asset IDs, data types, commands, emitted events and acceptance gates.",
"Report a contradiction when two loaded normative records cannot both be true. Pause only the affected task.",
"Implement through the authoritative owner. Presentation code, scene nodes and prose never mutate simulation truth directly.",
"Run schema validation, deterministic tests, content-integrity tests and the domain acceptance gate.",
"Record evidence, unresolved risks and any decision request in the build ledger before starting the next package."
])

add_h(doc,'A.2 Single-home rule',2)
add_callout(doc,'Locality invariant',"Every normative fact has exactly one authoritative home. Other sections refer to its stable ID and may explain consequences. They never restate the rule with altered wording. A repeated number, formula, state transition, content count or prohibition is a documentation defect.")
add_table(doc,['Information class','Authoritative home','Permitted references'],[
('Product promise and exclusions','PROD domain','Experience prose, acceptance tests and milestone rationale.'),
('Runtime state and commands','SIM domain','UI previews, pseudocode and test fixtures.'),
('Combat formula and status behavior','COMBAT domain','Skill definitions, enemy definitions and encounter fixtures.'),
('Campaign, chapter and time state','CAMPAIGN domain','Quest conditions, journal presentation and estate schedules.'),
('Relationship and harem rules','REL domain','Scene eligibility, pair arcs, skill unlocks and household behavior.'),
('World facts, cultures and naming','WORLD domain','Site packages, dialogue, journals and art briefs.'),
('Content quantities and production budget','SCOPE domain','Backlog, validators and milestone gates.'),
('Godot ownership and resource schemas','TECH domain','Builders, importers, debug tools and automated tests.'),
('Visual, animation and audio constraints','PRES domain','Asset packages and presentation acceptance captures.')],[1.55,2.15,2.8])

add_h(doc,'A.3 Stable identifier grammar',2)
add_p(doc,"Every requirement, decision, data record, content package, test and asset uses a permanent lowercase identifier. Display names may change. IDs remain stable after entering a shared build. References always store IDs; they never store headings, filenames, Godot node paths or translated display text.")
add_table(doc,['Class','Pattern','Example','Owner'],[
('Requirement','req.<domain>.<subject>.<verb>','req.combat.turn.resolve','Specification registry'),
('Invariant','inv.<domain>.<subject>','inv.sim.single_writer','Domain validator'),
('Decision','dec.<domain>.<sequence>','dec.scope.0042','Decision ledger'),
('Data type','type.<domain>.<name>','type.combat.ability_def','Schema registry'),
('Command','cmd.<domain>.<verb>','cmd.combat.commit_action','Simulation owner'),
('Event','evt.<domain>.<past_tense>','evt.combat.action_resolved','Event registry'),
('Content','<class>.<namespace>.<slug>','ability.betty.healing_impact','Content database'),
('Asset','asset.<domain>.<owner>.<purpose>','asset.actor.betty.battle_idle','Asset registry'),
('Test','test.<domain>.<behavior>','test.combat.reaction_order','Test catalog'),
('Package','pkg.<domain>.<slug>','pkg.site.veyra_tidehouse','Package manifest')],[1.0,1.75,2.25,1.5])
add_callout(doc,'Identifier correction',"The example spelling rules are enforced by registry validation. Character namespace IDs are betty, ayla, vix, grisha, isabella and nara. A misspelled ID fails import even when its display label appears correct.")

add_h(doc,'A.4 Domain registry and permitted dependencies',2)
domain_rows=[
('PROD','Product identity, pillars, exclusions and player promise.','None.','All domains.'),
('SCOPE','Shipped counts, duration, content budget and production limits.','PROD','All content domains, QA.'),
('SIM','Authoritative commands, state owners, events, transactions and RNG.','PROD','Every runtime domain.'),
('CAMPAIGN','Chapters, time, quests, factions, knowledge and world consequences.','PROD, SCOPE, SIM','EXPEDITION, REL, ESTATE, WORLD.'),
('EXPEDITION','Party selection, routes, sites, camps, supplies, retreat and aftermath.','SIM, CAMPAIGN, WORLD','COMBAT, ESTATE, REL.'),
('COMBAT','Bands, initiative, actions, resources, statuses, abilities and enemies.','SIM, SCOPE','PRES, REL, EXPEDITION.'),
('REL','Household membership, authored romance milestones, heroine-pair arcs and bond ranks.','SIM, CAMPAIGN, CAST','COMBAT unlocks, ESTATE, NARRATIVE.'),
('CAST','Protagonist and heroine identity, biography, equipment and performance grammar.','PROD, WORLD','REL, COMBAT, PRES, NARRATIVE.'),
('WORLD','Island history, cultures, ecology, temporal rules, places and naming.','PROD','CAMPAIGN, EXPEDITION, CAST, NARRATIVE, PRES.'),
('ESTATE','Rooms, residents, schedules, upgrades, recovery and household events.','SIM, CAMPAIGN, REL','EXPEDITION preparation, NARRATIVE, PRES.'),
('NARRATIVE','Text surfaces, evidence, dialogue, scene packages and journal extraction.','CAMPAIGN, REL, CAST, WORLD','UI presentation and localization.'),
('PRES','Cards, active actor, camera, VFX, UI, animation, art and audio.','SIM, COMBAT, CAST, WORLD','Runtime scenes only.'),
('TECH','Godot folders, schemas, services, import, saves, debug and build pipeline.','All authoritative domains','Implementation and QA.'),
('QA','Validators, fixtures, acceptance captures, performance budgets and release gates.','All domains','Release decision.')]
add_table(doc,['Domain','Owns','May depend on','Consumed by'],domain_rows,[.65,2.0,1.55,2.3])
add_callout(doc,'Dependency prohibition',"A lower-level owner never imports a presentation or narrative package to decide simulation truth. COMBAT does not query animation completion to apply damage. REL does not inspect a rendered dialogue line to change trust. WORLD does not read a scene node to determine whether a fort exists.")

add_h(doc,'A.5 Requirement-record schema',2)
add_table(doc,['Field','Required content','Validation'],[
('id','Permanent req.* identifier.','Unique and never reused.'),
('status','locked, target, candidate, deprecated or superseded.','Enum; superseded records name replacement.'),
('owner','One domain and one runtime state owner.','Must exist in domain registry.'),
('statement','One testable rule using one principal verb.','No rationale, examples or second requirement.'),
('inputs','Typed commands, referenced IDs and preconditions.','Every type and ID resolves.'),
('outputs','State delta, event, player feedback or generated artifact.','At least one observable output.'),
('invariants','inv.* records preserved before and after execution.','Every invariant has an automated check.'),
('dependencies','Requirement IDs that must already hold.','Acyclic dependency graph.'),
('failure','Blocked result and player/developer explanation.','No silent fallback.'),
('verification','Named test IDs and evidence artifact.','All tests pass before status is implemented.')],[1.15,3.0,2.35])
add_p(doc,"When a paragraph contains more than one independently falsifiable claim, split it into separate requirement records. When several records share the same input and output shape, define one schema and reference it. This prevents both prose drift and schema duplication.")

add_h(doc,'A.6 Authoritative state-owner matrix',2)
add_table(doc,['State','Single writer','Legal mutation entry','Required event'],[
('CampaignState','CampaignService','cmd.campaign.commit_transition','evt.campaign.transition_committed'),
('ExpeditionState','ExpeditionService','cmd.expedition.commit_action','evt.expedition.action_committed'),
('BattleState','BattleService','cmd.combat.commit_action','evt.combat.action_resolved'),
('RelationshipState','RelationshipService','cmd.rel.apply_authored_change','evt.rel.state_changed'),
('HouseholdState','EstateService','cmd.estate.commit_project_or_schedule','evt.estate.state_changed'),
('SiteState','SiteService','cmd.site.commit_interaction','evt.site.feature_changed'),
('KnowledgeState','KnowledgeService','cmd.knowledge.record_proposition','evt.knowledge.proposition_recorded'),
('WorldPopulationState','WorldService','cmd.world.resolve_midnight','evt.world.midnight_resolved'),
('AdaptationState','AdaptiveDirector','cmd.adaptation.observe_resolved_event','evt.adaptation.pattern_updated')],[1.2,1.25,2.45,1.6])
add_bullets(doc,[
"Commands validate against an immutable snapshot and return either Blocked(reason_id) or an atomic delta plus ordered events.",
"A service applies its delta once, appends the event ledger, then offers the event to presentation and dependent services.",
"Presentation may delay, accelerate, skip or replay visual sequences. It never changes the committed result.",
"Autosaves occur only at declared transaction boundaries. A save never contains half an action, half a relationship change or a removed cost without its result.",
"Debug tools mutate state only by issuing the same commands as production systems or by loading a labeled fixture outside a player save."
])

add_h(doc,'A.7 Content-package contract',2)
add_p(doc,"Every authored unit ships as a package with a manifest. A package may be a heroine, ability kit, enemy family, route edge, travel event, site, quest, household scene, romance scene, estate project or chapter transition. Package locality means an AI can build and validate one unit without reading the entire bible.")
add_table(doc,['Manifest field','Rule'],[
('package_id','Permanent pkg.* ID and schema version.'),
('domain','One primary owner; secondary consumers listed separately.'),
('purpose','One sentence naming the player decision or state change delivered.'),
('requires','Stable package, requirement, type and asset IDs. No filename-only dependencies.'),
('provides','Commands, data records, scenes, text blocks, assets, tests and debug fixtures exported by the package.'),
('entry_conditions','Typed predicates evaluated by the owning service.'),
('exit_results','Every possible state delta, event and authored aftermath key.'),
('variation_budget','Exact number of variants by state, party, romance, difficulty and localization class.'),
('performance_budget','Memory, load, draw, animation and text limits relevant to the package.'),
('acceptance','Requirement IDs, test IDs, captures and human review gates.'),
('change_log','Decision IDs and migration notes since the last shipped schema.')],[1.55,4.95])

add_h(doc,'A.8 Build order and stop gates',2)
build_steps=[
('B00','Freeze registries','Domain IDs, requirement IDs, content namespaces and state owners validate.','No feature code before this gate.'),
('B01','Build simulation kernel','Commands, atomic deltas, ordered events, seeded RNG and event ledger pass deterministic fixtures.','No scenes or content-specific branches.'),
('B02','Build content database','Typed resources load, reference and validate without Godot scenes.','No unresolved IDs, duplicate IDs or node-path identity.'),
('B03','Build save and migration harness','Every authoritative state serializes, reloads and migrates across archived fixtures.','No campaign content before round-trip success.'),
('B04','Build one complete combat transaction','Four cards, one expansion, one action, reaction, damage, status, retreat and return resolve from commands.','Gray-box presentation only.'),
('B05','Build one expedition transaction chain','Estate departure, route, site, camp, battle, retreat/return and aftermath share one ExpeditionState.','One deterministic seed and ledger.'),
('B06','Build text and knowledge pipeline','Observation, testimony, inference, confidence, journal extraction and action unlocks remain attributable.','No arbitrary adjective assembly.'),
('B07','Build relationship and household pipeline','Authored milestone flags, pair arcs, household membership, schedules, rooms and skill unlocks validate together.','No generic attraction economy or legal agreement simulator.'),
('B08','Build vertical-slice content packages','Betty, Ayla, estate, two routes, one tomb, enemies, Champion and aftermath meet package manifests.','No second major site.'),
('B09','Replace gray-box presentation','Identity packs, card expansion, full-body animation, VFX, UI, audio and accessibility pass captures.','No cropped body, weapon or VFX envelope.'),
('B10','Run complete-loop proof','Fresh start through return and changed estate passes deterministic, usability, save, performance and content audits.','Expansion begins only after the proof report is approved.')]
add_table(doc,['Gate','Build package','Pass evidence','Stop condition'],build_steps,[.55,1.5,3.0,1.45])

add_h(doc,'A.9 AI task packet',2)
add_p(doc,"Every task sent to an implementation AI uses the following fields. Missing fields are a planning defect. The receiving AI may derive subtasks within the declared package; it may not expand scope into another domain without a new packet.")
add_table(doc,['Field','Required value'],[
('objective','One observable result stated in player or build terms.'),
('package','One pkg.* owner and its version.'),
('requirements','Complete req.* set needed to judge completion.'),
('dependencies','Already-approved package versions and generated interfaces.'),
('inputs','Exact source files, resource IDs, fixtures and reference assets.'),
('outputs','Exact files, resources, scenes, tests, captures and ledger entries to create or edit.'),
('prohibitions','Applicable inv.* records and out-of-scope domains.'),
('procedure','Ordered construction steps with a checkpoint after each irreversible or high-risk step.'),
('verification','Commands or tools, test IDs, expected results and required visual review.'),
('handoff','Changed IDs, assumptions, evidence, remaining risks and next legal gate.')],[1.25,5.25])
add_callout(doc,'No inferred completion',"A task is complete only when every declared output exists and every verification record passes. Plausible code, a successful import, a clean screenshot or a narrative summary alone never satisfies a gate.")

add_h(doc,'A.10 Change control and contradiction handling',2)
add_numbered(doc,[
"Open a decision record with a permanent dec.* ID. Name the conflicting requirement IDs and the player or production problem.",
"List viable replacements, affected packages, migrations, deleted assumptions and proof required. Preserve the current rule until a replacement is approved.",
"Approve one option with an owner and effective schema version. Mark replaced requirements superseded; never erase their IDs.",
"Update the authoritative home first. Update references by ID. A search for the old value must find only the decision history, migration and explicitly labeled legacy assets.",
"Run dependency, save-migration, content-integrity and affected-domain acceptance suites. Record evidence in the build ledger.",
"Reject a change that cannot identify its downstream consumers or migration effect."
])
add_table(doc,['Ledger field','Required content'],[
('build_id','Immutable build or document revision identifier.'),('package_versions','Exact package manifest versions used.'),('requirements_loaded','Sorted req.* IDs consulted.'),('artifacts_changed','Stable IDs plus paths generated from them.'),('tests','Test ID, seed, result and evidence location.'),('visual_review','Capture set, reviewer, defects and resolution.'),('decisions','New or consumed dec.* records.'),('risks','Specific unresolved risk, owner and blocking gate.')],[1.5,5.0])

add_h(doc,'A.11 Canonical requirement registry',2)
add_p(doc,"This registry is the normative index for the current design pass. Each record owns one behavior. Detailed chapters explain context, enumerate content or provide examples; their implementation instructions resolve back to these IDs. New normative rules enter this registry before they enter code or content.")
core_requirements=[
('req.prod.loop.cross_consequence','PROD','LOCKED','Every expedition result changes at least one persistent campaign, household, relationship, knowledge, faction, injury, inventory, site or adaptation state.','test.prod.loop_cross_consequence'),
('req.prod.text.actionable','PROD','LOCKED','Every mandatory text surface contributes actionable observation, decision context, relationship state or consequence.','test.narrative.text_utility'),
('req.scope.party.active_count','SCOPE','LOCKED','The field party contains the protagonist plus exactly three selected heroines.','test.scope.party_count'),
('req.scope.cast.core','SCOPE','LOCKED','The shipped core cast contains the protagonist and six heroines: Betty, Ayla, Vix, Grisha, Isabella and Nara.','test.scope.cast_manifest'),
('req.scope.campaign.duration','SCOPE','TARGET','Median first completion is twenty-four hours; accepted first-completion range is twenty to thirty hours.','test.telemetry.campaign_duration'),
('req.sim.command.atomic','SIM','LOCKED','Every accepted command commits one validated atomic delta and an ordered event list.','test.sim.atomic_command'),
('req.sim.preview.same_rules','SIM','LOCKED','Command preview and command resolution call the same rule definition with different immutable snapshots.','test.sim.preview_resolution_parity'),
('req.sim.rng.seeded','SIM','LOCKED','All gameplay randomness is generated by named deterministic streams stored in authoritative state.','test.sim.seed_replay'),
('req.sim.presentation.read_only','SIM','LOCKED','Presentation consumes committed events and cannot mutate authoritative gameplay state.','test.sim.presentation_write_rejected'),
('req.sim.save.boundary','SIM','LOCKED','Autosaves and manual saves capture complete transaction boundaries only.','test.save.transaction_boundary'),
('req.campaign.time.authored','CAMPAIGN','LOCKED','Time advances through declared estate activities, route segments, site actions, camps and chapter transitions.','test.campaign.time_sources'),
('req.campaign.deadline.explicit','CAMPAIGN','LOCKED','Every deadline exposes its fiction wording, mechanical advances remaining and possible consequence class.','test.campaign.deadline_preview'),
('req.campaign.knowledge.provenance','CAMPAIGN','LOCKED','Every journal proposition records source, time, place, evidence class, confidence and contradiction links.','test.knowledge.provenance_complete'),
('req.expedition.state.single','EXPEDITION','LOCKED','One ExpeditionState owns party, objective, route position, time, fatigue, injuries, load, site state references and ledger until aftermath commits.','test.expedition.state_owner'),
('req.expedition.party.change_anchors','EXPEDITION','LOCKED','Party composition changes only at the estate, a supplied legal field camp, an owned vessel with route access or an authored reunion.','test.expedition.party_change_legality'),
('req.expedition.route.commit','EXPEDITION','LOCKED','Route travel begins only after a RouteCommand records edge, pace, load forecast, known hazards and retreat destination.','test.expedition.route_commit'),
('req.expedition.retreat.command','EXPEDITION','LOCKED','Retreat is a legal command with stated condition, time, cost, destination and aftermath.','test.expedition.retreat_complete'),
('req.combat.round.activation','COMBAT','LOCKED','Each conscious actor receives one activation per round unless a named status or command explicitly changes it.','test.combat.activation_count'),
('req.combat.initiative.formula','COMBAT','LOCKED','Round initiative is Speed plus seeded d6; ties resolve by Precision, party priority and stable actor ID.','test.combat.initiative_order'),
('req.combat.bands.five','COMBAT','LOCKED','Battle position uses Party Rear, Party Front, Contested, Enemy Front and Enemy Rear.','test.combat.band_enum'),
('req.combat.reaction.token','COMBAT','LOCKED','Each actor begins a round with one reaction token and spends it only through a declared reaction trigger.','test.combat.reaction_budget'),
('req.combat.target.commit','COMBAT','LOCKED','Targets lock when the command commits; invalidated targets resolve the ability fallback and never retarget silently.','test.combat.target_fallback'),
('req.combat.hit.formula','COMBAT','LOCKED','Hit chance is clamp(50,95,75+5*(Precision-Evasion)).','test.combat.hit_formula'),
('req.combat.damage.formula','COMBAT','LOCKED','Damage is max(1,Potency+Power+d6-Armor-GuardSpent).','test.combat.damage_formula'),
('req.combat.status.typed','COMBAT','LOCKED','Every status belongs to Body, Control, Mind, Defense or Tempo and declares source, duration, stack, cleanse, tags, icon and deterministic expiry.','test.combat.status_schema'),
('req.combat.skill.functional_name','COMBAT','LOCKED','Every skill display name states its action or tactical result in ordinary language.','test.content.skill_name_review'),
('req.narrative.action_file_style','NARRATIVE','LOCKED','Rules, bios, skills and production instructions use direct action-file prose: actor, action, target, result and constraint.','test.narrative.action_file_lint'),
('req.rel.household.stable','REL','LOCKED','The harem is a stable informal household, not a contract, council, voting body or negotiated rules simulation.','test.rel.no_contract_state'),
('req.rel.format.mfff','REL','LOCKED','The shipped romance format is MFFF+: one male protagonist and a growing household of adult women.','test.rel.cast_format'),
('req.rel.hero.admired','REL','LOCKED','The heroines judge the protagonist to be heroic, capable, honorable and worthy of their respect, love and sexual desire.','test.rel.heroine_admiration_scenes'),
('req.rel.willing.participation','REL','LOCKED','Romance, sex, recruitment and household participation occur because the adult women visibly want them and volunteer. The protagonist never forces compliance.','test.rel.voluntary_scene_direction'),
('req.rel.positive_fantasy','REL','LOCKED','The romance campaign delivers love, admiration, happiness, fanservice and shared victory; it does not analyze relationship failure or use chronic romantic drama as progression.','test.rel.positive_scene_mix'),
('req.rel.exclusivity.genre','REL','LOCKED','Committed heroines have no romantic or sexual relationships with anyone outside the harem. Female-female relationships inside the harem are part of the harem and do not violate the protagonist-romance promise.','test.rel.romance_targets'),
('req.rel.no_cheating','REL','LOCKED','No heroine cheats, leaves for another partner or becomes an outside character romance after joining the harem.','test.rel.no_rugpull_routes'),
('req.rel.no_futa','REL','LOCKED','The erotic-content specification contains adult male-female and female-female pairings; it excludes futanari content.','test.rel.content_taxonomy'),
('req.rel.outside_flirt.recruitment','REL','LOCKED','Romantic or sexual tension between the protagonist and a woman outside the harem always advances her authored recruitment path.','test.rel.outside_flirt_destination'),
('req.rel.member.permanent','REL','LOCKED','Once a heroine joins the harem, household membership is permanent for the shipped campaign and endings.','test.rel.membership_monotonic'),
('req.rel.members.recruit','REL','LOCKED','Existing women may flirt with, seduce and help recruit a compatible woman through authored story scenes.','test.rel.recruitment_scene_flags'),
('req.rel.milestones.authored','REL','LOCKED','Romance progress uses authored milestone IDs and household membership rather than generic relationship meters.','test.rel.milestone_schema'),
('req.rel.rank.story_gated','REL','LOCKED','D through SSS bond ranks unlock from named field, romance, personal-story and household milestones listed in the heroine package.','test.rel.rank_requirements'),
('req.cast.signature_weapon','CAST','LOCKED','Each heroine retains one signature weapon silhouette across equipment progression.','test.cast.weapon_identity'),
('req.cast.identity.package','CAST','LOCKED','Every core character package fixes anatomy, face, costume, handedness, weapon dimensions, prop sockets, palette and performance exclusions before final animation.','test.cast.identity_manifest'),
('req.world.built_balance','WORLD','LOCKED','Two principal cities use the 1700–1750 colonial baseline; most monumental construction is inherited magical Bronze Age elven infrastructure.','test.world.location_class_counts'),
('req.world.dinosaurs.wild','WORLD','LOCKED','Dinosaurs occupy island habitats as wild fauna and are never confined to a themed park, pen or single region.','test.world.ecology_distribution'),
('req.world.midnight.return','WORLD','LOCKED','At island midnight the Return pulse resurrects eligible dead people and repopulates every active monster habitat in visible flashes of light.','test.world.midnight_transaction'),
('req.world.death.memory','WORLD','LOCKED','A returned named character remembers the fact, cause, location and responsible party for each recorded death.','test.world.death_memory_roundtrip'),
('req.world.death.counter','WORLD','LOCKED','Every persistent character record stores total deaths, party-caused deaths and protagonist-caused deaths as monotonic counters.','test.world.death_counters'),
('req.world.return.variance','WORLD','LOCKED','Each return resolves one deterministic condition outcome: Same, Recovered, Changed or Unstable.','test.world.return_condition_table'),
('req.world.monster.daily_level','WORLD','LOCKED','Each midnight generates the next day’s monster populations and levels from habitat, campaign act, island pressure and seeded daily variance; party level is not an input.','test.world.daily_population_seed'),
('req.world.monster.individual','WORLD','LOCKED','Daily monster generation creates plentiful individual threats with their own level, traits, behavior and condition; it does not create disposable one-hit packs.','test.world.spawn_individuality'),
('req.world.monster.level_delta','WORLD','LOCKED','Monster-versus-party level difference materially changes hit pressure, resistance, damage and viable tactics; high-level creatures remain dangerous regardless of encounter count.','test.world.level_delta_curve'),
('req.world.monster.encounter_budget','WORLD','LOCKED','Ordinary encounters field one to three meaningful enemies; four requires a swarm or formation encounter tag and matching action-economy budget.','test.world.encounter_actor_cap'),
('req.world.death.grudge_limit','WORLD','LOCKED','Death caused by the player unlocks acknowledgement but adds no automatic hostility, romance penalty or faction penalty.','test.world.death_no_automatic_grudge'),
('req.world.place_names.honest','WORLD','LOCKED','Place names follow resident naming practice and reject chapter-title constructions that announce a mystery.','test.world.naming_lint'),
('req.estate.upgrade.visible','ESTATE','LOCKED','Every estate upgrade changes a persistent space layer, available action and at least one resident behavior.','test.estate.upgrade_outputs'),
('req.narrative.observation.separate','NARRATIVE','LOCKED','The UI distinguishes direct observation, attributed testimony, inference, system fact and unknown.','test.narrative.knowledge_classes'),
('req.narrative.conditional.bound','NARRATIVE','LOCKED','One prose surface uses a base block plus no more than four independent conditional insert groups without review.','test.narrative.variant_budget'),
('req.pres.card_active.core','PRES','LOCKED','Inactive party members remain readable cards; the selected or acting character expands into one large full-body actor on the battle plane.','test.pres.card_active_capture'),
('req.pres.frame.complete','PRES','LOCKED','Every animation capture contains the full body, signature weapon, required partner or target proxy, full VFX envelope and eight-percent margin.','test.pres.frame_envelope'),
('req.pres.visual_proof.required','PRES','LOCKED','Every visually consequential content record cites an approved coherent reference package before it becomes ART_LOCKED or BUILD_READY.','test.pres.visual_coverage'),
('req.pres.skill.board.five_panel','PRES','LOCKED','Every heroine skill and protagonist Echo has entry, anticipation, contact, consequence and recovery images plus timing and game-size proofs.','test.pres.action_board_schema'),
('req.pres.reference.graph','PRES','LOCKED','Every reference plate names the stable content IDs it proves and the approved master plates whose invariants it inherits.','test.pres.reference_dependency_graph'),
('req.pres.state.sync','PRES','LOCKED','Animation, targeting, damage feedback and committed state remain synchronized at every accessibility speed.','test.pres.speed_sync'),
('req.tech.content.ids','TECH','LOCKED','Runtime references use stable content IDs and never use Godot node paths as content identity.','test.tech.id_reference_scan'),
('req.tech.content.data','TECH','LOCKED','Authored rules and content live in typed resources or data outside presentation scenes.','test.tech.scene_rule_scan'),
('req.tech.save.migrate','TECH','LOCKED','Every shipped schema version has archived fixtures and deterministic forward migrations that preserve the original save on failure.','test.save.archived_migrations'),
('req.qa.package.complete','QA','LOCKED','A content package enters production only when its manifest, dependencies, outputs, validators, fixtures and acceptance evidence are complete.','test.qa.package_manifest')]
add_table(doc,['Requirement ID','Owner','Status','Statement','Primary verification'],core_requirements,[1.65,.55,.55,2.75,1.0])

add_h(doc,'A.12 Section locality map',2)
add_table(doc,['Later section','Local purpose','Normative owner referenced'],[
('0. Product definition','Player promise, product pillars, exclusions, scope risk and shipped budget.','PROD, SCOPE'),
('1. Player and campaign','Control, information, party, time, chapters, failure, difficulty, saves and onboarding.','CAMPAIGN, SIM, EXPEDITION'),
('2. Island','World history, ecology, cultures, regions and naming examples.','WORLD'),
('3. Expedition play','Knowledge, text, routes, sites, verbs, supplies, camps and complete-loop content.','EXPEDITION, NARRATIVE, CAMPAIGN'),
('4–5. Combat and skills','Battle experience, formulas by reference, ability content and animation actions.','COMBAT, PRES, CAST, REL'),
('6–8. Household and cast','Stable harem premise, authored romance milestones, biographies, pair arcs, estate behavior and upgrades.','REL, CAST, ESTATE'),
('9–12. World content','Tombs, sites, intelligence, quests, factions, economy and equipment.','WORLD, CAMPAIGN, EXPEDITION'),
('13–14. Presentation','Art, asset, camera, animation and audio production constraints.','PRES, CAST'),
('15. Prototype','Vertical-slice package contents and acceptance evidence.','SCOPE, QA'),
('16–17. Architecture and tests','Godot projection, data resources, services, saves, validators and fixtures.','TECH, QA, SIM'),
('18. Production','Gate sequence, responsibilities and proof artifacts.','SCOPE, QA'),
('19–20. References and principles','Non-normative visual evidence, limitations and concise rationale.','No new requirements permitted.')],[1.35,3.2,1.95])
add_callout(doc,'Instruction placement audit',"A sentence in sections 19–20 that appears to introduce a new rule fails the documentation audit. Move it to the requirement registry and authoritative domain section, then leave only a stable-ID reference in the closing material.")

add_h(doc,'A.13 Specification compilation procedure',2)
add_numbered(doc,[
"Parse the task packet and select one primary package.",
"Resolve its manifest, schema version, domain owner and dependency closure.",
"Load the referenced requirements and invariants. Reject unresolved, deprecated or cyclic references.",
"Generate or update typed data definitions before scene or presentation work.",
"Generate command validation and deterministic simulation tests before presentation binding.",
"Create the smallest gray-box projection that exposes every command, state transition, blocked reason and event.",
"Build authored content against validated schemas. Reject one-off script fields and unregistered flags.",
"Bind presentation to committed events. Export envelope, timing, focus and accessibility metadata with every asset.",
"Run package validation, dependency tests, seeded scenario tests, save round-trip and migration tests.",
"Run human-readable acceptance at game scale: controls, text, framing, animation, audio and player feedback.",
"Write the build-ledger record, including requirement IDs, versions, evidence and remaining risks.",
"Advance only to the next legal gate in A.8."
])

add_h(doc,'0. What we are doing',1)
add_p(doc,"Project 42: Pirate Island RPG is a single-player, party-based, adult haremlit fantasy RPG about establishing a household in a place that resists being understood. The player is a capable but initially displaced man. He explores a wild tropical island with a party of adult heroines, learns the practical rules of its impossible geography, survives fights that are dangerous enough to make retreat meaningful, and converts discoveries into security, capability, intimacy and influence. The game is not primarily about collecting women, clearing icons, building a colonial empire or solving a single mystery. It is about making a life with specific people in a dangerous world whose history is physically present and politically contested.")
add_p(doc,"The design has three inseparable centers. The first is the expedition: preparation, uncertain travel, close textual observation, tactical combat, discovery and the decision to continue or return. The second is the household: treatment, romance, fanservice, research, craft, recovery, celebration and visible changes to the estate. The third is accumulated understanding: the island becomes more navigable because the player remembers what was seen, compares testimony, develops companions and turns knowledge into new action. If a proposed feature does not strengthen at least one of these centers and create a useful consequence in another, it probably does not belong in the game.")
add_callout(doc,'One-sentence product thesis',"Lead a vividly characterized party of women into a beautiful, unfair island; read the world closely enough to survive it; and build the romantic household that makes survival worth the effort.")

add_h(doc,'0.1 The player’s repeating experience',2)
add_table(doc,['Phase','Player question','Principal systems','What returns to the loop'],[
('Prepare at home','What are we trying to accomplish, whom do I trust with it, and what can we afford to risk?','Party selection, relationship state, injuries, equipment, supplies, research, estate projects.','A plan with visible assumptions and tradeoffs.'),
('Travel','Which route is safest, fastest, politically acceptable or most likely to reveal something valuable?','Overworld graph, weather, dinosaurs, factions, temporal instability, narrated travel.','Costs, encounters, observations and possible diversion.'),
('Explore and read','What happened here, what rule is operating, and which details matter?','Painted site, authored description, inspection verbs, companion interpretation, journal evidence.','Knowledge, access, danger, treasure and competing explanations.'),
('Confront','Can we negotiate, manipulate, bypass, fight or retreat, and what will each answer cost?','Dialogue, faction standing, five-band combat, objectives, hazards, enemy intent, retreat.','Consequences rather than a universal victory state.'),
('Return and change','What did this expedition do to the party, the house and the island?','Treatment, spoils, research, romance, celebration, upgrades, faction reaction, Champion learning.','New capability and a stronger household for the next decision.')],[1.05,1.8,2.1,1.55])
add_p(doc,"This loop is the arbiter of scope. Sailing, crafting, erotic scenes, temporal anomalies, dinosaurs, politics and tomb mechanisms are not separate attractions lined up beside one another. Each must occupy a definite place in the loop. A dinosaur encounter may change the route, consume medicine, reveal a migration sign and become an evening argument about whether the party took a foolish risk. A romantic scene may resolve that argument, unlock a coordinated skill and change who volunteers for the next expedition. The game becomes coherent when consequences cross the boundary between adventure and home.")

add_h(doc,'0.2 Product pillars',2)
add_table(doc,['Pillar','Promise','Failure signal'],[
('Dangerous expeditions','Preparation, knowledge and retreat matter because the world does not scale politely around the player.','Most encounters are solved by repeating the same rotation or walking forward until the objective marker ends.'),
('Text worth reading','Prose provides actionable perception, competing interpretation and emotional texture unavailable in the image alone.','The player can skip every description without losing understanding, choice or advantage.'),
('Women with continuing lives','Every heroine has work, politics, desire, boundaries and relationships that continue outside her romance with the protagonist.','Companions become interchangeable bonuses or vanish from the fiction when their route is not active.'),
('A household that changes play','Romance, trust, conflict and domestic investment alter preparation, capabilities, events and the physical estate.','The home is a menu lobby and affection is a currency spent on scenes.'),
('Spectacular readable combat','Every ability has an obvious physical idea, a decisive hero image and tactical information that survives the spectacle.','Effects obscure bodies, skills feel generic, or animation scale makes the state harder to understand.'),
('Knowledge becomes mastery','The player’s growing model of the island changes routes, tactics, negotiations and interpretations.','Progress is primarily larger numbers and color-coded loot.')],[1.25,2.8,2.45])

add_h(doc,'0.3 Explicit non-goals',2)
add_bullets(doc,[
"This is not an open-world map-cleaning game. The island contains a limited number of dense, remembered places and dangerous spaces between them.",
"This is not a colony-management fantasy. The estate is a household and expedition base, not a mechanism for converting the island into obedient territory.",
"This is not a gacha structure, even aesthetically. Cards are a battle-staging device; heroines are authored party members, not collectible rarity objects.",
"This is not six isolated dating simulations. Romantic progression exists inside a shared household whose members form relationships with one another.",
"This is not a punishing-retro usability exercise. The world may be obscure and unfair; controls, feedback, saving, comparison and journals must be excellent.",
"This is not a parade of disconnected historical cameos. Temporal displacement creates communities, institutions and consequences rather than references for their own sake.",
"This is not a visual spectacle that asks the player to stop reading. Art, prose, rules and performance must reinforce the same event."
])

add_h(doc,'0.4 Foundational decisions, hypotheses and open questions',2)
add_p(doc,"The document previously allowed firm decisions, attractive hypotheses and unresolved questions to sit at the same level. That makes iteration look like inconsistency. From this pass forward, every major claim should have a status. A foundational decision may still be reopened, but only by naming the problem it no longer solves. A working hypothesis exists to be prototyped. An open question receives an owner, a test and a decision date rather than being disguised as atmospheric prose.")
add_table(doc,['Status','Current item','Reason / next proof'],[
('Foundation','Single-player, four-member active party, side-view command combat, card-to-active staging.','This combination is the project’s most distinctive playable presentation and must be proved first.'),
('Foundation','Adult haremlit household with six core heroines who love the protagonist and form friendships, romances and sexual relationships with one another.','This is the narrative form of the game, not optional flavor.'),
('Foundation','Wild island dominated by magical Bronze Age elven infrastructure; only two large colonial cities.','Preserves the adventure wilderness and prevents the colonial baseline from visually swallowing the premise.'),
('Foundation','Textual description and inspection are first-class interaction surfaces.','The game’s knowledge progression requires language to carry information and interpretation.'),
('Working hypothesis','Five positional bands, four active characters and one expanded performer at a time.','Prototype must confirm tactical depth, target clarity and animation throughput.'),
('Working hypothesis','Seven bond-ranked skills per heroine from D through SSS.','Useful production constraint, but rank count and unlock rhythm must survive kit and campaign pacing tests.'),
('Working hypothesis','Eighteen to twenty-four named destinations, of which six are major multi-visit sites; twelve major-site concepts remain a longlist.','First-release budget subject to the vertical slice. A smaller graph with changing routes is preferable to a large map of disposable stops.'),
('Open question','Exact campaign length, chapter structure and number of mandatory expeditions.','Resolve after the complete vertical slice produces reliable playtime and content-cost measurements.'),
('Open question','Explicitness, presentation and distribution of adult intimacy.','Requires rating, storefront and narrative-direction decisions; authored scene packages remain valid at either presentation level.'),
('Open question','Degree of protagonist appearance customization and voice performance.','Test whether customization weakens authored chemistry or improves identification without multiplying content unsustainably.')],[1.2,2.55,2.75])

add_h(doc,'0.5 Pass structure for the design',2)
add_numbered(doc,[
"Pass one — coherence: define the product thesis, repeating loop, campaign promise, pillars, non-goals and system boundaries. This document is now that pass.",
"Pass two — player model: specify protagonist, party selection, controls, difficulty, failure, saving, onboarding, accessibility and the exact information available at every decision.",
"Pass three — expedition model: specify overworld graph, travel time, supply consumption, camps, weather, dinosaurs, random and authored events, retreat and return.",
"Pass four — textual exploration: specify description assembly, inspection verbs, evidence provenance, companion interpretation, journal operations, theories and clue-to-action conversion.",
"Pass five — combat: specify turn order, action economy, bands, targeting, damage, guard, composure, conditions, objectives, AI, retreat, rewards and full heroine kits.",
"Pass six — household and haremlit: specify household membership, heroine-pair arcs, domestic schedules, named romance milestones, recruitment stories and adult-scene packages.",
"Pass seven — progression and economy: specify experience, bond ranks, Echoes, equipment, supplies, crafting, injuries, estate upgrades, money, trade, loot and pacing curves.",
"Pass eight — world simulation: specify factions, settlement state, route control, quest consequences, temporal events, ecological state, Champion observation and adaptation.",
"Pass nine — content architecture: specify campaign chapters, critical path, companion arcs, major sites, enemy families, bosses, events, prose budgets and reusable authoring patterns.",
"Pass ten — production proof: convert the resolved specifications into Godot schemas, validation, debug tools, telemetry, tests, performance budgets, milestone scope and acceptance criteria."
])

add_h(doc,'0.6 Where development time will actually disappear',2)
add_p(doc,"The project is at greatest risk where authored specificity meets persistent simulation. None of the expensive features is individually unreasonable. The danger is multiplication. Six protagonist romances, fifteen heroine pairs, dozens of locations, party-dependent prose, persistent injuries, daily monster individuals, faction consequences, temporal variants and an enemy that learns can create millions of theoretically distinct states. If content is implemented as bespoke scene logic and loosely named boolean flags, the game will become costly to change before it becomes large enough to play.")
add_p(doc,"The response is not to remove everything interesting. It is to decide which variation is systemic, which is authored, which is cosmetic, and where the game deliberately refuses to react. Every major system needs one authoritative owner, a bounded public interface, validation, debug visualization and a declared variation budget. The following items are the project’s time bombs. They should be treated as preproduction requirements rather than as problems for implementation to discover.")

add_table(doc,['Time bomb','Why it explodes late','Decision to make now','Proof before scale'],[
('Campaign-state sprawl','Flags accumulate across quests, romances, sites and temporal variants; nobody knows which combinations are legal.','Use typed domain state and event history. Ban arbitrary global booleans. Every flag has owner, scope, lifecycle and validation rule.','Load generated state matrices; enumerate unreachable and contradictory states; inspect them in a campaign debugger.'),
('Relationship combinatorics','Six protagonist romances plus fifteen heroine pairs can condition dialogue, schedules, skills and endings.','Only milestone bands drive authored branching. Continuous values tune tone and availability but do not produce unique scenes at every value. Cap pair-specific turning scenes.','Simulate every milestone combination used by the slice; prove deterministic scene selection and readable block reasons.'),
('Household scheduling','Residents, rooms, injuries, projects, privacy and conversations produce collisions and missing characters.','Schedules choose from authored activity slots through priorities and reservations. Never script independent free-running resident logic.','Run a thirty-day accelerated estate simulation with no deadlocks, duplicate residents or unavailable critical scenes.'),
('Conditional prose','Descriptions conditioned on party, knowledge, time, weather and prior choices become impossible to author and localize.','Build prose from a stable base passage plus a small number of ordered inserts. Set a hard condition count per passage and expose provenance in tools.','Generate every legal variant for one site, lint grammar and length, and test journal extraction in all supported UI sizes.'),
('Quest consequence graph','Every faction and route reacting to every quest creates content debt and contradictory outcomes.','Record consequences as changes to a small set of world facts. Quests read facts; they do not directly modify unrelated quests.','Visualize the dependency graph and run chapter-boundary validation against all critical-path states.'),
('Temporal location variants','Multiple historical versions multiply art, collision, encounters, persistence and prose.','A site has one canonical topology unless a major-site budget explicitly permits a variant. Minor anomalies swap layers, rules and text rather than entire scenes.','Build one two-version site and measure art, QA, save and authoring cost before approving another.'),
('High-resolution actors','Large sprites, many poses and VFX can exhaust memory, loading time and texture limits.','Define battle display pixels, source resolution, atlas limits, compression, streaming groups and maximum simultaneous actors before final art.','Profile a worst-case boss battle on minimum hardware with final-size Betty, Ayla and maximum VFX placeholders.'),
('Paired and rescue animation','Carrying, embracing, intercepting and pulling require bodies of different sizes to align.','Use a small library of partner sockets and standardized victim proxies. Reserve bespoke paired animation for high-value scenes.','Prove Betty carry, Isabella swap, Nara pull and one intimate household pose across all supported partner body classes.'),
('Ability scripting','Forty-two heroine skills, Echoes, enemies and items become hand-coded exceptions.','Resolve actions through composable commands and effects. Custom code requires an explicit extension point and test.','Implement two radically different heroine kits without adding special cases to the battle runner.'),
('Adaptive Champion','Learning logic can feel unfair, become exploitable or require bespoke responses to every build.','Adapt from a small taxonomy of observed patterns. Every adaptation has evidence, telegraph, counter, cooldown and story legality.','Run seeded combat histories; confirm players can predict and counter selected adaptations.'),
('Save compatibility','Persistent games change throughout development; renamed IDs and altered schemas destroy saves.','Use stable content IDs, schema versions, migrations, atomic writes and debug snapshots from the first slice.','Load saves from every milestone build through automated migration tests.'),
('Controller and focus logic','Card expansion, targets, journal evidence, maps and dense text create many focus modes.','Define one input-action vocabulary and explicit focus ownership. Keyboard, controller and mouse must share state, not separate implementations.','Complete the slice without a mouse and recover correctly from every modal interruption.'),
('Testing state explosion','Manual play cannot cover relationship, world, party, route and Champion combinations.','Separate deterministic simulation from presentation; expose seeds, commands, snapshots and headless scenario tests.','Execute thousands of generated campaigns and battles nightly with invariant checks.'),
('Adult-content branching','Fixed participant combinations, pair histories and storefront variants can multiply scenes and assets.','Author intimacy around named milestones with fixed participant sets and a small approved variant budget. Decide rating variants before final asset production.','Export a content matrix showing which scenes exist, their participants, their variants and the exact milestone flags they change.')],[1.15,1.65,1.95,1.75])

add_h(doc,'0.7 Anti-bankruptcy constraints',2)
add_p(doc,"These constraints are not aesthetic preferences. They are the rules that allow the project to retain its richness without producing an untestable bespoke game for every player state.")
add_bullets(doc,[
"One authoritative state owner per domain: battle, campaign, relationships, estate, world map, site persistence and adaptive opposition. Presentation scenes never become the source of truth.",
"No feature may branch on an unregistered string flag. Conditions reference typed facts, milestones or tags that validation tools can enumerate.",
"No authored passage may depend on more than four independent condition groups without a design review. Extra nuance should normally become a companion insert or follow-up interaction.",
"No ordinary quest creates a unique permanent version of a location. It changes world facts, actors, available interactions or lightweight layers within a controlled persistence schema.",
"No skill enters final animation until gray-box combat proves its decision value, target rule, duration, repetition tolerance and required body/prop sockets.",
"No paired animation assumes arbitrary body proportions. Every interaction declares supported partner classes and a fallback staging.",
"No content ID is renamed or reused after it enters a saved build. Human-readable display names may change; stable IDs do not.",
"No systemic reaction is promised without an authoring path, a debugging view and an automated way to verify at least its invariants.",
"No new major site, heroine, romance dimension or historical variant is approved using optimistic per-item estimates. Its cost includes integration, conditional writing, UI, saves, localization, testing and future maintenance.",
"No amount of sunk art or prose protects a feature that fails the complete loop. Content can be cut; architectural debt compounds."
])

add_h(doc,'0.8 Tools required before content production',2)
add_table(doc,['Tool','Minimum capability','Why it precedes scale'],[
('Content registry and validator','Browse every stable ID and dependency; detect missing references, duplicates, illegal ranks and unused content.','Prevents data errors from becoming runtime mysteries.'),
('Campaign state inspector','Edit typed state, view event history, compare snapshots, explain why content is available or blocked.','Makes romance, quest and world bugs reproducible.'),
('Relationship milestone debugger','Display household membership, protagonist romance milestone, heroine-pair milestones, scene flags and eligible scenes.','Prevents invisible authored conditions from consuming narrative QA.'),
('Site variant preview','Render a location under chosen time, knowledge, faction, hazard and completion states.','Allows artists, writers and designers to review persistence without replaying the campaign.'),
('Prose variant generator','Enumerate base text and inserts with their conditions, word counts, provenance and journal effects.','Controls grammatical, localization and combinatorial failure.'),
('Battle scenario runner','Choose actors, skills, bands, enemy intents, seed and objective; step or simulate; export event log.','Makes ability design and AI testable before animation.'),
('Animation envelope viewer','Overlay body, weapon, partner and VFX bounds across the timeline with camera safe areas.','Catches cropping and scale problems before expensive final rendering.'),
('Save migration harness','Load archived fixtures from previous schema versions and report every transformation.','Protects long campaigns and accelerates iteration.'),
('World dependency graph','Show which facts, quests, routes, sites, factions and chapters read or write one another.','Reveals accidental coupling and impossible critical-path states.'),
('Automated campaign simulator','Advance days, schedules, travel, encounters and consequences with deterministic seeds.','Finds deadlocks and state combinations no manual team can cover.')],[1.45,2.65,2.4])

add_h(doc,'0.9 Kill criteria we should agree to now',2)
add_p(doc,"A prototype is valuable partly because it gives the team permission to stop pursuing an expensive idea that does not produce the promised experience. These criteria prevent an attractive feature from surviving indefinitely on the assumption that one more implementation pass will reveal its value.")
add_bullets(doc,[
"Replace card-to-active staging if players cannot reliably read turn ownership and targets, or if transition time makes ordinary actions tedious after acceleration and skip options are applied.",
"Reduce or restructure the five-band model if most abilities ignore position or if optimal play collapses into permanent front and rear assignments.",
"Reduce seven-skill kits if late-rank skills cannot remain tactically distinct, readable and affordable to animate without stealing roles from other heroines.",
"Constrain textual variation if players cannot identify actionable information, writers cannot preview every legal output, or localization cost grows faster than content value.",
"Simplify companion-pair simulation if ambient variation is not noticed or if pair state repeatedly blocks critical household scenes in ways players cannot understand.",
"Remove adaptive responses that players attribute to cheating rather than learning after evidence and telegraphs are present.",
"Collapse temporal variants into rule and text changes when a full scene variant costs more than a new location while producing less memorable play.",
"Cut any estate subsystem that does not change preparation, relationships, expedition capability or the felt life of the household."
])

add_h(doc,'0.10 Review ledger for this pass',2)
add_p(doc,"This bible is not allowed to grow only by accretion. Every pass must re-read earlier promises and either reaffirm, revise or demote them. The following corrections were made while specifying expedition play; they supersede looser language elsewhere if any survives in notes or exploratory art.")
add_bullets(doc,[
"The repeating experience is now the six-step loop in section 1.8. The earlier four-part phrasing remains only as a broad emotional summary and no longer defines implementation boundaries.",
"The first-release world target is eighteen to twenty-four named destinations with six major multi-visit sites. The twelve named major sites in section 9.2 are a creative longlist, not twelve promised production dungeons.",
"The field-party hypothesis is explicitly the protagonist plus three heroines. The protagonist therefore requires a complete card, full-body battle actor, animation envelope and readable combat kit; he cannot remain an invisible commander while the document claims four active performers.",
"The ExpeditionState is now an authoritative temporary state owner rather than a loose collection of campaign flags. Route, camp, exploration and combat consequences must pass through its event ledger before becoming campaign aftermath.",
"The save matrix in section 1.11 is the authoritative save contract. Shorter lists later in the document are implementation examples and may not silently narrow it.",
"The prototype tomb is twelve authored spaces, not twelve combat rooms. Its purpose is to prove cross-system persistence, evidence, routes, camp, tactical encounters and aftermath in one complete loop.",
"Reference images remain exploratory plates. They cannot overrule the written party size, full-body framing, wild-island settlement ratio, character identity packages or state contracts."
])
add_callout(doc,'Next review pressure',"The next pass must challenge combat rather than simply add more skills: action economy, initiative, band occupancy, enemy objectives, status taxonomy, resource curves, encounter duration, protagonist kit and whether seven skills per heroine remain mechanically distinct after repeated play.")

add_h(doc,'0.11 Authority of the executable contract',2)
add_p(doc,"Sections 0.11 through 0.18 are the implementation baseline. Their values enter data, tests and production estimates. Descriptive passages elsewhere explain intent and atmosphere; they cannot override these rules. A change requires a dated decision record containing the old value, new value, evidence, affected content, migration requirement and approving discipline. The team builds the stated default while a test is pending. Prototype uncertainty never leaves a programmer to invent the rule inside a scene script.")
add_bullets(doc,[
"MUST identifies a validator, save, accessibility or content-integrity requirement. A build fails when it is violated.",
"DOES states shipped baseline behavior. Implementation follows it until a recorded change replaces it.",
"TARGET states a measured balance or performance value with an allowed range and a named test.",
"CANDIDATE identifies content that has no production allocation. Candidate material does not create dependencies in shipping systems.",
"Every system lists one state owner, accepted commands, emitted events, save representation, debug view and automated test surface.",
"Every authored exception uses the same command and event interfaces as ordinary content. No quest directly manipulates scene nodes or another system's private state."
])

add_h(doc,'0.12 Shipped campaign budget',2)
add_table(doc,['Item','Locked baseline','Production interpretation'],[
('Campaign length','24-hour median first completion; accepted range 20–30 hours.','External playtest median controls cuts. Optional completion may reach 38 hours.'),
('Campaign structure','Prologue, four acts and finale.','Six transition councils and six protected chapter saves.'),
('Playable cast','One protagonist and six heroines.','Seven complete cards, battle actors, exploration portraits and relationship identities.'),
('Field party','Protagonist plus three selected heroines.','Four active cards. No ordinary fifth guest combatant.'),
('World destinations','18 named destinations.','Estate, two cities, six major sites and nine minor/service destinations.'),
('Major sites','Six multi-visit sites, each containing 10–14 authored spaces.','One production site per act, two distributed across optional/finale structure.'),
('Route graph','26 directional route edges.','Each direction stores its own time, hazards and event budget.'),
('Enemy library','Eight families; three standard roles and one elite per family.','32 reusable enemy definitions before bosses and summons.'),
('Bosses','Seven authored bosses.','One prototype burial lord, four act bosses, Champion and finale form.'),
('Heroine rank gifts','Seven per heroine; 42 total.','Exactly four combat commands, one reaction/passive, one expedition capability and one capstone per heroine.'),
('Romance spine','Six heroine romances with six mandatory turns each.','Thirty-six core turns: attraction, courtship, first intimacy, heroic admiration, move-in and SSS capstone.'),
('Adult romance','Two male-female adult scenes per core heroine in the adult build.','Twelve authored scenes. Storefront variants alter presentation, never romance facts.'),
('Heroine pairs','Fifteen pair definitions with one defining scene each; six pairs receive romantic or sexual follow-through.','Twenty-one authored pair scenes plus reusable estate and camp behaviors.'),
('Endings','Four world resolutions with household epilogues assembled from committed relationship and faction facts.','No combinatorial bespoke ending for every flag combination.')],[1.45,1.95,3.1])
add_callout(doc,'Scope enforcement',"The content registry reports actual counts against this table. Adding one item requires removing an item of equal production class or approving a budget change that includes writing, art, animation, audio, localization, testing, saves and future maintenance.")

add_h(doc,'0.13 Runtime mode state machine',2)
add_table(doc,['Mode owner','Accepted entry','Legal exits','Authoritative state'],[
('EstateMode','New day, expedition return, chapter transition, local scene return.','WorldMapMode, ConversationMode, ProjectMode, SaveMenu.','CampaignState plus HouseholdState.'),
('WorldMapMode','Departure command from estate or safe anchor.','TravelMode, EstateMode, SaveMenu.','CampaignState plus new or restored ExpeditionState.'),
('TravelMode','Committed RouteCommand.','ExplorationMode, CampMode, WorldMapMode at safe node, BattleMode.','ExpeditionState.'),
('ExplorationMode','SiteEntryCommand or return from local battle/conversation.','BattleMode, ConversationMode, CampMode, TravelMode.','ExpeditionState plus SiteState.'),
('BattleMode','EncounterStartEvent with complete participant snapshot.','Origin mode through BattleResultEvent.','BattleState; result projects into ExpeditionState atomically.'),
('ConversationMode','ConversationStartEvent with participants and context.','Origin mode through ConversationResultEvent.','ConversationState; result projects into relationship and campaign state atomically.'),
('CampMode','CampCommand at legal anchor.','TravelMode, ExplorationMode or BattleMode.','ExpeditionState plus CampState.'),
('ProjectMode','Estate project selection.','EstateMode.','HouseholdState; completion resolves only at declared time boundary.')],[1.45,1.75,1.7,1.6])
add_p(doc,"One mode owns input. Overlays such as journal, inventory, help and settings pause command acceptance without changing mode. Every transition follows Request, Validate, Commit, Present, Acknowledge. Validation returns a stable blocked-reason ID. Commit writes one transaction. Presentation consumes emitted events. Acknowledge establishes the next safe input state and save boundary.")

add_h(doc,'0.14 Combat rules',2)
add_p(doc,"Combat is a visible round-based command system. A round contains one activation for every conscious combatant. At round start, initiative equals Speed plus a seeded six-sided roll. The queue is displayed before the first activation. Ties resolve by higher Precision, then party before enemy, then stable actor ID. Initiative remains fixed for that round. A Stagger event moves the target behind the next unresolved actor; it never deletes an activation unless the effect explicitly applies Stunned.")
add_table(doc,['Activation rule','Exact behavior'],[
('Command','Choose one: Basic Attack, Guard, Reposition, Item, Signature Command or Context Command.'),
('Movement','Reposition moves one band. A skill moves only the distances stated in its CommandSteps.'),
('Reaction','Each actor has one reaction token per round. Triggered reactions state their trigger, priority and target before commitment.'),
('Target commitment','Targets lock when the command is confirmed. Invalid targets after an interrupt cause the declared fallback; they never produce an improvised retarget.'),
('Hit test','Chance = clamp(50, 95, 75 + 5 × (Precision − Evasion)). The preview displays the exact percentage.'),
('Damage','Damage = max(1, Potency + Power + d6 − Armor − GuardSpent). Preview displays the seeded range before commitment.'),
('Guard','Guard is a visible pool. Incoming physical damage consumes Guard before Vitality. Guard resets to zero at battle end.'),
('Composure','Composure ranges 0–10. Costs and hostile effects reduce it. At 0, the actor gains Shaken and cannot use SS, SSS or Echo commands.'),
('Downed','Vitality 0 applies Downed. A downed actor loses activations, remains targetable and receives one Injury check when battle ends.'),
('Victory','Resolve the encounter objective. Killing every enemy is only one objective type.'),
('Retreat','Use the encounter’s declared RetreatCommand. Immediate retreat costs one round; contested retreat requires the stated exit, escort or hold condition.')],[1.45,5.05])
add_p(doc,"The five bands are Party Rear, Party Front, Contested, Enemy Front and Enemy Rear. An actor has one band index. Small actors do not block friendly occupancy. Large actors occupy two adjacent bands and declare both indices. Melee commands reach the actor's band or one adjacent band unless their data states otherwise. Firearms reach any visible band and suffer Engaged when an enemy occupies the same or adjacent contested band. Area commands list exact affected indices. Presentation can move through depth for choreography; the simulation uses band indices only.")
add_table(doc,['Status family','Shipping statuses'],[
('Body','Bleeding, Burning, Poisoned, Injured.'),
('Control','Staggered, Stunned, Pinned, Engaged.'),
('Mind','Shaken, Terrified, Compelled, Alien Marked.'),
('Defense','Guarded, Exposed, Warded, Concealed.'),
('Tempo','Marked, Prepared, Pursued, Delayed.')],[1.35,5.15])
add_p(doc,"Every status has source ID, duration unit, stack rule, cleanse tags, icon, color-independent shape, log text and deterministic expiration event. The baseline permits one stack unless the definition states a numeric maximum. Ordinary battles target three to six rounds and eight to twelve minutes. Boss battles target six to ten rounds and fifteen to twenty-five minutes. Automated balance reports fail encounters outside those ranges after three seeded runs unless the encounter carries an approved duration exception.")

add_h(doc,'0.15 Character progression and rank gifts',2)
add_p(doc,"Combat level and bond rank are separate. Every playable character has Combat Level 1–12. Experience is awarded to the entire recruited cast after an expedition objective, major discovery or boss result; individual kill experience does not exist. Required cumulative experience is 0, 100, 250, 450, 700, 1,000, 1,350, 1,750, 2,200, 2,700, 3,250 and 3,850. Each level adds the CharacterDef vitality growth. Levels 2, 4, 6, 8, 10 and 12 grant one choice from that character's twelve-entry mastery library. Monster levels are generated for the whole island at midnight from habitat tier, campaign act, island pressure and seeded daily variance. Party level is excluded from that calculation, so the player can encounter a population that is currently too dangerous.")
add_table(doc,['Bond rank','Gift class','Availability baseline'],[
('D','Core combat command.','Available when recruited; teaches the signature weapon.'),
('C','Second combat command.','Unlocks after the heroine’s named first field-success scene.'),
('B','Hybrid combat or rescue command.','Unlocks after personal-story milestone one and its named follow-up scene.'),
('A','Signature combat command.','Unlocks after the heroine’s signature proof encounter.'),
('S','Reaction or passive rule.','Unlocks after the heroine’s named romance milestone and personal-story milestone two.'),
('SS','Expedition or household capability.','Unlocks after the heroine changes the estate through her named household milestone.'),
('SSS','Once-per-expedition capstone.','Unlocks after personal-story resolution and her named permanent-household scene.')],[.8,1.8,3.9])
add_p(doc,"The seven named gifts remain character-specific. The command bar displays only combat commands and eligible capstones. Expedition capabilities appear at route, site, camp or estate decision surfaces. Passives appear on the card inspection panel and in previews they modify. This removes the false claim that every bond gift is an ordinary attack animation.")
add_p(doc,"The protagonist has five permanent commands: Weapon Attack, Guard, Reposition, Field Order and Echo. He equips three of six unlocked Echo definitions before departure. Echo use costs Composure, writes an ObservationEvent for the Champion and cannot copy a heroine command. His Combat Level follows the same 1–12 table. Background selects one exploration verb bonus and one starting mastery; it never changes his authored voice or core command set.")

add_h(doc,'0.16 Household membership and romance progression',2)
add_p(doc,"The harem is an informal household and a stable genre premise. People join because they fall in love, want the protagonist, want the women already there and choose that life. The runtime records membership and authored milestones. It does not model household law or relationship failure. Daily scenes show affection, sex, flirtation, work, jokes, admiration, comfort and women enthusiastically helping the household grow.")
add_p(doc,"Heroines direct romantic and sexual attention toward the protagonist, women already in the harem and compatible women they are helping bring into it. They do not pursue men outside the harem or privately wish for another male partner. The heroines flirt with one another, sleep with one another, fall in love in different combinations, and sometimes take the lead in seducing a prospective member. The household remains stable while these relationships grow.")
add_table(doc,['Stored field','Type','Purpose'],[
('household_member','bool','Whether the character currently lives in and belongs to the harem household.'),
('protagonist_romance_milestone','MilestoneID','Current authored protagonist–heroine romance turn.'),
('pair_arc_states','map<PairID, MilestoneID>','Current authored heroine–heroine friendship, flirtation or romance turn for relevant pairs.'),
('bond_rank','D–SSS','Current capability rank; its heroine package names the exact milestone that unlocks each rank.'),
('scene_flags','set<FlagID>','Specific remembered events needed by later scenes; no generic affection arithmetic.'),
('recruitment_focus_id','CharacterID?','Optional woman whom one or more household members are currently courting into the harem.')],[1.7,1.45,3.35])
add_p(doc,"Recruitment is written as a character story. A household member notices or already knows a compatible woman, flirts with her, brings her into the group’s life and may seduce her before she moves in. The protagonist may lead, follow or join the courtship according to the characters involved. The decisive scene sets household_member to true. There is no mandatory five-stage protocol and no unanimous approval mechanic.")
add_p(doc,"Adult scenes are authored scenes with fixed participants and variants written for those participants. Eligibility reads named story milestones, location and presence. The scene itself depicts mutual desire through dialogue, behavior and performance. The runtime does not calculate desire, assemble permitted acts, store a consent score or award relationship points for watching the scene.")

add_h(doc,'0.17 Economy, load and recovery values',2)
add_table(doc,['System','Locked values'],[
('Currency','Silver is the UI accounting unit. Local coins and credit convert at merchant-defined rates; the player inventory stores silver value plus named obligations.'),
('Personal equipment','One signature weapon, one armor garment, two accessories and one prepared consumable per actor.'),
('Expedition load','Base 12 load units; +4 pack animal; +8 carriage; +10 canoe; +20 owned vessel. Injured carriers reduce walking load by 2 each.'),
('Food','One party-day costs one load unit and feeds four active members for one day.'),
('Medicine','One medical kit costs one load unit and contains three treatment charges.'),
('Ammunition','One ammunition bundle costs one load unit and supplies six firearm commands.'),
('Light','One light bundle costs one load unit and supplies two four-hour exploration intervals.'),
('Camp kit','Two load units; upgrades an exposed legal anchor by one quality step, never above Secure.'),
('Salvage','Every recoverable object has 1, 2, 4 or 8 load cost. Reserved capacity is visible before departure.'),
('Injury recovery','Minor: one day; Serious: three days plus treatment; Severe: seven days, treatment and authored limitation.'),
('Defeat floor','The estate always provides one basic meal and one basic treatment per day. Currency depletion cannot deadlock the campaign.')],[1.45,5.05])

add_h(doc,'0.18 Required screens and internal tools',2)
add_table(doc,['Surface','Must ship'],[
('Estate HUD','Time slot, current residents, injuries, active projects, visitors, tracked lead and departure action.'),
('Party/load screen','Four active slots, reserve activities, exact load, equipment conflicts, objections, route contributions and blocked reasons.'),
('World map','Known/reported/inferred nodes, directional routes, duration, supply, transport, weather, camps, uncertainty source and commitment preview.'),
('Exploration HUD','Current space, interaction focus, party condition, light, time, retreat direction and prose/history access.'),
('Battle HUD','Initiative queue, four party cards, enemy intents, five-band diagram, acting command bar, exact preview and event log.'),
('Camp screen','Anchor quality, two watches, assignments, recovery preview, privacy, detection and departure state.'),
('Journal','Observations, testimony, inferences, contradictions, provenance, confidence, map links and actionable comparisons.'),
('Relationship ledger','Household membership, protagonist romance milestone, heroine-pair milestones, promises, scene flags and recent authored changes.'),
('Campaign inspector','All authoritative states, event history, command replay, blocked-reason explanation and snapshot comparison.'),
('Content validator','Counts, IDs, dependencies, unreachable content, illegal transitions, missing localization, missing assets and budget variance.'),
('Scenario runner','Seed, party, encounter, route, site state, relationship state, command stepping and golden-log export.'),
('Variant preview','Character identity package, prose combinations, site layers, card states, animation bounds and storefront content level.')],[1.55,4.95])

add_h(doc,'1. The game we are actually making',1)
add_p(doc,"This is a modern, painted 2D party RPG set on a tropical island that does not belong to one century. Its two principal colonial cities and a scattering of forts, plantations, free ports and mission settlements resemble the world of roughly 1700 to 1750: flintlocks are dangerous and slow, sailing ships carry trade and conquest, and piracy is less a costume than an economy conducted at cannon range. The island as a whole does not look colonial. Its dominant built inheritance is the monumental infrastructure of a high-magical elven Bronze Age: causeways crossing valleys, verdigris aqueducts, tomb terraces, ritual harbors, observatories, colossal retaining walls and civic machines that continue operating under roots and rain. Between them lies enormous, exuberant wilderness. Dinosaurs cross all of it as wild animals rather than as curiosities assigned to one valley.")
add_p(doc,"The island does not behave like a cheerful multiverse in which every historical reference is an invitation to wink at the player. People live here. They establish law, repeat old injustices, intermarry, trade, invent customs, and misremember the worlds from which their ancestors arrived. Fox-folk maritime houses with East Asian institutional and visual roots occupy compact ports, island compounds and ships rather than a huge themed metropolis; their society is expressed through contracts, clan obligations, shipyards, shrines, debt, art and rival ideas about citizenship. Orcs possess one of the two major colonial cities and maintain powder regiments, academies, officers and factional politics, but most orcs the party meets beyond it are travelers, settlers, soldiers, hunters or members of mixed frontier communities. Jungle elves are neither timeless woodland innocents nor direct survivors of the ancient empire. They are descendants who live among a built world far larger than any present society, inheriting dangerous infrastructure, partial rites, and the burden of outsiders treating their ancestors' tombs as treasure vaults.")
add_p(doc,"The player arrives without mastery of any of this. He survives first, acquires explanations second, and gradually learns that explanations are themselves contested resources. The fantasy is not to become the chosen king of a conveniently vacant island. It is to become competent enough that the island can no longer dispose of him casually, trusted enough that people share what they know, and rooted enough that an abandoned coastal estate becomes a household whose future is worth defending.")

add_h(doc,'1.1 The protagonist we need',2)
add_p(doc,"Haremlit works poorly around a protagonist who is so blank that every woman appears to be talking past an empty space. The player character should therefore be partially authored. The player chooses his name, a restrained appearance package and one prior background, but he retains a consistent baseline: observant, capable of humor, willing to act, capable of sexual and emotional desire, and increasingly unwilling to confuse domination with leadership. He can make cowardly, selfish, dishonest or reckless choices, yet the game does not pretend those choices are charisma simply because he is the viewpoint character.")
add_p(doc,"His initial background changes practical affordances rather than replacing his personality. A sailor reads weather and ships. A soldier understands formations and authority. A physician supports Betty without superseding her. A scholar accelerates comparison and translation. A hunter understands tracks and animal pressure. An occultist recognizes when an explanation is technically correct and still dangerous. Backgrounds create alternative openings inside systems; they do not require six separate protagonists or six campaigns of bespoke dialogue.")
add_h(doc,'1.2 Leadership without ownership',2)
add_p(doc,"The protagonist becomes party leader because capable women watch him make hard decisions, win dangerous fights and bring people home. Party selection, expedition objectives and tactical commands are player responsibilities. A heroine supplies missing facts or warns him when a plan contradicts the mission, then helps execute the decision he makes. The game tests leadership through incomplete information, scarce resources and dangerous opposition. It does not turn every command into a domestic argument. He earns admiration by succeeding without cruelty, keeping his word and treating formidable women as trusted partners.")
add_callout(doc,'Protagonist working hypothesis',"Use a customized name and restrained visual options, one of six backgrounds, an authored conversational voice and no fully voiced protagonist during the prototype. Revisit voice and customization only after testing whether chemistry, player identification and production cost remain in balance.")

add_h(doc,'1.3 The player control contract',2)
add_p(doc,"The player needs a stable answer to the question of what kind of game he is controlling. Project 42 is not an action platformer disguised by turn-based menus, and it is not a visual novel interrupted by detached battles. The player directs one expedition leader through a connected campaign: moving and inspecting in side-view locations, choosing what the protagonist says and attempts, selecting the field party and its supplies, and issuing tactical commands when danger becomes a formal encounter. The game owns bodily execution, camera staging, companion autonomy, uncertain information and the consequences of resolved actions. This division permits theatrical animation without asking the player for fighting-game inputs, and it permits authored companions without turning the protagonist into a spectator.")
add_table(doc,['Context','Player controls','The game resolves','Required feedback'],[
('Estate','Time-slot activity, conversation target, projects, equipment, party and departure plan.','Schedules, private companion activity, household reactions and delayed consequences.','Availability, cost, forecasted time, blocked reason and likely downstream category.'),
('Overworld','Destination, route, transport, optional stops and retreat destination.','Weather, route events, patrols, ecological pressure and temporal disturbances.','Known facts, inferred risks, uncertainty, travel cost and source of each claim.'),
('Exploration','Leader movement, inspect/interact target, tool use, dialogue choices and formation posture.','Companion following, ambient behavior, hidden checks, site persistence and encounter transition.','Interaction affordance, party observation, risk warning and newly learned fact.'),
('Combat','Acting character, skill, legal target, consumable, formation move and retreat attempt.','Animation, hit resolution, enemy action, reactions, status ticks and morale.','Turn owner, intent, target preview, cost, predicted consequence and event log.'),
('Conversation','Protagonist response, question, promise, disclosure, flirtation, refusal and exit.','Companion interpretation, boundaries, memory, interruptions and later availability.','Tone when necessary, known stakes, promises being made and relationship cause after resolution.'),
('Journal and map','Filtering, comparison, pinning, annotation and selection of a tracked lead.','Organization of discovered evidence and explanation of current conclusions.','Source, confidence, contradictions, recency and the action each fact can support.')],[1.0,1.75,1.9,2.1])
add_p(doc,"Input contexts must be mutually exclusive and inspectable. When a dialogue choice is open, exploration movement does not also fire. When target selection is active, the same directional input moves focus among targets rather than cards. Every context declares who owns focus, what cancels one level, what confirms, and where focus returns after a modal interruption. Keyboard, controller and mouse operate the same state machine; they are not three separately improvised interfaces.")

add_h(doc,'1.4 The information contract',2)
add_p(doc,"Text is a first-class game feature only if the player can distinguish observation from interpretation and interpretation from fact. The interface itself never lies about rules, costs, legal targets, saved state or the reason an action is blocked. People, books, maps and memories may be wrong, biased, incomplete or deliberately deceptive, but the journal identifies the source and confidence of a claim. If the player makes a bad decision because a merchant lied, that is drama. If he makes it because the interface silently changed a rule, that is merely distrust of the game.")
add_table(doc,['Knowledge class','Presentation','Permitted uncertainty','Design use'],[
('System fact','Plain rule text, exact cost and explicit status.','None after the relevant rule has been introduced.','Lets the player plan without reverse-engineering the interface.'),
('Direct observation','Attributed field note, date and location.','The observation may be incomplete; what was visibly seen is not rewritten.','Builds reliable evidence through attention.'),
('Testimony','Speaker, relationship and circumstance attached.','The speaker may be mistaken, evasive or lying.','Makes social knowledge useful without making the UI dishonest.'),
('Inference','Journal conclusion with confidence and supporting evidence.','May strengthen, weaken or split into competing explanations.','Turns reading and comparison into play.'),
('Unknown','Question stated clearly enough to investigate.','Answer, mechanism and consequence can remain hidden.','Creates mystery without withholding the existence of an objective.')],[1.15,1.8,1.75,2.05])
add_p(doc,"Enemy intents follow the same rule. The interface may show a category such as heavy attack, movement, control, defense or unknown ritual, with confidence based on prior observation and relevant expertise. It does not need to reveal exact damage before the party understands an enemy, but it must communicate what the character is visibly preparing. A concealed action is a designed enemy capability with evidence and counters, not an arbitrary exemption from readability.")

add_h(doc,'1.5 Field party, reserves and companion agency',2)
add_callout(doc,'Field-party working hypothesis',"The active field party contains the protagonist and three heroines. All four appear as cards when inactive and expand into the battle plane when acting. Other recruited heroines remain at the estate, pursue declared household activities, recover, research, travel on authored business or become temporarily unavailable. Prototype this exact size before producing final battle layouts or party-wide scenes.")
add_p(doc,"A three-heroine choice makes composition meaningful because the player cannot carry every answer. It also limits the number of actors that exploration blocking, conversation interjections, combat staging and save restoration must support at once. The selection screen therefore does more than display statistics. It explains each companion's route contribution, current injuries and fatigue, personal interest or objection, relevant promises, equipment conflicts and the estate role left uncovered by taking her. The player can still choose an imperfect party, but the consequence is legible before departure.")
add_p(doc,"There is no unrestricted reserve bench teleportation. The party changes at the estate, at a properly supplied field camp, aboard an owned vessel when a route plausibly returns to it, or during an authored reunion. A heroine may be unavailable while completing an assigned estate job, commanding her own faction duty or recovering from injury. The party screen always states the immediate operational reason and the exact condition that makes her available again. Ordinary tactical disagreement never creates a surprise lockout or damages romance progression.")

add_h(doc,'1.6 The campaign clock',2)
add_p(doc,"The campaign uses authored time rather than a continuously ticking survival clock. At the estate, a normal day has morning, afternoon and evening activity slots, followed by a night resolution. Conversations, treatment, research, construction decisions, trade and local errands declare which slots they consume. An expedition replaces this local schedule with route and site time: crossing a river, forcing a sealed door, making camp, waiting for a tide or treating an injury advances the clock by a stated amount. The player may read, compare records and arrange equipment without being punished for the real-world minutes spent in menus.")
add_table(doc,['Time unit','What advances it','What resolves','Player protection'],[
('Estate activity slot','Beginning a marked activity or departing during that period.','Schedules, projects, treatment, visitors and local opportunities.','Preview the slot cost and any deadline it will cross.'),
('Travel segment','Committing to a route leg, detour, rest or retreat.','Weather exposure, supplies, route events, ecology and patrol movement.','Show expected duration as a range when uncertain and explain its source.'),
('Site action','A consequential interaction such as excavation, ritual, forced entry or extended search.','Hazards, light, noise, local state and external arrival windows.','Routine inspection and ordinary reading consume no campaign time unless explicitly marked.'),
('Camp watch','Resting, treating injuries, crafting, talking or researching overnight.','Recovery, ambush risk, companion scenes and changes in weather.','The camp screen previews risk and permits guard assignments.'),
('Chapter transition','Completing a spine objective and choosing to proceed.','Large faction moves, new arrivals and estate/world transformations.','The game warns about expiring opportunities and permits postponement until the player confirms.')],[1.2,1.9,1.9,2.0])
add_p(doc,"Deadlines are rare, local and explicit. The island should feel urgent because situations evolve, not because every conversation hides a timer. Most opportunities wait, transform visibly or recur in another form. A critical-path deadline can change who controls a site or what rescue remains possible, but it cannot silently end the campaign. The journal gives both calendar language and mechanical language: ‘the convoy sails after the next storm’ is accompanied by the currently estimated number of relevant time advances.")

add_h(doc,'1.7 Campaign shape and chapter gates',2)
add_p(doc,"The first campaign should be a dense twenty-to-thirty-hour completion rather than an imaginary hundred-hour epic whose middle will never be produced. Its mandatory spine establishes the estate, brings the six heroines into the household, opens the tomb network, reveals the Champion's pattern of adaptation and forces a final decision about the intelligence beneath the island. Optional expeditions deepen factions, romances, companion pairs, routes, equipment and the meaning of the finale. They are not disposable side content: each belongs to at least one enduring system, but the player is not required to exhaust all of them to see an ending.")
add_table(doc,['Campaign movement','Dramatic job','System introduced or stressed','Gate to proceed'],[
('Prologue — The Black Beach','Survive arrival, meet Betty, reach the abandoned estate and learn that close reading changes survival.','Exploration, prose evidence, one combat, treatment, first household choice.','Estate secured and one reliable route established.'),
('Act I — A House on the Wild Coast','Recruit the first functioning expedition circle and make the estate habitable while nearby powers assess it.','Daily slots, party selection, supplies, local factions, first bond ranks.','Household can support a two-night expedition and possesses three independent leads.'),
('Act II — Claims Upon the Island','Enter human, orc, fox-folk and elven disputes whose histories cannot all be reconciled.','Faction standing, companion objections, open routes, political consequences, deeper tomb access.','Two tomb-network keys understood and a chosen alliance or viable neutrality plan.'),
('Act III — The Tomb Roads','Follow ancient burial infrastructure across habitats and discover that the Champion is learning the party.','Persistent sites, temporal variants, counter-adaptation, high-rank skills, vessel access.','Champion pattern demonstrated, network destination located and household prepared for retaliation.'),
('Act IV — What the House Has Become','Defend the household as accumulated promises, rivalries and faction choices arrive together.','Estate crisis, companion-pair state, romance commitments, reserve activity and world convergence.','Crisis resolved without an invalid household state; final expedition plan accepted.'),
('Finale — The End of Yesterday','Enter the deepest infrastructure, confront the adaptive Champion and decide what relationship the living island will have with the alien intelligence.','Complete combat/exploration/prose synthesis and ending-state projection.','A mechanically valid resolution based on accumulated knowledge, capability and chosen obligations.')],[1.35,2.0,2.05,1.85])
add_p(doc,"Chapter gates test readiness facts rather than quest-count totals. The game checks that the household can support the next expedition, that the player possesses a coherent lead, and that required people are alive and reachable; it does not demand that five unrelated errands be cleared because a progress bar says four of five. Before a transition, an explicit council scene summarizes what will change, which known opportunities may expire and which uncertainties remain. Proceeding is a player decision, not an invisible threshold crossed by looting the wrong chest.")

add_h(doc,'1.8 The repeatable session loop',2)
add_numbered(doc,[
"Orient: resume at a clear state summary showing location, current period, party condition, tracked lead, recent consequences and any decision awaiting resolution.",
"Prepare: speak to companions, treat injuries, compare testimony, equip the field party, buy or craft limited supplies and choose a route whose risks are described honestly at the party's current level of knowledge.",
"Commit: travel through one or more route segments, respond to events and enter a site with a declared objective and a plausible retreat path.",
"Press or withdraw: spend health, morale, light, ammunition, tools, time and social capital while new information changes the value of continuing. Retreat is a normal strategic choice rather than a shame screen.",
"Resolve: return, make camp or reach another safe anchor; distribute treatment and spoils, record knowledge, resolve immediate relationship responses and save the new campaign state.",
"Project forward: expose at least two meaningful next intentions so the player ends a session with curiosity rather than administrative confusion."
])
add_p(doc,"A useful play session is twenty to forty-five minutes, but the structure must tolerate five-minute interruption. Every route arrival, pre-combat staging point, combat resolution, camp opening, estate activity resolution and major conversation exit is a safe autosave boundary. A player returning after a week receives a concise recap generated from event history, not a generic codex dump.")

add_h(doc,'1.9 Failure, retreat and recovery',2)
add_p(doc,"Failure should create a changed problem more often than a reload requirement. The player may lose a fight, abandon an objective, arrive too late, spend a unique tool, offend a faction, misread testimony or die. Death removes the character until the midnight Return and may produce lost time, lost unsecured salvage, a death memory and a changed return condition. A bad expedition can cost days, resources, standing, access, confidence and opportunity. It cannot quietly destroy the save while continuing to pretend that victory remains possible.")
add_table(doc,['Failure level','Immediate result','Persistent consequence','Recovery path'],[
('Tactical setback','Objective step fails or enemy gains advantage.','Resource loss, position loss, status or enemy adaptation evidence.','Continue under worse conditions, spend a counter or retreat.'),
('Battle defeat','Party is downed, routed, captured or forced from the field according to encounter fiction.','Injuries, lost supplies, time, altered site control or a rescue obligation.','Fallback scene, ransom/escape, ally intervention with cost, or recovery expedition.'),
('Expedition failure','Party returns without the objective or cannot reach it in time.','Lead changes, faction acts first, route becomes harder or opportunity transforms.','Acquire better evidence/equipment, choose another route or pursue the changed consequence.'),
('Household project setback','An estate task lacks time, material or the required specialist.','The upgrade waits, a room remains damaged or a planned celebration changes form.','Acquire the missing resource, assign help or choose another project. Romance membership does not regress.'),
('Campaign-ending decision','Player knowingly accepts a terminal resolution in the finale or an explicitly labeled optional ironman mode.','Ending and epilogue state.','Return to a pre-decision save unless the player deliberately enabled ironman.')],[1.2,1.7,2.05,2.15])
add_p(doc,"Retreat has a target rule, cost and animation like any other tactical action. Some encounters permit immediate withdrawal; others require reaching an exit band, breaking pursuit, protecting an incapacitated companion or surviving one telegraphed round. The interface explains the condition before the player commits resources. The protagonist, heroines and named residents can die and return at midnight under section 2.6. Their pain, interruption, witnesses, death memories and Return outcomes remain persistent consequences even when the body returns.")

add_h(doc,'1.10 Difficulty, accessibility and rule transparency',2)
add_p(doc,"The default Expedition mode should deliver the intended first-edition-D&D danger: uncertain routes, limited expedition resources, enemies that can be wrong for the current party, and consequences that make retreat intelligent. Difficulty must not be produced mainly by inflating health. It changes the generosity of information, recovery, enemy coordination, resource pressure and failure consequences, and it states those changes plainly. Narrative importance never excuses inaccessible interaction; text scale, contrast, input remapping, animation speed and cognitive load options exist independently of challenge.")
add_table(doc,['Mode','Purpose','Principal rule changes','What does not change'],[
('Story','Prioritize world, romance and textual discovery while retaining tactical meaning.','Generous recovery and supplies, clearer intents, softer defeat consequences, optional automatic formation advice.','Authored choices, faction meaning, core enemy capabilities and access to endings.'),
('Expedition — default','Deliver dangerous but recoverable adventure with incomplete information.','Baseline resources, intent confidence, injuries, retreat costs and enemy coordination.','No hidden cheating, random heroine death or irreversible resource deadlock.'),
('Old-School','Reward players who want harsher logistics and less prediction.','Scarcer recovery, broader damage estimates, more severe injuries, stronger pursuit and fewer free camp safeguards.','Interface truth, explicit deadlines, valid recovery paths and content availability.'),
('Custom','Let players tune pressure without accepting an unwanted bundle.','Separate sliders/toggles for damage, supplies, injury, intent clarity, timing, hints and defeat recovery.','Narrative respect and transparent display of active rules.')],[1.15,1.35,2.55,2.05])
add_bullets(doc,[
"Every animation can be accelerated after its first complete viewing; routine sequences can be shortened or skipped without hiding resolution information.",
"Text speed, auto-advance, voice volume, font size, line spacing, high-contrast panels, dyslexia-friendly font option and a reviewable dialogue history are independent settings.",
"Hold inputs can become toggles; rapid taps can become holds; camera shake, flashes, particles, hit-stop and screen motion have separate intensity controls.",
"Controller focus never requires a pointer. Every visible action is reachable, every modal has a consistent cancel path and focus returns to the initiating object.",
"Color is never the only carrier of band, target, status, rarity, confidence or relationship meaning.",
"Difficulty and accessibility changes can be made during a campaign. The game records settings for debugging but does not punish or shame the player."
])

add_h(doc,'1.11 Save, suspend and state continuity',2)
add_p(doc,"A long, branching RPG cannot treat saving as an afterthought or a moral referendum on how players enjoy uncertainty. The game maintains rotating autosaves before and after consequential boundaries, allows manual saves at safe anchors, and offers a single suspend save almost anywhere outside an unresolved write transaction. Suspending captures the exact exploration or battle state, closes cleanly and is consumed only after a successful reload and subsequent save. The player is warned when a rare cinematic or platform restriction delays suspension.")
add_table(doc,['Save type','Created when','Retention','Restores'],[
('Rolling autosave','Departure, route arrival, site entry, pre-combat, post-combat, camp, activity resolution and major conversation exit.','At least twelve chronological slots plus chapter anchors.','The last fully committed campaign transaction.'),
('Manual save','At estate, safe camp, owned vessel or other declared safe anchor.','Player-managed slots with timestamp, chapter, location, party and thumbnail.','Complete deterministic campaign snapshot.'),
('Suspend save','On request during stable exploration, target selection or between battle resolution steps.','One protected slot, removed only after confirmed continuation.','Exact scene and simulation state, including focus context and random stream position.'),
('Chapter anchor','Immediately before a confirmed chapter transition and final decision.','One per chapter, protected from rolling deletion.','Pre-transition state with an explanatory label.'),
('Developer fixture','Curated or generated state used by tests and preview tools.','Version-controlled outside player profiles.','A declared schema version with validation expectations.')],[1.2,1.85,1.55,2.5])
add_p(doc,"The simulation commits consequences transactionally. Presentation may play for several seconds, but a save never contains half of a damage event, half of a relationship change or a removed item without its corresponding result. Stable content IDs are never reused. Every shipped schema version has migrations and archived fixtures. Loading an incompatible or corrupted save preserves the original file, explains the failure in plain language and offers earlier autosaves rather than silently resetting state.")

add_h(doc,'1.12 The first ninety minutes',2)
add_table(doc,['Window','Player experience','Rules introduced','Proof required'],[
('0–15 minutes','Wake on the Black Beach, inspect wreckage, encounter Betty and make the first consequential reading of the environment.','Movement, inspection, attributed observations, one dialogue choice and one tool use.','A careful player discovers a safer option without a tutorial popup giving the answer.'),
('15–35 minutes','Cross a short dangerous route and fight beside Betty.','Cards, active expansion, bands, intent, one attack, one support skill and retreat preview.','Turn ownership and target consequence are understood after one ordinary fight.'),
('35–55 minutes','Reach the estate, treat the aftermath and choose how to use the remaining day.','Household anchor, time slot, treatment, relationship cause and autosave summary.','The player can state why home matters beyond being a menu.'),
('55–75 minutes','Compare two leads, prepare supplies and select a route.','Journal confidence, map risk, equipment conflict and expedition commitment.','Text changes the chosen plan rather than merely decorating it.'),
('75–90 minutes','Complete a compact ruin objective and return with a new problem.','Site action time, environmental hazard, tactical choice, retreat and aftermath.','The complete prepare–venture–return loop creates at least two desired next actions.')],[1.2,2.05,2.2,1.65])
add_p(doc,"Onboarding introduces one decision layer at a time and then asks the player to use it before adding another. It establishes the game’s mature haremlit identity early through attraction, flirtation and household chemistry. Adult scenes occur at their authored romance milestones rather than functioning as tutorial rewards. The first ninety minutes must prove adventure, textual play, tactical spectacle and interpersonal chemistry; if one of those exists only in a promise about later chapters, the slice is not representative.")

add_h(doc,'1.13 Acceptance tests and unresolved decisions',2)
add_bullets(doc,[
"A new player can explain what advances campaign time, what does not, and when a deadline will resolve after the first estate day.",
"After resuming a two-week-old save, a player can identify location, party condition, current intention, last major consequence and two available next actions within two minutes.",
"Every blocked party selection, route, skill, conversation and project exposes a specific reason and, where fiction permits, a discoverable path to eligibility.",
"A complete campaign remains finishable after repeated defeats and poor resource use, though its political and relationship outcomes may be substantially worse.",
"Every battle, exploration interaction and conversation can be completed with keyboard only and controller only, including recovery from every modal interruption.",
"Automated runs can advance the campaign clock, schedules, travel, failure and chapter gates without loading presentation scenes.",
"The prototype demonstrates that MC plus three heroines creates better composition choices and readability than either MC plus two or the entire recruited cast.",
"Story, Expedition and Old-School modes produce measurably different pressure without changing interface truth or requiring separate authored campaigns."
])
add_callout(doc,'Decisions deliberately left open',"Exact day length per expedition, the final number of manual-save slots, whether camps permit remote party exchange after a vessel upgrade, the protagonist's visible presence in every active card, and the precise act count remain working hypotheses. They must be answered by the complete-loop prototype and campaign content budget, not by taste in isolation.")

add_h(doc,'2. The island outside time',1)
add_h(doc,'2.1 Temporal behavior',2)
add_p(doc,"Time displacement should be treated as weather with memory. It has patterns, warnings, local folklore and measurable consequences, but no faction understands the whole phenomenon. Blue moths may gather before a road changes its destination. Iron bells may steady a doorway for one night. A village may insist that a naval war ended twenty years ago while the fort across the bay is still receiving orders from it. These contradictions should not occur every few minutes. If temporal rupture becomes constant decoration it loses its ability to disturb the player's understanding of a familiar route.")
add_p(doc,"Most travel should therefore establish dependable geography. The player needs enough ordinary roads, ferries and landmarks to form expectations. The island breaks those expectations selectively. A northbound jungle trail may emerge at a frozen monastery, a prehistoric beach, a battlefield after all of its soldiers vanished, or the same trail eighty years later. Some events are temporary excursions. Others alter a route, introduce a stranded population, or deposit an object whose historical consequences now become part of the simulation.")
add_h(doc,'2.2 Wilderness, ruins and living dinosaurs',2)
add_p(doc,"The visual and experiential model is an extravagant Stevenson adventure wilderness rather than a fully settled strategy-game island. From the deck of a ship, the player sees immense green relief, broken bronze lines on ridges, storm-dark volcanoes, isolated roofs and river mouths whose interiors remain unsurveyed. The two major cities are bright, dense exceptions attached to harbors. Beyond their last guns, roads become muddy, ancient causeways lose whole spans to the canopy, and every cultivated clearing feels borrowed from a landscape capable of taking it back.")
add_p(doc,"Dinosaurs belong to this ecology everywhere. Small feathered scavengers steal food at the estate and follow market carts. Herd animals open glades, damage crops and migrate along elven processional roads because the grades are easier. River predators make ferries seasonal. Large hunters occasionally enter ruins, plantations or the outer streets of a city when weather, fire or temporal disturbance shifts their territory. The game should never present a fenced dinosaur basin, a zoological exhibit or a single ‘dinosaur region.’ Species distribution varies by habitat, but their presence is an ordinary fact that reshapes architecture, travel, agriculture, hunting and military doctrine across the island.")
add_h(doc,'2.3 Cultures in the colonial present',2)
add_table(doc,['Culture','Present condition','Design responsibility'],[
('Colonial humans','Competing governors, merchants, pirates, missionaries, settlers, laborers and deserters share a language of empire without sharing interests.','Show class, origin and institution; never make “human” a single political bloc.'),
('Colonial orcs','Urban, militarized and internally divided; their powder discipline and engineering make them a major regional power.','Avoid coding physical strength as simplicity. Give officers, workers, reformers, profiteers and families distinct positions.'),
('Fox-folk maritime houses','East-Asian-influenced seafaring networks manage credit, shipping, intelligence and ritual hospitality.','Build a lived culture, not ornamental exoticism. Contracts and kinship should affect quests and trade.'),
('Jungle elves','Descendants of communities that survived the ancient collapse through adaptation, migration and partial custodianship.','Separate modern identity from ancient empire; allow disagreement over tomb access, inheritance and sacred obligation.'),
('Pirate communities','Crews, free ports, smugglers and coastal families occupy the space between predation and practical autonomy.','Treat piracy as labor, governance and violence, not only swagger.')],[1.2,2.45,2.85])
add_figure(doc,'island-map-revised.png','Figure 1 — the island as wilderness interrupted by two colonial cities.','High-magical Bronze Age elven works dominate the built silhouette and disappear repeatedly beneath jungle. Dinosaurs are visible across coasts, rivers, ruins and the interior. Human, orc and fox-folk settlements occupy a small fraction of the land, leaving the routes between them genuinely adventurous.')
add_h(doc,'2.4 Regional plan',2)
add_bullets(doc,[
"The Leeward Harbors: colonial administration, fortified trade, plantations, labor conflict, respectable smuggling and the easiest early roads.",
"The Corsair Coast: coves, wreckers, tidal caves, free settlements and the social economy through which rare goods circulate.",
"The Eastern Reaches: fox-folk ports, shipyards, cliff shrines, disciplined merchant fleets and sophisticated credit networks.",
"The Orcish March: powder foundries, academy towns, engineered roads and a border politics that cannot be reduced to invasion.",
"The Green Interior: dinosaur territory, elven settlements, living ruins, dangerous rivers and paths understood through ecology rather than survey.",
"The Ash Crown: volcanic highlands, ancient ventilation shafts, exposed funerary works and the island's most unstable temporal weather.",
"The Outer Keys: reefs, lighthouse stations, hidden anchorages, drowned sites and sea routes that become essential once the household acquires a vessel."
])
add_h(doc,'2.5 Place-naming rule',2)
add_p(doc,"Places are named by the people who use them, administer them, inherit them or live nearby. A colonial fort is named for a monarch, governor, admiral, saint, patron, military function or headland. A manor carries a family or estate name. A mission carries its dedication. A mine carries an owner, settlement, mountain or extracted material. An ancient elven work carries an inherited proper name, a modern local name or a plain translated institutional function. Later arrivals preserve ordinary names such as a hotel name, flight number or survivor-camp designation. The supernatural premise belongs in conflicting records, physical evidence and behavior; the title does not announce the twist.")
add_table(doc,['Place class','Naming construction','Examples used in this bible'],[
('Colonial fort','Fort plus patron, commander, saint or geographic name.','Fort Calder; Fort Saint Orra; North Cape Battery.'),
('Estate or manor','Family surname, plantation name or geographic estate.','Bellamy House; Marrowfield; Vane Estate.'),
('Religious foundation','Institution type plus dedication or local place.','Cathedral of Saint Orra; Saint Lysa Abbey.'),
('Ancient elven work','Inherited proper name plus translated function when needed.','Veyra Tidehouse; Talar Lower City; Ilyon Necropolis.'),
('Industrial site','Owner, mountain, settlement or material plus function.','Mount Kestrel Glass Quarry; Calder Foundry.'),
('Displaced later site','Original operational name or practical survivor name.','Hotel Bellevue; Camp 309.')],[1.4,2.6,2.5])
add_callout(doc,'Naming rejection test',"Reject a place name if it sounds like a chapter title, reveals the twist, depends on a negative construction such as ‘with no’ or ‘that never,’ or could only have been coined by marketing copy. Record separate endonyms and exonyms only when the cultural disagreement matters in play.")

add_h(doc,'2.6 Midnight Return and the daily population',2)
add_p(doc,"Death on the island is temporary often enough to shape ordinary life. At 00:00 island time, a visible Return pulse crosses the world. Monster habitats flare as new bodies arrive. Dead people who remain eligible for return reappear at their registered return anchors. People have lived with this for generations. They still fear pain, violent death, disfigurement, lost time, helplessness and the possibility of returning changed. They also make appointments for the morning after an execution, argue about inherited debts across several deaths, and sometimes discuss being murdered with the exhausted practicality of people discussing a storm.")
add_callout(doc,'WORLD authority',"Requirements req.world.midnight.return through req.world.death.grudge_limit own the system. Midnight Return is a campaign transaction with world-scale consequences. Scene reloads, map entry and distance from the player never trigger it.")
add_h(doc,'2.6.1 Midnight transaction order',3)
add_numbered(doc,[
"At 23:59:59, CampaignService closes new time-consuming commands and finishes the currently committed transaction.",
"Commit cmd.world.resolve_midnight with world day, campaign act, island pressure, active habitats, persistent character records and the midnight RNG stream.",
"Increment world_day and derive daily_seed = hash(campaign_seed, world_day, island_pressure_version).",
"Resolve every dead persistent character in stable character-ID order. Update counters, return outcome, condition, inventory ownership, location and memory records.",
"Generate every active habitat population in stable habitat-ID order. Choose individual species, levels, traits, conditions, patrol purposes and anchors from the daily seed.",
"Advance daily estate schedules, deadlines, prices, faction operations, weather fronts and Champion decay or preparation after the return population exists.",
"Emit one evt.world.midnight_resolved containing per-character and per-habitat child events. Commit one atomic state delta.",
"Autosave the completed midnight state. Presentation then plays the global flash, local arrivals, sound, text recap and any eligible wake-up scene."
])
add_h(doc,'2.6.2 Persistent character death record',3)
add_table(doc,['Field','Type','Rule'],[
('character_id','ContentID','Persistent named character or promoted recurring actor.'),
('alive','bool','False from committed death until a midnight return or authored exception.'),
('death_count_total','uint32','Increments once for every committed death.'),
('death_count_party','uint32','Increments when the field party is causally responsible.'),
('death_count_protagonist','uint32','Increments when the protagonist delivers or orders the lethal result.'),
('last_death','DeathRecord?','Cause, location, world time, encounter, responsible actor IDs and witnesses.'),
('death_memories','array<DeathMemory>','One immutable record per death; summary may compact after twenty records while counters remain exact.'),
('return_anchor_id','AnchorID','Home, civic return house, faction facility, estate or last approved personal anchor.'),
('return_outcome','ReturnOutcome','Same, Recovered, Changed or Unstable for the current return.'),
('return_effect_ids','array<EffectID>','Mechanical and narrative conditions applied by the outcome.'),
('return_eligible','bool','True by default; explicit world rules may delay a return and must expose evidence.'),
('next_dialogue_ack_id','DialogueID?','Optional acknowledgement chosen from relationship, witness and death context.')],[1.65,1.25,3.6])
add_h(doc,'2.6.3 Return outcomes',3)
add_table(doc,['Outcome','Daily weight','Mechanical result','Narrative result'],[
('Same','55%','Returns at prior persistent attributes with ordinary post-return fatigue for six hours.','Memory is intact; manner and priorities remain continuous.'),
('Recovered','25%','Returns without one eligible pre-death Injury or Body condition.','The person may experience the return as relief, luck or unsettling repair.'),
('Changed','15%','Returns with one bounded Return Trait selected from a reviewed table; the trait has a countermeasure or adaptation path.','A habit, sensory response, appearance detail or conviction shifts while identity remains recognizable.'),
('Unstable','5%','Returns with Return Sickness, reduced Composure and one temporary anomaly tag for one to three days.','Memory may be fragmentary, doubled or accompanied by a perception from another history.')],[.8,.75,2.55,2.4])
add_p(doc,"Weights are defaults for ordinary named people. A character package may replace them with an authored table keyed by campaign state. The daily seed makes the outcome reproducible. The system never rerolls because the player reloads. A Changed outcome draws only from traits approved for that character’s identity package; random generation cannot rewrite sexuality, household commitment, core culture, signature profession or essential personality.")
add_h(doc,'2.6.4 Memory and social consequence',3)
add_bullets(doc,[
"A returned named person knows that they died, how it felt at the level appropriate to the content rating, where it happened and who caused it when that responsibility was observable or later established.",
"Player responsibility unlocks a direct acknowledgement topic. The NPC may mention it without prompting when recency, relationship, temperament or current stakes make it relevant.",
"The death record itself adds zero automatic hostility, fear, attraction, trust, romance or faction change. Those changes occur only through an authored consequence that cites the death record as context.",
"A hostile person may resume the same conflict after returning. A friendly person may treat the killing as an ugly necessity. A comic character may be irritated. The authored response belongs to character identity and circumstance.",
"Dialogue may query exact count bands: first death, second, third to fifth, sixth to tenth and more than ten. It may also query whether this speaker has killed the protagonist or a heroine.",
"Ordinary residents rarely deliver philosophical exposition without cause. Their language shows habituation: practical arrangements, gallows humor, unease around midnight, return insurance, funeral customs that focus on pain and memory, and concern about people who came back different."
])
add_h(doc,'2.6.5 Daily monster population and level generation',3)
add_p(doc,"Every HabitatDef declares its species, ecological roles, carrying capacity, base level, pressure response, patrol anchors, nocturnal behavior and exclusions. Midnight creates a full DailyHabitatState in one transaction. The island receives many monsters, spread across routes and sites. Each generated monster is an individual combatant with a level, species template, two behavior traits, one physical variation, current condition, patrol purpose and loot seed. The system does not manufacture a pack of interchangeable one-hit bodies merely to make an area look busy.")
add_p(doc,"Killed monsters remain absent for the rest of the day. The next midnight creates replacements in visible flashes. A replacement receives a new ID and a new combination of traits. The player can therefore clear a route for today without permanently domesticating the island. Plentiful population produces route pressure, interrupted travel and changing tactical problems. It does not produce endless proximity spawns or enemies appearing behind the player because a timer expired.")
add_table(doc,['Value','Exact calculation or selection'],[
('daily_seed','hash(campaign_seed, world_day, habitat_id, island_pressure_version).'),
('population_individuals','clamp(min_individuals,max_individuals,base_individuals + seeded_variance[-2,+2] + pressure_population_modifier).'),
('monster_level','clamp(1,12, habitat_base_level + campaign_act_modifier + island_pressure_level + seeded_variance[-1,+1]). Determine separately for each individual.'),
('party influence','Excluded from population count, level and species selection.'),
('individual traits','Choose two compatible behavior traits, one physical variation and one current condition from the species tables.'),
('encounter assembly','Select one to three individuals whose combined Threat Cost fits the route budget. Four requires a Swarm or Formation tag and explicit budget.'),
('placement','Choose distinct legal patrol anchors; reject occupied civic safe zones, sealed persistent spaces and another individual’s minimum patrol radius.'),
('elite chance','Habitat elite weight plus island pressure. Every elite has a visible model, route clue or behavior tell before commitment.'),
('persistence','Store individual ID, species, level, traits, condition, patrol purpose, anchor, alive state and loot seed until the next midnight.'),
('midnight visibility','Active local space plays individual arrival flashes; remote habitats update in state and may be reported through sound, sky glow, testimony or the morning map.')],[1.45,5.05])
add_h(doc,'2.6.5.1 D&D-style level power',3)
add_p(doc,"Level is a real statement about power. A level-7 allosaur is not a level-3 allosaur with more health. It controls more space, reads weak formations, resists low-grade control, hits through improvised guard and has a developed species technique. A high-level creature can dominate a route by itself. A low-level creature can remain relevant through terrain, surprise, poison, flight, water, concealment or a narrow vulnerability, but the generator may not fake difficulty by adding twenty disposable copies.")
add_table(doc,['Enemy level relative to party median','Encounter meaning','Mechanical treatment','Expected player decision'],[
('-4 or less','Overmatched individual. Still a living obstacle, not a confetti target.','Player attacks gain +20 hit; control duration +1; enemy direct damage x0.60. One clean signature hit may defeat it.','Avoid unnecessary cost, bypass, capture, frighten or finish quickly.'),
('-3 to -2','Inferior threat with one credible trick.','Player attacks gain +10 hit; enemy damage x0.80; normal species traits remain active.','Control the trick and conserve supplies.'),
('-1 to +1','Peer threat. One monster can demand a full tactical exchange.','Use baseline hit, damage, resistance and action rules.','Read tells, spend guard and exploit party composition.'),
('+2 to +3','Superior threat. It can down an exposed heroine in one committed sequence.','Enemy attacks gain +10 hit, +2 Potency and +1 control resistance; its advanced species action is enabled.','Prepare a counter, use terrain, accept injury risk or retreat.'),
('+4 or more','Dominant threat. The route belongs to it until the party earns an answer.','Enemy attacks gain +20 hit, +4 Potency, +2 control resistance and boss-style stagger protection.','Observe from safety, find a weakness, recruit help, reroute or return later.')],[1.45,1.65,2.0,1.4])
add_p(doc,"Level modifiers never grant absolute immunity to ordinary attacks. They change probability, damage, control reliability and action access. Specific armor, scale, incorporeality or tomb authority may still create a typed immunity, but the bestiary must name the immunity and its counter. This preserves first-edition asymmetry while keeping the interface honest.")
add_h(doc,'2.6.5.2 Individual monster record',3)
add_table(doc,['Field','Required content'],[
('monster_id','Stable daily ID derived from world day, habitat and individual index.'),
('species_id and level','Species rules plus individual level from 1 to 12.'),
('behavior_traits','Two tags such as Ambusher, Nest-Guard, Carrion-Following, Duelist, Gun-Shy, Light-Hating or Relic-Bound.'),
('physical_variation','One readable variation such as scarred jaw, pale scales, broken horn, funerary brands, powder burns or temporal afterimage.'),
('current_condition','Healthy, hungry, wounded, gravid, territorial, displaced, Return-sick or another species-approved condition.'),
('patrol_purpose','What it is doing here: hunting, crossing, feeding, guarding, following a faction, defending young or responding to an anomaly.'),
('combat_signature','Basic action, species action, level-unlocked action, reaction, retreat trigger and surrender or capture rule when applicable.'),
('evidence','Tracks, sound, damage, odor, silhouette, testimony or journal clue available before encounter commitment.'),
('reward','Ordinary salvage, research value, faction relevance and persistent first-discovery claim. No repeated mastery payout.')],[1.6,4.9])
add_h(doc,'2.6.6 Death, return and loot boundaries',3)
add_table(doc,['Case','Rule'],[
('Named civilian or faction character','Body remains until midnight or lawful handling. Bound signature possessions return with the person; deliberately transferred legal property follows authored ownership state.'),
('Anonymous monster','Drops its generated ordinary salvage once. The replacement receives a new daily individual ID, traits, condition and loot seed at the next midnight.'),
('Unique monster or boss','Returns only when its package permits it. The return may advance its memory, form or Champion adaptation. Unique progression rewards remain claimed.'),
('Protagonist','Returns at the last committed safe anchor at midnight. Battle defeat commits lost time, expedition failure, dropped unsecured salvage and a Return outcome before control resumes.'),
('Core heroine','Returns at her registered household or expedition-safe anchor. Relationship continuity, death memory and injury/return outcome persist. Random permanent death is excluded.'),
('Execution or murder investigation','The crime remains socially real. Witnesses, law, pain, property loss and interruption persist even though the victim may return.'),
('Midnight during camp','The party witnesses local flashes, new surrounding populations and any returning member. Camp safety is re-evaluated after population generation.'),
('Midnight during combat','Time cannot cross midnight inside an unresolved combat transaction. The encounter resolves or reaches a declared suspension boundary first.')],[1.75,4.75])
add_callout(doc,'Farming control',"Daily repopulation creates renewed danger and ecology, not unlimited unique rewards. Quest items, mastery rewards, boss claims and first-discovery rewards are persistent campaign facts. Ordinary salvage can recur once per generated individual because supply pressure already prices the time, danger and return journey.")
add_h(doc,'2.6.7 Required implementation tests',3)
add_bullets(doc,[
"test.world.midnight_transaction: identical pre-midnight state and seed produce byte-equivalent post-midnight authoritative state.",
"test.world.death_counters: each committed death increments the correct counters exactly once across save, reload and midnight.",
"test.world.death_no_automatic_grudge: applying a death memory alone changes no relationship or faction track.",
"test.world.daily_population_seed: generated populations are independent of party level and stable across reload.",
"test.world.spawn_individuality: every generated monster has two behavior traits, one physical variation, one condition, one patrol purpose and one stable daily ID.",
"test.world.level_delta_curve: each level-difference band applies the declared hit, damage, resistance and action changes without silent scaling.",
"test.world.encounter_actor_cap: ordinary assembly produces one to three enemies; four requires a declared Swarm or Formation tag and legal Threat Cost.",
"test.world.return_condition_table: every outcome obeys character exclusions and cannot select an unapproved identity rewrite.",
"test.world.midnight_safe_zone: no monster anchor resolves inside an active civic or estate safe zone.",
"test.world.daily_loot_claims: one generated individual yields ordinary salvage once; persistent rewards never reset.",
"test.world.midnight_camp_recheck: camp attack forecast and escape routes update after nearby populations return."
])

add_h(doc,'3. The player fantasy and daily rhythm',1)
add_p(doc,"The player repeatedly moves through the six-step session loop defined in section 1.8: orient, prepare, commit, press or withdraw, resolve, and project forward. At its broadest this still feels like leaving home and returning changed, but the six-step version is the implementation contract because it names the decision boundaries, save points and information each phase must provide. Equipment matters because an expedition can strand the party. Knowledge matters because the right rumor changes the risk of a route. Relationships matter because companions interpret danger differently and remember how the protagonist treated them when survival was expensive.")
add_numbered(doc,[
"Orient at the current safe anchor: review location, time, injuries, supplies, recent consequences, companion concerns and the decision presently awaiting commitment.",
"Prepare: compare evidence, choose the field party, resolve equipment conflicts, pack the load and select an objective and route.",
"Commit: accept the stated time, supply and uncertainty profile of the first route segment and leave the safe anchor.",
"Press or withdraw: travel and explore, spend limited tools, negotiate or fight, record evidence and continually reassess whether the objective remains worth its cost.",
"Resolve at camp, another safe anchor or the estate: treat injuries, settle immediate consequences, distribute salvage and commit the expedition ledger.",
"Project forward: expose how knowledge, relationships, faction reactions and household changes have created the next practical choices."
])
add_h(doc,'3.1 Knowledge as progression',2)
add_p(doc,"Character levels improve numerical capacity, but the more distinctive progression is the accumulation of usable knowledge. The journal should record observations in the player's language without converting every mystery into a checklist. An entry may state that a brass funerary door vibrated when Nara's astrolabe faced west, that Ayla refused to cross while water was running, and that a recovered maintenance tablet depicts the channel dry. The interface has done its job: it preserved evidence. It does not need to announce the solution.")
add_callout(doc,'Modern UX boundary',"The journal records what the party actually learned, the map distinguishes certainty from rumor, inventory comparisons expose consequences, and failed actions explain their mechanical cause. The game never hides a rule that the protagonist would plainly understand.")
add_h(doc,'3.2 Text as a first-class play surface',2)
add_p(doc,"The world should be seen, heard and read. Painted environments establish space, scale, weather and bodies, but prose can describe temperature, smell, remembered comparison, uncertain perception and culturally specific interpretation with a precision that an image cannot provide by itself. Entering a location therefore produces a concise authored description whose wording changes with time, companions, injuries, knowledge and prior choices. Inspection is not a tooltip that repeats the art. It is an additional sensory and intellectual layer: what the bronze tastes like in the air, why the channel’s silence troubles Ayla, which military habit Grisha recognizes in the spacing of the dead, or what Betty notices about a wound before anyone names the creature that made it.")
add_p(doc,"Descriptions should occasionally be actionable. The player may select a phrase to ask a companion about it, compare two contradictory accounts, copy an inscriptional pattern into the journal, or choose which detail to follow when time is short. A route can be navigated through written landmarks. A conversation can turn on whether the protagonist repeats a colonial scholar’s name for a structure or the name Ayla taught him. A monster may be identified from behavior described in prose before its body is fully revealed. These interactions make close reading valuable without turning every paragraph into a hidden-object test.")
add_table(doc,['Textual surface','What it contributes','How the player acts'],[
('Arrival description','Mood, sensory facts, changed state and the party’s immediate interpretation.','Expand details, ask a present companion, compare with the last visit.'),('Focused inspection','Material, workmanship, ecology, tracks, damage and uncertain hypotheses.','Record a clue, use a tool, touch, wait, test, or deliberately leave alone.'),('Travel narration','Distance, fatigue, weather, route memory, wildlife and social observation.','Change pace, consume supplies, divert, camp, hide, follow or turn back.'),('Combat narration','Readable consequences that animation cannot fully express: smell, pain quality, alien pressure, remembered tells.','Open the event log, inspect a condition, connect a described tell to journal knowledge.'),('Journal dossier','Quoted observations, paraphrased testimony, contradictions and provenance.','Pin evidence, form a working theory, share or conceal it from a faction.'),('Household chronicle','Small domestic changes, private letters and the emotional residue of expeditions.','Respond, revisit a conversation, respect privacy, plan around a resident’s state.')],[1.25,2.75,2.5])
add_callout(doc,'Writing rule',"Text adds information, perception or choice. State the design directly. Use negation for a literal prohibition or boundary. Avoid the rhetorical frame ‘not X but Y.’ Never define a heroine, culture, romance, location or skill by arguing against an inferior imaginary version. Rich prose earns its space through useful detail, consequence and point of view.")
add_h(doc,'3.2.1 Action-file writing standard',3)
add_p(doc,"Use the prose discipline of a strong action-figure file card or mission brief. Name the subject. State what the subject does. Name the target. State the visible result. State the limit. The writing may describe a dense and extravagant world, but it may not conceal game logic inside metaphor, attitude or trailer copy. A player, writer, animator and programmer should extract the same action from the same sentence.")
add_table(doc,['Content type','Required sentence pattern','Approved example','Reject'],[
('Skill','Actor + verb + target + exact result + duration or cost.','Grisha hooks one adjacent enemy, pulls it one band and applies Exposed for one round.','Grisha turns authority against the enemy line.'),
('Character bio','Former role + decisive incident + present capability + present objective + household behavior.','Betty served as a ship’s surgeon. She broke quarantine orders, saved the crew and lost her commission. She now protects the field party with medicine and a boarding mace.','Betty has learned that care can be a weapon.'),
('Enemy','Body plan + habitat + attack method + defense + behavior + reward or reason to avoid.','A level-5 river tyrant controls the ford, charges exposed targets and resists light firearms. It retreats after a broken jaw unless defending eggs.','The river remembers its oldest hunger.'),
('Location','Owner or builder + physical function + present controller + current hazard + player use.','Fort Calder guards the western harbor mouth. Orc marines hold its guns. A funerary drain beneath the powder magazine returns the dead at noon.','A fortress caught between history and death.'),
('Quest step','Player action + destination or target + success test + immediate consequence.','Carry the bronze valve to Veyra Tidehouse, install it before low tide and keep the maintenance channel clear for three rounds.','Restore what the elves abandoned.'),
('Production instruction','Owner + artifact + required fields + acceptance test.','Animation creates five panels for skill.betty.healing_impact and keeps Betty, target, mace and full VFX inside the eight-percent frame margin.','Make the attack feel powerful.')],[1.0,2.05,2.15,1.3])
add_callout(doc,'Tone test',"If a sentence could appear unchanged in a trailer voice-over, rewrite it. If it tells an implementer who acts, what changes and what proves completion, keep it.")
add_h(doc,'3.3 First-edition danger without inherited inconvenience',2)
add_p(doc,"The deliberate old-school quality belongs in the world's distribution of danger and reward. A magnificent weapon may appear before the party can use it safely. A sealed room may contain a creature whose sensible solution is to close the door. Rumors may be wrong because people are wrong. Some enemies have narrow vulnerabilities that make remembered folklore more useful than another level. Consumables can be genuinely precious. Retreat should be a respected tactical choice, not a disguised failure state.")
add_p(doc,"None of that excuses hidden arithmetic, inventory misery, illegible targeting, or loss of hours to an unforeseeable save restriction. The player can inspect what an item changes, understand why an attack missed, see the probable cost of travel, and save in a manner appropriate to the platform. The island is allowed to be janky in the old tabletop sense: surprising, asymmetric, occasionally rude, and generous enough to produce stories. The application must not be janky in the ordinary sense.")

add_h(doc,'3.4 What we will actually build for expedition play',2)
add_p(doc,"The expedition game is not one enormous seamless island scene. It is a stateful network presented through three deliberately different scales. The illustrated overworld is where the player compares destinations and commits to route edges. Travel segments are short authored or systemic decisions that consume time and resources without pretending that walking every mile is interesting. Sites are side-view room graphs in which the player controls the expedition leader directly, reads the environment, positions the party around hazards and enters conversations or battles without changing to a different fiction. These scales share one ExpeditionState, so supplies spent on the road, noise created at a door, an injury suffered in combat and a promise made at camp remain parts of the same journey.")
add_callout(doc,'Build contract',"For the complete-loop prototype, build one estate anchor, a three-node overworld, two route edges, one camp state, one twelve-space tomb and one return route. Do not build free-roaming ocean travel, procedural islands, a day-night lighting simulation or a generic dungeon generator. The prototype must prove authored reactivity and persistent state before breadth.")
add_table(doc,['Playable scale','Concrete thing to build','State it reads','State it writes'],[
('Overworld planner','Illustrated node-and-route screen with party, weather, duration, transport, supply and confidence comparison.','Discovered nodes and edges, route knowledge, weather window, faction control, transport, party condition.','Chosen edge, departure time, packed load, tracked objective and declared pace.'),
('Travel segment','One decision card or compact side-view interruption with 2–4 meaningful responses.','Edge tags, current weather, party skills, supplies, prior route events and time.','Time, fatigue, supplies, observations, edge condition, encounter pressure and optional detour.'),
('Exploration site','Connected side-view spaces with walk anchors, interaction anchors, exits, hazards and encounter volumes.','Site persistence, party knowledge, local alert, light, weather, clock and companion presence.','Room state, local alert, observations, opened paths, consumed tools, objectives and retreat route.'),
('Camp','Safe, pressured or forbidden camp interface attached to a route or site anchor.','Camp quality, pursuit, weather, party needs, food, medicine, watch skills and relationship eligibility.','Recovery, time, watch assignments, conversation outcomes, crafted items and ambush state.'),
('Return and aftermath','Expedition ledger followed by treatment, spoils, knowledge and household reactions.','Complete expedition event history and unresolved obligations.','Campaign facts, injuries, inventory, faction reactions, relationships, project progress and new leads.')],[1.25,2.2,1.7,1.85])

add_h(doc,'3.5 The overworld is a decision surface, not a movement toy',2)
add_p(doc,"The island map should feel illustrated, uncertain and physically desirable without turning into a cursor-driven list of quest markers. A destination appears because the party has seen it, received testimony about it, inferred it from evidence or found a route that implies it. Its marker shape communicates certainty. A known place has a surveyed silhouette; a reported place is drawn in another person's hand; an inferred place is a bounded region rather than a false pin. Selecting a destination reveals the routes the party currently believes are possible, not every route that exists in the data.")
add_p(doc,"A route preview is a comparison of commitments. It shows the expected travel window, supply estimate, transport requirement, known terrain, recent weather, faction exposure, dinosaur pressure, temporal warnings, camp opportunities and the companions whose expertise altered the estimate. Exact numbers appear when the party genuinely knows them. Otherwise the preview uses a range and states why it is uncertain. ‘One to two days; bridge condition unverified since the storm’ is useful uncertainty. ‘???’ is not design.")
add_bullets(doc,[
"The player may pin one primary objective and up to two opportunistic stops. Additional discoveries remain visible but do not become a checklist stapled to the journey.",
"Pace has three values: cautious, standard and forced. Pace changes time, encounter exposure, observation quality, fatigue and the chance of arriving ready rather than merely movement speed.",
"Transport is a capability with constraints. Walking, pack animals, canoe, hired carriage and owned vessel each enable different edges and create different event families; they are not cosmetic fast-travel skins.",
"A route can be safe in one direction and dangerous in the other because of current, slope, tides, pursuit, daylight or the difficulty of carrying an injured person.",
"Fast travel is earned route competence. Once an edge is surveyed and its current hazard conditions are satisfied, the player may collapse uneventful traversal while still paying declared time and supplies.",
"No map marker exists solely because a quest needs somewhere to point. Every named node offers a service, decision, persistent site, faction presence, route function or repeatable observation value."
])

add_h(doc,'3.6 Route-event grammar',2)
add_p(doc,"Travel events must be short enough that choosing a route does not mean accepting a procession of interchangeable pop-ups. Each edge declares an event budget and pressure profile. An ordinary one-day edge should usually generate zero or one consequential interruption; a dangerous edge may generate two, but one of them should be visible in the route forecast. Events are selected from authored families filtered by habitat, weather, time, faction, dinosaur ecology, anomaly pressure, party composition and recent repetition. Selection is deterministic from the expedition seed so saves and tests can reproduce it.")
add_table(doc,['Event family','Question posed','Reusable resolution grammar'],[
('Obstacle','How will the party cross, bypass, repair or exploit a physical obstruction?','Spend time, tool, supply or specialist capability; accept risk; discover another edge; turn back.'),
('Ecology','How will the party behave around animals whose territory or migration intersects the route?','Observe, wait, divert, conceal food, deter, hunt, fight or exploit movement as cover.'),
('Social contact','What does the party owe, reveal or risk when meeting travelers, patrols or settlements?','Greet, identify, trade, deceive, aid, avoid, submit to inspection, negotiate passage or escalate.'),
('Evidence','Will the party stop to understand something that may change a later decision?','Observe while moving, spend time inspecting, ask a companion, record, collect or leave undisturbed.'),
('Temporal weather','Will the party trust instruments, folklore, visible signs or urgency when the road begins to disagree with itself?','Anchor, shelter, press on, follow a displaced feature, mark the event or abandon the edge.'),
('Party pressure','Whose need, objection, injury or opportunity matters enough to interrupt the plan?','Listen, postpone, spend time, change assignment, keep moving with a stated consequence or retreat.')],[1.2,2.2,3.0])
add_p(doc,"Every event records why it was eligible, why it was selected and why each response is present or absent. A writer can preview the event with any legal party and state. A tester can force it by stable ID. If an event cannot explain its eligibility without reference to an unregistered flag or a scene-tree accident, it is not ready to enter content production.")

add_h(doc,'3.7 Side-view site exploration',2)
add_p(doc,"A site is authored as a graph of spaces rather than as one continuous collision map. A space may be a beach landing, tomb reception court, broken stair, ossuary gallery, flooded maintenance channel or ritual chamber. Within the current space the protagonist moves laterally between authored anchors, companions follow using reserved stand points, and the camera frames the location as a painted theatrical composition. Exits connect spaces and may be open, concealed, blocked, dangerous, one-way or dependent on a known rule. This structure supports rich backgrounds and persistent changes without requiring a seamless world whose edge cases consume the project.")
add_p(doc,"The player does not pixel-hunt. Interactive anchors receive restrained visual grounding when nearby or deliberately highlighted, and the prose may name a feature before it becomes actionable. An anchor can expose several verbs depending on equipment and knowledge. A bronze spillway might be observed from a distance, inspected closely, tested with water, compared with a tablet, discussed with Ayla, forced with a lever or left untouched. The art object remains one feature; the available interaction set changes through state.")
add_table(doc,['Exploration object','Minimum authored fields','Runtime responsibility'],[
('Space','Stable ID, background layers, walk line, camera bounds, light profile, ambience, entry anchors, companion stands.','Restore visual layers from persistence; place party safely; report current environmental state.'),
('Exit','Source/destination anchors, direction, traversal time, visibility condition, lock or hazard, retreat relevance.','Preview consequence, validate traversal, move party and commit time/state atomically.'),
('Interaction anchor','Feature reference, proximity, verbs, condition groups, text variants, tool sockets, state changes.','Explain available and blocked verbs; assemble prose; resolve command; append event history.'),
('Hazard','Trigger, telegraph, affected area or target rule, escalation, counter tags and reset policy.','Expose perceivable warning, resolve deterministic consequence and persist mitigation.'),
('Encounter volume','Participants, objective, entry formation, escape rule, reinforcements, aftermath and repeat policy.','Freeze exploration safely, create BattleState, project result back to the exact space.'),
('Observation source','Sensory facts, expertise inserts, evidence IDs, confidence changes and journal links.','Compose legal output, attribute knowledge and prevent duplicate discovery rewards.')],[1.25,2.7,2.45])

add_h(doc,'3.8 The interaction verb set',2)
add_p(doc,"The game should not invent a new control scheme for every clever room. Most field interactions are expressed through a small vocabulary whose consequences can still be highly specific. The command strip shows only verbs that make sense for the selected feature, with unavailable but intelligible options shown when their absence teaches the player something useful.")
add_bullets(doc,[
"Observe: receive what can be learned safely from the current position. It never advances time and never triggers a concealed punishment.",
"Inspect: approach or spend attention to obtain material, ecological, historical or bodily detail. It may expose the party to a clearly telegraphed local hazard but does not consume campaign time unless labeled as extended search.",
"Discuss: ask one present companion to interpret a selected phrase, object or contradiction. The response depends on expertise, relationship, politics and what she is willing to disclose.",
"Use: apply a selected inventory tool, signature capability or contextual object. Preview consumption, time and obvious risk before confirmation.",
"Manipulate: pull, rotate, open, close, carry, brace, cut, pour or otherwise change the feature through an authored command. Complex manipulation is a sequence of these named steps, not an unexplained puzzle minigame.",
"Record: preserve an observation, rubbing, measurement, sketch, sample or testimony as evidence. Recording itself is normally free; collecting a physical sample may consume capacity or alter the site.",
"Wait: advance to a declared condition such as lower tide, sunset, patrol passage or machine cycle. The preview states the expected duration and current risks.",
"Leave: close the interaction without commitment. The player is never trapped in a feature panel because he inspected the wrong thing."
])

add_h(doc,'3.9 Prose assembly and close-reading play',2)
add_p(doc,"Arrival and inspection prose should be assembled from controlled blocks rather than generated from arbitrary combinations of adjectives. Each textual surface has a base passage that is grammatically complete, followed by ordered insert slots: changed-state notice, protagonist-background perception, one highest-priority companion observation, one injury or condition perception when relevant, and one knowledge-dependent interpretation. A second companion may interject only when the relationship or disagreement is itself the point. This keeps prose rich without allowing six companions, weather, time, faction state and every prior choice to compete inside one paragraph.")
add_table(doc,['Prose block','Authoring rule','Player-facing behavior'],[
('Base description','Complete without inserts; establishes geometry, immediate sensory character and interactable subjects.','Always available on first arrival and reviewable afterward.'),
('Changed-state notice','Names the most consequential difference since the last confirmed visit.','Presented first on return; links to the event or elapsed interval that caused it when known.'),
('Expertise insert','One observation selected by explicit relevance priority, not random banter.','Attributed to protagonist background or present companion; may add evidence or a verb.'),
('Uncertainty phrase','Uses authored confidence language tied to evidence class.','Can be compared with testimony and revised without erasing the original observation.'),
('Action phrase','Marks a phrase or object that supports discuss, compare, record or investigate.','Keyboard/controller focus treats it as a normal reachable action, never as tiny inline pixel targeting.'),
('Journal extraction','A concise proposition with source, time, place and evidence links.','Recorded once, updated through provenance rather than duplicated as another quest log sentence.')],[1.25,2.65,2.5])
add_p(doc,"The writing preview tool must enumerate every legal passage variant for a selected state matrix, identify unreachable blocks, highlight outputs that exceed the reading budget and detect combinations that produce duplicate facts or pronoun failures. Prose is allowed to be florid because the assembly system preserves room for thought; it is not allowed to be combinatorially unreviewable.")

add_h(doc,'3.10 Expedition resources and load',2)
add_p(doc,"The player packs categories of useful capability rather than managing a warehouse of nearly identical objects. Each expedition has a load allowance derived from transport, containers, party condition and estate upgrades. Signature weapons and ordinary clothing do not compete with food. The meaningful burden is expedition gear: medicine, ammunition, food, light, camp equipment, climbing gear, salvage tools, ritual materials, trade goods and reserved capacity for bringing something back.")
add_bullets(doc,[
"Capacity is measured in whole load units, usually one to three per item stack. No fractional weight arithmetic is exposed to the player.",
"Food is packed in party-days. The interface calculates the current field party's expected consumption and makes the risk of an extra night obvious.",
"Medicine is a shared treatment stock with named special preparations for exceptional conditions. Betty improves use; she does not conjure an unlimited parallel inventory.",
"Ammunition is tracked by useful bundle or prepared shot type, not individual generic bullets. Signature ammunition remains distinct when its tactical choice matters.",
"Light is measured in reliable exploration intervals and can also be provided by environmental or magical sources with different hazards.",
"Tools have tags, durability or limited charges only when depletion creates a decision. A shovel that never breaks and has no alternative is a capability unlock, not inventory clutter.",
"Salvage consumes reserved capacity. The party may cache, mark, dismantle, carry, abandon or arrange later recovery; treasure is allowed to create a logistics problem.",
"Companions comment on conspicuous omissions before departure. The game does not automatically correct the loadout, because knowingly taking an underprepared expedition is legitimate play."
])

add_h(doc,'3.11 Camps as field households',2)
add_p(doc,"A camp is the household loop under pressure. It is not a universal heal button and not a random-encounter tax. The party chooses a known or improvised anchor, sees its shelter, concealment, water, fire, dinosaur and anomaly qualities, and assigns the night. One companion keeps each watch, while others may treat, repair, research, cook, sleep or participate in an eligible conversation. The protagonist performs one evening activity and one watch or sleep assignment. These choices create fatigue and opportunity rather than a schedule puzzle that requires perfect optimization.")
add_p(doc,"Camp conversations read the same authored milestone and scene flags as the estate, with camp-specific eligibility. Exhaustion, danger, an object recovered that day or the person sharing a watch can enable a scene. Each adult camp scene is written for a particular anchor quality and story situation; a bedroll beside an active predator route does not substitute for a private room at home. If a camp is attacked, the opening formation and available equipment reflect assignments the player actually made.")
add_table(doc,['Camp quality','Rest outcome','Conversation/privacy','Attack consequence'],[
('Secure','Full ordinary recovery within injury limits; reliable treatment and cooking.','Private pair scene possible; longer research or relationship scenes allowed.','Rare and strongly telegraphed; party begins prepared unless infiltrated by a specific capability.'),
('Exposed','Partial fatigue recovery; fire and treatment choices may increase detection.','Short conversations; privacy requires a selected lookout or terrain feature.','Watch skill determines warning; sleepers begin compressed and may lose initial readiness.'),
('Hostile','Only emergency treatment, rationing and brief rest; some actions unavailable.','Conflict, reassurance and watch dialogue only; no implausibly relaxed intimacy.','Attack pressure is known. The player assigns a withdrawal plan and opening formation.'),
('Forbidden','Local rule, pursuit, flooding or temporal instability makes camping invalid.','None until the party reaches another anchor.','The interface explains the rule and nearest known alternative rather than accepting and punishing the choice.')],[1.0,2.0,2.2,2.2])

add_h(doc,'3.12 Persistence, reset and authored change',2)
add_p(doc,"Every site feature declares its persistence policy. Permanent changes survive indefinitely: a door opened, a bridge destroyed, a body buried, an artifact removed. Visit changes last until the expedition ends or a named reset condition occurs: a movable ladder, a drained cistern, a frightened herd. Cycle changes belong to tides, weather, machinery or temporal phases and can recur without pretending the world forgot the player. Cosmetic ambience may vary freely but cannot contradict recorded facts.")
add_p(doc,"A site does not save the entire scene tree. It saves a compact SiteState keyed by stable feature IDs: discovered spaces and exits, feature-state enums, local alert, removed or placed entities, active cycle, unresolved encounters and authored counters. On entry, the scene projects from that state. This is the only acceptable source of persistence truth. A broken statue may select a damaged art layer and different inspection text, but the visibility of that layer does not itself decide whether the statue is broken.")
add_callout(doc,'Reset honesty',"If an enemy, harvestable resource, puzzle element or route condition returns, the fiction and journal must support why. Respawn is a rule of ecology, labor, ritual maintenance or temporal cycling—not a scene reload side effect.")

add_h(doc,'3.13 Exact complete-loop expedition content',2)
add_p(doc,"The first implementation target is deliberately concrete. The estate begins with Betty's infirmary corner, a chart table and one departure interaction. The overworld offers the estate, a damaged river landing and the tomb reception terrace. The safe road is longer and crosses a visible herb-grazing dinosaur route; the short jungle edge has a washed bridge, one optional evidence event and a higher ambush pressure. The player can camp at the river landing or press directly to the tomb. These are enough elements to prove route comparison, time, party advice, resource packing, travel events, camp assignments and return consequences.")
add_table(doc,['Tomb space','Primary purpose','Persistent proof'],[
('1. Reception terrace','Establish threshold etiquette, dinosaur traces and two visible entrances.','Approach observed; companion objection or approval recorded.'),
('2. Processional ramp','Teach that slope, drainage and burial rank govern movement.','Debris cleared, route marked or dangerous shortcut opened.'),
('3. Registry court','Present names, damaged records and the first evidence comparison.','Inscription recorded; lineage hypothesis created.'),
('4. Water stair','Introduce a cycle condition and a route available only at low flow.','Valve state and water cycle persisted.'),
('5. Petition gallery','First ordinary encounter with a non-kill objective and retreat exit.','Encounter aftermath and damage layers persisted.'),
('6. Service crawl','Offer a cramped bypass, tool use and a salvage-capacity decision.','Grate, cache and noise state persisted.'),
('7. Mourning court','Provide a safe but emotionally charged camp anchor after its rite is understood.','Camp permission and memorial choice persisted.'),
('8. Ossuary lift','Combine machinery, load, noise and an optional shortcut.','Lift position, counterweight and alert persisted.'),
('9. Flooded archive','Make textual evidence change the boss plan rather than merely explain lore.','Recovered dossier, copied evidence and damaged shelf state persisted.'),
('10. Custodian bridge','Elite encounter whose intent becomes clearer through prior observations.','Custodian disposition, bridge condition and alternate crossing persisted.'),
('11. Sealed petition room','Optional dangerous door that is sensible to leave closed.','Seal knowledge, opening attempt and contained threat persisted.'),
('12. Burial-lord chamber','Resolve the tomb rule through negotiation, combat, ritual compliance or retreat.','Tomb authority, artifact custody, faction claim and return route changed.')],[1.35,2.65,2.15])
add_p(doc,"This tomb is not twelve rectangular rooms with fights distributed between them. It is twelve authored spaces that force the systems to meet: prose changes verbs, evidence changes tactical preview, time changes water, route decisions change arrival condition, camp choices change formation, and the final resolution changes both the site and the estate aftermath. If the implementation cannot preserve and explain those relationships, expanding to another site would only multiply a broken architecture.")

add_h(doc,'3.14 Expedition authoring and validation',2)
add_numbered(doc,[
"Create stable IDs for the destination, edges, spaces, exits, features, observations, events, encounters and aftermath outcomes before scene dressing begins.",
"Write the state diagram and persistence policy for every feature that can change. A feature with no declared reset policy fails review.",
"Gray-box the route graph and twelve-space tomb using anchors, labels and collision before producing painted backgrounds.",
"Author base descriptions and interaction verbs against the gray box. Confirm that reading changes at least one route, resource, encounter or relationship decision.",
"Connect one deterministic travel seed, one camp assignment matrix, two combat encounters and one noncombat resolution to the same ExpeditionState.",
"Run the site from departure through return, then reload at every autosave boundary and compare state/event histories.",
"Use the site-variant preview to inspect first visit, partially explored, defeated, peacefully resolved, flooded and post-return states without replaying the campaign.",
"Only after these states pass should environment art, final choreography, voice, ambient behaviors and extended prose variants enter production."
])
add_bullets(doc,[
"Validator: every exit has a reachable destination or an explicitly allowed one-way terminal state.",
"Validator: every consumed item, time advance, injury, fact and relationship change appears in the expedition event ledger.",
"Validator: every blocked verb has a player-readable reason or is intentionally hidden because the action itself is not yet conceivable.",
"Validator: every critical-path obstacle has at least two resolution categories and one recoverable failure or retreat path.",
"Validator: every prose insert can be enumerated, attributed, localized and read without another insert being present.",
"Validator: every site state can be loaded without relying on the order in which rooms were originally visited.",
"Validator: no route event, hazard or encounter repeats before its declared cooldown unless the repetition is the authored point."
])

add_h(doc,'4. Combat as a theatrical party performance',1)
add_p(doc,"Combat uses a side-view battle plane because the game's competitive advantage is the cast. The system should spend screen space on adult full-body figures with readable posture, weapon handling and emotion. Four active party members remain present as compact cards when they are not acting. Selecting a card lifts it, opens its bronze frame and draws that character upward into the battle plane. The actor lands at fighting-game scale, performs the action, holds a readable recovery, and then either remains expanded because of the current threat state or compresses back into the party rail. This grammar applies to the protagonist as well as the heroines; his card and actor require the same production discipline even though the heroines remain the principal cast focus.")
add_p(doc,"Compression is the combat staging system. It keeps the full party visible while reserving the painted battle plane for the acting character. Expansion gives each action ownership. When Betty steps out with her mace, the player reads her anticipation, the ampoule in her off hand, the force of the strike and the healing arc returning to the party. Ayla’s spear and stance produce a different silhouette the instant her card expands. The transition communicates turn order, targeting and character identity.")
add_figure(doc,'battle-ui.png','Figure 2 — card-to-active battle presentation.','Cards preserve party state and turn readiness at a glance. The selected heroine occupies the visual center at full-body scale; enemies retain full silhouettes; the command rail sits low enough that VFX can travel through the battle plane without obscuring targets.')
add_h(doc,'4.1 Core loop',2)
add_bullets(doc,[
"Read intent: enemy tells, initiative order, hazards, cover, formation pressure and status icons are visible before commitment.",
"Choose an actor: selecting a party card previews legal targets, projected movement and known resource costs.",
"Choose one action: basic weapon technique, guard, reposition, item, signature skill or contextual interaction.",
"Resolve with authored timing: anticipation, contact, hit pause, reaction, VFX decay and return must be interruptible by accessibility speed settings.",
"Update the field: position, exposure, conditions, morale, hazards and card state change visibly before the next decision."
])
add_h(doc,'4.2 Position without grid fuss',2)
add_p(doc,"The battle plane uses a small number of meaningful bands rather than a freely measured grid: party rear, party front, contested center, enemy front and enemy rear. Actions can advance, pull, pin, exchange positions or create hazards between bands. Large enemies may occupy two bands. This permits authored animation and intelligible targeting while retaining enough spatial consequence for weapons to feel different. A spear can own the contested center. A rapier can exploit an exposed front. A flintlock can threaten across bands but becomes awkward when engaged.")
add_h(doc,'4.3 Resources and failure',2)
add_table(doc,['Resource','Meaning in play','Failure pressure'],[
('Vitality','Immediate capacity to remain in the encounter.','At zero, a character is downed and exposed to lasting injury rather than simply erased.'),('Composure','Ability to execute complex skills under terror, pain and alien influence.','Low composure distorts previews, increases Echo risk and can trigger authored reactions.'),('Guard','Temporary protection created by stance, cover and allies.','Break exposes the actor to stagger and interrupts.'),('Supplies','A shared expedition budget for medicine, ammunition and prepared tools.','Using the last good bandage may win now and worsen the return journey.'),('Bond charge','A situational teamwork charge earned by executing coordinated actions, not a romance meter.','Strong abilities require the relevant authored bond rank plus compatible combat circumstances.')],[1.0,2.65,2.85])

add_h(doc,'5. Signature weapons, bonds and Echoes',1)
add_p(doc,"Every heroine owns one signature weapon family. Equipment changes should modify material, reach, handling, ammunition, inscriptions and tactical properties without replacing the silhouette that makes her readable. The weapon is part of characterization: Betty's brutal boarding mace makes care look forceful; Ayla's bronze spear connects custodianship to distance and ritual geometry; Vix's saber and flintlock embody maritime adaptability; Grisha's halberd turns drilled authority into space control; Isabella's rapier makes risk and precision inseparable; Nara's staff and astrolabe make navigation an occult act.")
add_p(doc,"Each heroine unlocks seven skills across bond ranks D, C, B, A, S, SS and SSS. A rank is a named story milestone that records how far her personal arc, romance and practiced teamwork have advanced. The corresponding scene changes what the pair can attempt because they have learned a specific technique or made a specific commitment. Dialogue choices never fill a hidden affection bar.")
skills={
'Betty — boarding mace / field medicine':[('D','Guarded Strike','Strike one adjacent enemy; gain 2 Guard for the threatened ally in the same band.'),('C','Condition Cleanse','Spend one medicine charge; remove one Body status and grant one round of resistance to that status.'),('B','Rescue Charge','Move up to two bands, collect one Downed ally and place that ally in the band behind Betty.'),('A','Healing Impact','Attack one adjacent enemy; every ally behind Betty recovers Vitality equal to half the damage dealt.'),('S','Fatal Intercept','Reaction once per round: take a lethal hit for one ally and convert half the remaining damage into an Injury check after battle.'),('SS','Mobile Infirmary','Expedition capability: establish treatment at any legal camp and reduce Serious Injury recovery by one day.'),('SSS','Combat Revival','Once per expedition: return one Downed committed harem member at 40% Vitality and apply a guaranteed Serious Injury after battle.')],
'Ayla — recurved bronze spear / tomb warding':[('D','Reach Counter','Reaction: attack an enemy that enters the Contested band.'),('C','Structural Scan','Reveal Armor, active construction tags and one valid counter on a construct, fortification or armored target.'),('B','Safe Passage','Move the party through one hazardous band without triggering movement reactions.'),('A','Ward Line','Place a ward between two adjacent bands; the first enemy crossing it takes damage and becomes Staggered.'),('S','Curse Dispel','Remove one identified curse, funerary status or imposed authority effect.'),('SS','Deny Activation','Once per site: cancel one understood machine, ritual or phase activation.'),('SSS','Override Tomb Rule','Once per expedition: replace one discovered tomb rule with an approved alternate rule until the party leaves the site.')],
'Vix — saber and flintlock / corsair craft':[('D','Marking Shot','Shoot one visible enemy and apply Marked for two rounds.'),('C','Steal Advantage','Remove one beneficial status from a Marked enemy and apply it to Vix for its remaining duration.'),('B','Decoy Dash','Move up to two bands and leave a decoy that consumes the next enemy reaction aimed at her route.'),('A','Feint and Fire','Attack one adjacent target with the saber and one visible target with the pistol; choose which hit applies Marked.'),('S','Supply Cache','Expedition capability: reveal one prepared cache on a surveyed route and choose food, ammunition or medicine.'),('SS','Chain Shot','Fire through up to three Marked enemies; each hit after the first gains +2 Potency.'),('SSS','Detonate Marks','Once per expedition: remove every Marked status and deal one pistol hit to each affected enemy in initiative order.')],
'Grisha — officer’s halberd / formation command':[('D','Formation Guard','Grant 2 Guard to every ally in Grisha’s band and adjacent allied bands.'),('C','Hook Pull','Pull one adjacent enemy one band toward Grisha and apply Exposed.'),('B','Coordinated Volley','Order every ally with a loaded ranged weapon to make one basic ranged attack against the selected target.'),('A','Formation Advance','Move every conscious ally one band forward and preserve their current Guard.'),('S','Forced Duel','Compel one enemy to target Grisha with its next direct attack; interference exposes the interfering actor.'),('SS','Command Standard','Expedition capability: establish a prepared camp or battle position that grants +2 starting Guard and +1 Composure.'),('SSS','Hold the Line','Once per expedition for one full round: allies cannot be displaced, Staggered or Terrified while Grisha remains conscious.')],
'Isabella — rapier / aristocratic piracy':[('D','Opening Lunge','Attack an enemy that has taken no Vitality damage this battle; gain +4 Potency.'),('C','Riposte Stance','Reaction stance: when a direct melee attack misses Isabella, strike the attacker and apply Exposed.'),('B','Ally Swap','Exchange bands with one ally; both actors keep their activation and gain 1 Guard.'),('A','Strip Command','Remove one command, leadership or pack benefit from the target and apply Exposed for two rounds.'),('S','Risky Thrust','Declare a target and minimum damage before rolling; meeting the value doubles damage, missing it applies Exposed to Isabella.'),('SS','Command Battlefield','Expedition capability: convert one neutral terrain feature into cover, an exit or a one-use tactical asset.'),('SSS','Seize Enemy Turn','Once per expedition after stripping command: cancel the next enemy activation and give an immediate activation to one ally.')],
'Nara — ritual staff and astrolabe / occult navigation':[('D','Reveal Paths','Reveal hidden exits, invisible lanes, temporal drift and Concealed actors in the current space.'),('C','Alien Ward','Place a two-band ward that blocks Alien Marked and incorporeal movement for two rounds.'),('B','Redirect Projectile','Reaction: choose a new legal target for one visible projectile or spell.'),('A','Teleport Ally','Move one ally to any visible allied band; spend 2 Composure.'),('S','Merge Effects','Select two compatible party effects; resolve their damage, movement or status application as one command.'),('SS','Reveal Route','Expedition capability: expose one temporary route, safe interval or retreat edge in the current anomaly.'),('SSS','Anchor Return','Once per expedition: return the field party to the last committed safe anchor after catastrophic displacement.')]
}
for heroine, rows in skills.items():
    add_h(doc,heroine,2); add_table(doc,['Rank','Skill','Function'],rows,[.55,1.6,4.35])
add_h(doc,'5.1 The protagonist’s Echo abilities',2)
add_p(doc,"The male protagonist does not copy these skills. Proximity to the island's deeper intelligence causes him to form Echoes: mechanically distinct responses shaped by what he has learned from a companion. Betty teaches him to notice the moment before collapse; his Echo may defer damage into a future turn, creating a debt rather than healing it. Ayla teaches him to read funerary authority; his Echo may mark a rule for temporary inversion, but it attracts the attention of whatever enforces that rule. The relationship is legible without making his kit a superior version of hers.")
add_table(doc,['Source bond','Echo example','Difference and cost'],[
('Betty','Borrowed Tomorrow','Defers injury rather than curing it; the debt returns unless the encounter ends safely.'),('Ayla','Profane Exception','Inverts one known ward for a short interval and raises tomb alert.'),('Vix','Unpaid Favor','Produces an improbable resource now and creates a later obligation.'),('Grisha','False Command','Issues an alien-perfect order that moves enemies but damages composure.'),('Isabella','Stolen Initiative','Acts inside an enemy phase at the risk of becoming its primary target.'),('Nara','Echo of Arrival','Returns to a previous position-state while retaining one consequence that should have been undone.')],[1.15,1.65,3.7])

add_figure(doc,'betty-keyframes.png','Figure 3 — Betty’s Healing Impact animation board.','This exploratory board supplied the action now named Healing Impact. Its embedded legacy title is superseded by the functional skill name and exact mechanical specification in this document. The board fixes the action grammar: card state, emergence, planted silhouette, ampoule read, weapon preparation, impact, backward healing pulse and return.')
add_h(doc,'5.2 Animation timing contract',2)
add_p(doc,"At sixty frames per second, ordinary skills should usually resolve in forty-five to ninety frames before reactions and VFX decay. Anticipation must be long enough to read the weapon and target; hit pause should be measured in a handful of frames; recovery must communicate whether the actor remains vulnerable. The game may accelerate noncritical transitions after the player has seen them, but input, targeting and damage may never become desynchronized from the visible action.")
add_callout(doc,'Framing contract',"Every storyboard panel and shipped animation capture must contain the actor from highest hair or ear tip to lowest foot, the complete signature weapon, every carried ally or target required to understand the action, and the full VFX envelope. Keep at least eight percent safe margin on every side. Scale the action down before allowing a boot, blade, tail, prop or effect arc to touch the frame.")

add_h(doc,'5.3 Action specifications for all forty-two skills',2)
add_p(doc,"A skill name is a command label. It states the action or tactical result in ordinary language. The effect defines the game rule. The animation demonstrates that rule through anticipation, contact and recovery. Spectacle comes from force, timing, silhouette, camera and VFX; it never depends on a poetic title or an explanatory slogan. Every shot preserves the complete actor, weapon, target proxy and effect envelope inside the safe frame.")

action_specs={
'Guarded Strike':('Sweep the mace across the ally’s line, catch the blow, strike the enemy, then plant beside the ally.','One enemy contact, one ally guard icon, fixed sockets.'),
'Condition Cleanse':('Break one ampoule over the ally; cleansing vapor strips the status icon and forms a brief resistance ring.','One bottle prop, one ally socket, maskable vapor.'),
'Rescue Charge':('Drive shoulder-first across two bands, lift the downed ally onto the standardized carry point, then skid into guard.','Two-band root motion and one universal carry silhouette.'),
'Healing Impact':('Smash an ampoule against the mace crown, sweep through the enemy, then send the impact pulse backward through allied cards.','Separate strike and healing VFX; no simultaneous ally bodies.'),
'Fatal Intercept':('Enter on the lethal tell, catch the attack across mace and bracer, drop to one knee, then cover the ally.','Universal intercept lane and standardized attacker proxy.'),
'Mobile Infirmary':('At camp, drive three treatment beacons into the ground, link their perimeter, and open the field kit.','Three reusable props, camp-only scene state.'),
'Combat Revival':('Pin the mace upright, brace the downed ally, deliver the revival shock, then show the injury warning before recovery.','One kneeling pair rig and one downed socket per body class.'),
'Reach Counter':('Lower the spear across the body; thrust the instant the enemy crosses the contested-band line.','Single contact frame and short root motion.'),
'Structural Scan':('Tap the target with the spear butt; bronze rings map armor plates and illuminate one fault.','Target sockets expose armor tags and one fault marker.'),
'Safe Passage':('Spin the spear to clear the hazardous line, vault the band boundary, and point the party through.','Allies travel as card trails; one hazard spline.'),
'Ward Line':('Drag the spear tip between two bands, ignite the line, then hold the spear vertical as the trap arms.','One ground spline and one crossing trigger.'),
'Curse Dispel':('Hook the identified curse core, pull it clear of the ally, and pin it under the spear point until it breaks.','All dispellable effects expose one hookable core.'),
'Deny Activation':('Wedge the spear into the mechanism sigil, slide under the force, and stamp the cancel seal.','Standardized activation wheel replaces room-specific machinery.'),
'Override Tomb Rule':('Trace the approved replacement seal, strike the floor once, and rotate the rule glyph from old state to new state.','One rule-glyph rig driven by validated alternate-rule data.'),
'Marking Shot':('Fire from a loose stance; let recoil turn the pistol arm into saber guard as the mark closes on the target.','One projectile socket and one two-round mark decal.'),
'Steal Advantage':('Cut the benefit icon from the marked enemy, catch it on a ledger charm, and snap it onto Vix.','Beneficial statuses expose standardized stealable emblems.'),
'Decoy Dash':('Dash beneath the attack line, leave a tinted freeze-pose at origin, and stop with pistol leveled.','One two-band dash and one static decoy instance.'),
'Feint and Fire':('Commit the saber feint at the adjacent target, turn through the recoil, and fire at the visible target.','Two fixed target sockets; mark choice resolves at command commit.'),
'Supply Cache':('Stamp the surveyed-route seal, open a compact cache hatch, and take the chosen supply bundle.','Route-only interaction with three authored supply props.'),
'Chain Shot':('Sight the marked line, fire through three targets, and show the potency step on each successive impact.','Maximum three travel beats; extra hits use projectile echoes.'),
'Detonate Marks':('Raise the pistol, lock every visible mark, then fire once as the marks discharge in initiative order.','Body holds one firing pose; ordered effects carry the scale.'),
'Formation Guard':('Brace the halberd horizontally, turn the hips into the impact, and drive the allied line into guarded stance.','Allies remain card silhouettes; guard applies by band query.'),
'Hook Pull':('Feint high, drop the rear hook behind the target’s leg or guard, then pivot and pull one band.','Body-class hook sockets and three target reactions.'),
'Coordinated Volley':('Raise the halberd to sighting angle, pause for loaded allies, then chop downward to release the volley.','Allied shots launch from card muzzle sockets.'),
'Formation Advance':('Lock the halberd crosswise and take one heavy step as every allied card advances one band with Guard intact.','One synchronized band tween; no full ally locomotion.'),
'Forced Duel':('Salute, point the blade at the chosen enemy, then catch its response in a centered clash.','Chosen target receives forced-target icon; bosses use an impact proxy.'),
'Command Standard':('Drive the halberd butt into the camp ground and unfold the command banner as formation marks appear.','One banner flipbook and camp-position decal.'),
'Hold the Line':('Plant both boots and walk a visible pressure wall backward while allied cards lock behind the halberd.','Pressure-wall VFX replaces multiple enemy bodies.'),
'Opening Lunge':('Salute, cross the band in one long lunge, strike, and recover before the coat finishes moving.','Classic anticipation-contact-recovery with overscan.'),
'Riposte Stance':('Open the guard, slip the missed melee attack through the fixed lane, then cut low across the attacker.','One stance loop, one miss lane, one counter contact.'),
'Ally Swap':('Take the ally’s offered forearm, pivot around the shared center, and finish in each other’s former band.','Universal partner socket plus a no-contact variant.'),
'Strip Command':('Catch the standardized command emblem on the rapier, circle once, cut it free, and drop it.','Leadership effects expose one removable emblem.'),
'Risky Thrust':('Sight the declared damage line, commit to full extension, show success at contact or Exposed on the strained recovery.','One lunge with two outcome recoveries.'),
'Command Battlefield':('Point to one validated neutral feature; coat sweep and rapier mark convert it into the selected tactical asset.','Feature data supplies cover, exit or asset state; no spawned crowd.'),
'Seize Enemy Turn':('Tap the stripped enemy emblem, freeze the enemy activation, then point to the ally whose card expands immediately.','Enemy timeline cancel plus standard ally expansion transition.'),
'Reveal Paths':('Open the astrolabe, align it with the staff rings, and project every valid lane and concealed actor boundary.','Space graph supplies exits, lanes, drift and reveal sockets.'),
'Alien Ward':('Cast salt in a low arc, sweep the staff through it, and close a ward across two bands.','One ground decal with two-round occupancy rule.'),
'Redirect Projectile':('Catch the visible projectile inside the staff rings, rotate the astrolabe, and release it along the chosen legal line.','Projectile converts to a standardized captured core before retargeting.'),
'Teleport Ally':('Cut a narrow fold, reach through, take the ally’s forearm, and pull them into the selected allied band.','One fold plane and universal partner socket.'),
'Merge Effects':('Draw one effect ring with the staff, one with the astrolabe, overlap them, then release the validated combined result.','Compatibility table selects one merged payload and composite VFX.'),
'Reveal Route':('Sweep the staff across the local chart, raise the temporary route into perspective, and pin its safe interval.','Anomaly data provides one route ribbon or retreat edge.'),
'Anchor Return':('Plant the staff, raise the astrolabe, close the coordinate dome, then cut directly to the last committed safe anchor.','Save-backed anchor transition; fixed party silhouettes and full-screen VFX.')
}
for heroine, rows in skills.items():
    short_name=heroine.split(' — ')[0]
    action_rows=[]
    for rank, skill, effect in rows:
        animation, implementation=action_specs[skill]
        action_rows.append((rank,skill,effect,animation,implementation))
    add_h(doc,short_name+' — seven-skill action specification',2)
    add_table(doc,['Rank','Skill','Exact effect','Animation','Implementation'],action_rows,[.4,1.15,2.0,1.8,1.15])
add_h(doc,'5.4 Multi-pass skill design before lock',2)
add_p(doc,"A skill does not become final because its first description is coherent or its first image is exciting. Each kit should circulate through the following passes, with a written change log that preserves what problem the revision was meant to solve. The team may run several passes in parallel, but the lock review happens only after the move survives all of them in combination.")
add_numbered(doc,[
"Character fantasy pass: ask whether the move expresses this woman’s profession, culture, temperament, weapon and relationship to risk. Remove generic RPG verbs wearing her costume.",
"Mechanical pass: test the move inside the five-band battle model. It needs a clear decision, counterplay, cost, target rule and reason to coexist with the other six skills.",
"Choreography pass: reduce the action to anticipation, hero/contact image and recovery. A viewer who has not read the tooltip should correctly guess its broad purpose.",
"Animation feasibility pass: inventory bodies, props, sockets, root motion, cloth, target reactions and VFX. Redesign any move whose spectacle relies on arbitrary geometry, uncontrolled crowds or character-specific grapples that cannot be standardized.",
"Consistency pass: compare face, body, costume, weapon dimensions, handedness, prop placement and performance vocabulary against the heroine’s identity package at matching scale.",
"Framing pass: test the union of body, hair, ears, tail, weapon, partner, target proxy and full VFX bounds. Add eight percent safe space. A crop is a failed board, not a stylistic option.",
"Encounter pass: place the move in ordinary, elite, boss, narrow-room, wide-field and hazard encounters. Look for dominant loops, unreadable target states and effects that become tedious on repeat.",
"Bond and narrative pass: ask why this capability becomes possible at its rank and what changed between the characters. Rename or relocate skills whose emotional meaning is arbitrary.",
"Production sample pass: animate a rough timing test at game scale with temporary VFX and the actual UI. Judge readability from the player’s camera, not from a full-screen art board.",
"Lock review: approve mechanics, choreography, identity, framing, content dependencies and performance budget together. A later discovery may still reopen the move, but no discipline silently treats an exploratory plate as final."
])
add_callout(doc,'Status of the current boards',"The existing reference images are samples used to discover direction and failure modes. Crowded seven-row sheets that crop bodies, weapons or effects are rejected as production boards even when individual poses are useful. They may inform the next pass, but they do not define final character models or final animation.")

add_h(doc,'6. The household and adult haremlit progression',1)
add_p(doc,"Haremlit is a core product feature. The shipped format is MFFF+: one male protagonist and a growing household of adult women. The protagonist loves and has sex with the heroines. The heroines love and have sex with him. The women may also flirt with, court, love and have sex with one another inside the harem. Those female-female relationships strengthen the household and do not break the male-female romance promise.")
add_h(doc,'6.1 The stable harem',2)
add_p(doc,"The player invests in specific heroines as permanent romantic partners. The game protects that investment. A committed heroine never cheats, leaves the protagonist for another partner, becomes another man’s lover, or reveals that her commitment was temporary. The romance story is about being in love and happy. Personality differences create jokes, tactical arguments, affectionate competition and different ways of showing love. They do not create a recurring cycle of romantic damage and repair.")
add_table(doc,['Haremlit rule','Exact shipped rule','Content consequence'],[
('Format','MFFF+: one male protagonist plus at least three adult female partners, expanding through the campaign.','The six core heroines all have permanent protagonist romances and household membership arcs.'),
('Allowed relationships','Male-female inside the harem; female-female inside the harem.','Heroine pair scenes may include friendship, flirtation, courtship, sex and love. Female-female content is fanservice and household development.'),
('Outside men','No heroine has romance, sex or private longing directed toward a man outside the harem.','Outside men remain social, familial, professional, political or hostile characters.'),
('Outside women','Romantic or sexual tension with a compatible woman points toward recruitment.','The protagonist or an existing heroine may flirt, court or seduce her only through an authored path that can bring her into the harem.'),
('Joining','A woman joins after her named romance and move-in milestone.','household_member changes false to true once and never returns to false in the shipped campaign.'),
('Permanence','Once in, she is locked in as a member and romantic partner.','No breakup ending, betrayal romance, outside-partner route or post-credits replacement.'),
('Cheating','Excluded.','No affair, lapse, false-infidelity plot or jealousy trap. A committed heroine never becomes a romantic or sexual option for an outside man.'),
('Futanari content','Excluded from the erotic-content specification.','Character identity packages and adult scene packages do not contain futanari variants.'),
('Fanservice','Expected and frequent.','Cards, costumes, banter, household blocking, victory poses, bathing, flirting and adult scenes may emphasize beauty and sexual chemistry while preserving character identity and combat readability.')],[1.15,2.45,2.9])
add_callout(doc,'Why these rules exist',"Players choose favorite heroines, spend time on their stories and treat them as waifus. The game rewards that attachment with loyalty, affection, sex, admiration, fanservice, domestic happiness and stronger teamwork. It never punishes attachment with cheating, replacement, a breakup route or a surprise change to the relationship premise.")
add_h(doc,'6.1.1 Hero fantasy and gentleman rule',3)
add_p(doc,"The protagonist’s fantasy is strength with direction. He becomes capable enough to choose his own course on an island that tries to dictate everyone’s life. He protects people, keeps his word, faces dangers others avoid and builds a home worth joining. The heroines watch him act. They judge him as a hero. They admire his competence, courage, generosity and command presence. Their love and desire confirm that he has become the man he chose to become.")
add_p(doc,"Command authority applies to game decisions: route, preparation, tactical orders, estate priorities and the risks he personally accepts. It does not give him ownership of women. He behaves as a gentleman. He courts rather than coerces. He listens when a woman speaks. He does not force romance, sex, clothing, household roles or recruitment. The heroines volunteer because they want the adventure, the household, one another and him. Their willingness should be visible in what they initiate: a heroine proposes an expedition, requests a date, starts a kiss, chooses a costume, joins another woman’s flirtation, asks for a room, offers a skill or brings a prospective member home.")
add_callout(doc,'Romance tone',"Do not write the harem as a case study in relationship failure. Write attractive, capable adult women having an excellent time with a heroic man they respect. Use danger, villains, mysteries, ambition and tactical disagreement for conflict. Use romance for desire, humor, tenderness, admiration, fanservice, sex, domestic pleasure and victory together.")
add_h(doc,'6.1.2 Recruitment direction',3)
add_p(doc,"Every romantic beat with a woman outside the household has one declared destination: recruitment. The content record names the prospective heroine, the current recruiter or recruiters, the next recruitment milestone and the condition that advances it. Flirting cannot exist as an unrelated collectible scene. A woman who will never be recruitable remains a friend, rival, client or faction figure without romantic framing.")
add_numbered(doc,[
"Attraction: the protagonist or a heroine recognizes mutual interest. The woman receives recruitment_focus_id and romantic framing may begin.",
"Courtship: adventure, dates, flirting and sex establish that she wants the protagonist, the women or both. Existing heroines may lead or join.",
"Join: she chooses the household. household_member becomes true permanently. Her room, protagonist romance and heroine-pair content activate."
])
add_h(doc,'6.2 Authored milestones instead of relationship meters',2)
add_p(doc,"Each protagonist–heroine romance and each supported heroine pair has a short ordered list of named scenes. The current milestone ID is the state. A scene may remember specific choices through scene flags, yet it does not add points to Attraction, Trust, Intimacy or Conflict. Writers decide what changes because of the event; the runtime records that exact change.")
add_table(doc,['Arc type','Required package content','Runtime state'],[
('Protagonist–heroine romance','Six mandatory turns: attraction, courtship, first intimacy, heroic admiration, move-in and SSS capstone.','Current milestone ID, scene flags, household_member and bond rank.'),
('Heroine pair','One defining scene for all fifteen heroine pairs; romantic or sexual follow-through for at least six supported pairs.','Current pair milestone ID and pair-specific flags.'),
('Recruitment','The scenes in which existing members notice, court, seduce and bring the prospective member into the household.','Recruitment focus ID, completed scene flags and household_member.'),
('Adult scene','One authored scene package with fixed participants and only the variants required by its story context.','Completion flag and any explicitly authored aftermath flags; no generic sexual-state simulator.')],[1.3,3.65,1.55])
add_callout(doc,'Authoring rule',"Write characters who visibly want one another. The game does not need a legal apparatus to prove its own romance premise.")
add_h(doc,'6.3 Minimum haremlit presence',2)
add_p(doc,"The harem cannot disappear for ten hours while the adventure plot runs. Romance and fanservice occupy scheduled space in the core loop. They appear in field banter, card presentation, battle assists, camp scenes, estate behavior, dates, recruitment and adult scenes. The player should regularly see that the women admire the protagonist, desire him, enjoy one another and prefer the household they are building.")
add_table(doc,['Content unit','Minimum shipped amount','Required job'],[
('Each heroine romance','Six mandatory turns: attraction, courtship, first intimacy, admiration after a heroic act, move-in and SSS capstone.','Shows why she wants him, what she initiates and how joining improves her life.'),
('Male-female adult scenes','At least two authored scenes per core heroine in the adult-content build.','Delivers the protagonist-heroine sexual promise without replacing character or plot progression.'),
('Heroine-pair content','Each of the fifteen pairs receives one defining relationship scene; at least six supported pairs receive romantic or sexual follow-through.','Makes female-female chemistry visible inside the household instead of treating it as an off-screen claim.'),
('Group household scenes','At least one new group scene per campaign act plus a finale celebration.','Shows the harem functioning as a happy household rather than six isolated routes.'),
('Estate return beat','After every major expedition, schedule one short affection, admiration, fanservice or domestic-comedy beat before the next departure.','Keeps romance present in the repeating game loop.'),
('Heroine initiation','Every heroine initiates dates, touch, kisses, sex, costume play, recruitment or private time in multiple scenes.','Proves that participation is voluntary and desired.'),
('Battle relationship display','Every heroine has one protagonist assist, one heroine-pair assist and one victory or recovery interaction.','Connects love and admiration to the card-to-active combat spectacle.')],[1.35,2.3,2.85])
add_callout(doc,'Haremlit acceptance test',"Play any consecutive ninety-minute section after the full harem begins. It must contain visible attraction or affection, one heroine-initiated beat and one reminder that the women relate to one another. If all three are absent, the segment has lost the product premise and must be revised.")

add_h(doc,'7. The six heroines',1)
heroines=[
('Betty','Betty served as surgeon’s mate aboard the privateer brig Wren. During the Saint Orra quarantine she ignored her captain’s order to seal the sick below deck, took control of the quarterdeck and organized treatment before the fever reached the harbor. The decision saved most of the crew and ended her naval career. She now carries her mentor’s brass ampoule rack beside a boarding mace built for cramped decks. Warm humor is how she keeps frightened people responsive; clipped commands take over when someone begins to bleed. She wants a household where care runs in every direction. Her romance advances when she accepts treatment, rest and protection from people she trusts, then chooses to help those same women court and welcome the next compatible member.'),
('Ayla','Ayla is a jungle-elf warden from the Kesh River community, trained to maintain the surviving funeral roads, water seals and lineage doors around Veyra Tidehouse. She broke a prohibited seal to rescue three children during a flood, saved them, and exposed a buried containment channel that her elders had concealed for political reasons. The council kept her in service while removing her authority to train apprentices. She knows rites, animal pressure, bronze mechanisms and the exact limits of inherited knowledge. She wants to build a living custodial school whose students can question a rule before they inherit it. Attraction grows through shared field judgment, respect for boundaries and the household’s willingness to become responsible custodians together.'),
('Vix','Vix was a factor and corsair for House Sairin, a fox-folk maritime house whose credit reaches farther than its guns. She diverted an insured spice convoy to evacuate dockworkers during the Kairo warehouse fire, saving hundreds while leaving her house exposed to claims it cannot publicly honor. Her cousin seized her ship under a lawful debt instrument, which Vix intends to overturn without destroying the family that taught her everything useful. Her saber, flintlock, marks and caches turn preparation into tempo. She flirts as openly as she negotiates, delights in pair chemistry inside the household, and often becomes the first member to recognize a woman who might want to join them.'),
('Grisha','Grisha commanded the Third Sapper Company at Daran Ford. Ordered to shell a riverside settlement occupied by raiders and civilians together, she moved her guns into direct-fire range, broke the raider barricade and evacuated the residents under cover. Her officers won the ground; the governor court-martialed her for disobedience and made the record disappear. Academy training gave her engineering, logistics, law and the halberd drill used to keep frightened soldiers aligned. She wants lawful service whose authority can survive scrutiny. Within the household she offers dependable structure, learns to share command, and courts women through acts of preparation that make their ambitions possible.'),
('Isabella','Isabella de Alvarado inherited a ruined harbor title, the fast sloop Belladonna and a commission signed by a governor who had already been deposed. She kept all three useful by controlling who could dock, which flag the ship flew and who received credit for each victory. During the San Telmo mutiny she crossed the quarterdeck alone, disarmed the gun captain and offered the crew better articles before the officers recovered from surprise. Her rapier kit turns initiative, position and enemy command into weapons. Performance is her chosen social craft: she enjoys entrances, teasing, ceremony and the pleasure of making a household feel grand. Romance deepens as she assigns private meanings to those performances, shares authority with the other women and helps stage courtships that make prospective members feel desired by the whole house.'),
('Nara','Nara was born near Cape Meridian Light and apprenticed to navigator Mara Venn, who taught her to compare stars, tides, oral routes and the behavior of frightened animals before trusting an instrument. A temporal storm returned their survey boat to shore eleven days before it departed. Mara vanished on the second arrival; Nara kept both contradictory logbooks and has mapped coastlines that appear only under specific weather. Her staff and astrolabe reveal paths, redirect trajectories and secure returns. She wants the island studied by people who accept responsibility for what attention can awaken. She brings prospective members on honest trial journeys, watches how they treat fear, and offers commitment after they have seen the household under pressure.')]
for name,text in heroines:
    add_h(doc,name,2); hp=add_p(doc,text); hp.paragraph_format.keep_together=True
add_figure(doc,'heroine-roster.png','Figure 4 — heroine silhouette and card system.','This sheet establishes six instantly separable silhouettes, signature weapons, accent colors and card identities. It is a direction board, not final model art: future turnarounds must standardize anatomy, costume construction, weapon dimensions and expression libraries before animation production.')
add_h(doc,'7.1 Heroine action files',2)
add_p(doc,"These records control character writing, combat animation, recruitment scenes and household behavior. Every scene must reinforce at least one listed capability, objective or relationship behavior. Do not replace these facts with generalized attitude.")
add_table(doc,['Heroine','Field role and weapon','Combat job','Personal objective','Harem action'],[
('Betty','Combat surgeon; boarding mace and brass medicine rack.','Protects exposed allies, removes conditions, rescues downed partners and turns impact into healing.','Build an infirmary where the healer also receives care.','Treats the protagonist as the hero who makes rescue possible. Initiates physical affection after danger. Helps nervous women feel safe enough to join.'),
('Ayla','Jungle-elf tomb warden; recurved bronze spear.','Controls crossings, identifies structures, blocks hazards and overrides understood tomb rules.','Create a living school that can maintain elven infrastructure without repeating inherited mistakes.','Respects demonstrated courage and responsible command. Volunteers as guide, teacher and lover. Tests prospective members in real field conditions.'),
('Vix','Fox-folk corsair and factor; saber and flintlock.','Marks targets, steals advantages, creates decoys, fires chained shots and prepares caches.','Recover her ship and defeat the debt claim without destroying House Sairin.','Flirts first, engineers dates, creates sexual opportunities inside the household and actively identifies women who would enjoy joining.'),
('Grisha','Disgraced sapper officer; officer’s halberd.','Builds Guard, moves formations, pulls targets, orders volleys and holds ground.','Establish a force whose authority protects civilians and survives public scrutiny.','Admires the protagonist’s honorable command. Shows love through preparation, protection and direct offers of service. Helps new women turn ambitions into plans.'),
('Isabella','Pirate aristocrat and harbor power; rapier.','Wins initiative, ripostes, swaps allies, strips command and steals enemy activations.','Make her ruined title real by building a household, fleet and harbor network people choose to follow.','Makes desire public and glamorous. Stages entrances, clothing, parties and courtships. Teaches prospective members that joining means being wanted by the whole house.'),
('Nara','Occult navigator; ritual staff and astrolabe.','Reveals paths, wards alien movement, redirects attacks, teleports allies and secures retreat.','Map the island without giving the cosmic intelligence a path into ordinary life.','Admiration grows when the protagonist accepts hard evidence and still acts decisively. She proposes private journeys, honest intimacy and final commitment after shared danger.')],[.7,1.25,1.45,1.5,1.6])
add_h(doc,'7.2 Party relationships beyond the protagonist',2)
add_p(doc,"Companion-to-companion states should be sparse enough to author and rich enough to matter. Each pair needs a baseline opinion, two or three pressure points, a small number of turning scenes, and ambient behaviors that reveal change. Betty and Grisha may initially respect one another's crisis discipline while fighting about acceptable casualties. Ayla may see Nara's measurements as a dangerous way of turning sacred limits into routes; later, each becomes the only person who understands the other's responsibility. Vix and Isabella can enjoy one another's games until a debt places the estate at risk. These relationships alter banter, assist animations, room use, expedition objections and a few combined skills.")

add_h(doc,'8. The estate: a home that becomes true',1)
add_p(doc,"The hub begins as a damaged coastal estate with two habitable rooms, a poor well, a leaking roof, jammed doors, rats, one reliable lantern and a veranda that remains beautiful enough to make the hardship feel almost insulting. It becomes home by accumulating use. Betty claims an infirmary because treatment is otherwise happening on the dining table. Ayla establishes a clean place for funerary objects so trophies stop being piled beside cookware. Vix improves the dock and quietly creates storage that customs inspectors would not find. Grisha repairs drainage and sight lines. Isabella makes one room suitable for receiving people whose vanity is politically useful. Nara marks the parts of the property that remain in the same time at night.")
add_p(doc,"Upgrades must be visible and behavioral. A library is not a menu unlock floating outside the fiction; it is shelves, tables, lamps, cataloguing work and companions choosing to read. A garden changes meals and puts people outdoors. Recovered artifacts appear physically, sometimes attracting comment or trouble. Rooms develop personal objects. After an expedition, the player may find two companions talking on the veranda, someone asleep over a book, someone cleaning a weapon, or someone crying where she expected privacy. The house earns emotional meaning because it hosts ordinary life between catastrophes.")
add_figure(doc,'home-estate.png','Figure 5 — the estate as an inhabited systems screen.','The scene demonstrates readable work zones, partial restoration, multiple concurrent companion behaviors, maritime access and recovered artifacts. Upgrades should preserve traces of the ruin so improvement feels accumulated rather than swapped for a pristine replacement.')
add_h(doc,'8.1 Upgrade families',2)
add_table(doc,['Space','System unlocked or improved','Visible life'],[
('Infirmary','Injury treatment, medicine crafting, recovery planning.','Bandaging, sterilizing, sleeping, difficult diagnoses.'),('Library and chart room','Research, rumor comparison, tomb translation, route forecasting.','Reading, arguing over maps, pinning notes, late-night study.'),('Workshop and powder room','Weapon modifications, repairs, ammunition and safe relic handling.','Cleaning, filing, testing mechanisms, controlled mishaps.'),('Kitchen and garden','Expedition meals, morale, medicinal plants and household events.','Cooking, harvesting, tea, shared meals and small celebrations.'),('Dock and boatshed','Sea routes, fishing, smuggling, evacuation and vessel upgrades.','Loading, repair, arrivals, private conversations at the water.'),('Private rooms and veranda','Intimacy, comfort, dates, letters and personal activities.','Personal decoration, bathing, dressing, flirtation, sex, cuddling and rest.')],[1.3,2.45,2.75])

add_h(doc,'9. Exploration sites and elven tomb infrastructure',1)
add_p(doc,"The ancient tombs formed a regional civic system for moving bodies, recording lineage, maintaining wards, distributing water, controlling air, conducting rites and keeping dangerous remains or knowledge in specific relationships to the living city. This infrastructure determines each layout. A processional road implies receiving courts, drainage, storage and staff circulation. Sealed bronze doors require manufacturing, replacement parts and authority. Star shafts affect ritual timing and ventilation. Maintenance gantries create alternate traversal routes. Ossuary districts preserve the logistics of receiving, preparing, recording and placing the dead.")
add_p(doc,"The collapse left parts of the system operating without the civic institutions that interpreted it. Jungle communities learned local safeties and prohibitions. Colonial excavators cut through them. The alien intelligence can exploit routines that were once protective. A dungeon therefore has three readable layers: its original civil function, the adaptations of later inhabitants, and the present disturbance caused by looting, ecology, temporal weather or the Champion.")
add_figure(doc,'elven-tomb.png','Figure 6 — funerary infrastructure as a playable side-view site.','The sloped processional route, waterworks, maintenance levels, sealed bronze mechanisms and deep ritual chamber create multiple traversal bands. The environment remains legible enough for exploration and combat while communicating that the tomb was once operated by a civilization, not assembled as a sequence of puzzle rooms.')
add_h(doc,'9.1 Tomb grammar',2)
add_bullets(doc,[
"Reception: thresholds establish legal and ritual identity. Outsiders can enter physically while remaining unrecognized by the system.",
"Procession: long sloped routes manage movement, grief, status and environmental transition. They are natural spaces for patrols and traps with historical purpose.",
"Preparation: water, oils, metal fittings, record tablets and specialist tools reveal the labor behind burial.",
"Interment: lineage, rank and dangerous properties determine placement. Spatial hierarchy becomes a clue.",
"Maintenance: staff corridors, drainage, counterweights, ventilation and repair access create the practical alternate routes modern delvers exploit.",
"Containment: some dead, objects and ideas were not honored but isolated. These regions should feel architecturally different before the threat is shown.",
"Return: exits, purification and memory rites matter. A tomb designed only for descent is not a social institution."
])
add_h(doc,'9.2 Major-site concept longlist',2)
add_p(doc,"These twelve concepts define the breadth of histories and site rules the island may eventually support. They are not a twelve-dungeon first-release commitment. The current production hypothesis funds six major multi-visit sites; concepts graduate from this longlist only after the complete-loop tomb establishes measured authoring, art, implementation and testing costs.")
sites=[
('Veyra Tidehouse','A half-flooded elven tidal instrument that forecasts temporal weather by changing the water level inside its calibrated chambers.'),('Bellamy House','A colonial manor whose rooms currently connect incompatible periods in the Bellamy family record.'),('Fort Calder','An occupied headland fort shown on several imperial charts despite the absence of any quarry, payroll or construction account.'),('Pallidwood Basin','A mineral-blanched forest where color, scent and sound return under different temporal conditions.'),('Cathedral of Saint Orra','A mission and burial complex built over an elven funerary outlet; low tide exposes rival congregations of the dead.'),('Hotel Bellevue','A resort displaced from 1927 whose staff continue preparing for an opening week that never reaches its final day.'),('Mount Kestrel Glass Quarry','A volcanic quarry where tools age rapidly and transparent stone preserves moments as physical inclusions.'),('Talar Lower City','An elven civic district whose inhabitants have maintained one day of emergency law for centuries.'),('Camp 309','A settlement founded by survivors of Flight 309, creating conflicts over technology, disease, citizenship and rescue.'),('Saint Lysa Abbey','A fortified abbey whose musicians maintain the sequence that keeps a buried predator dormant.'),('Ilyon Necropolis','A monumental cemetery whose maintenance system classifies living settlements as unauthorized construction.'),('Cape Meridian Light','A lighthouse built over a maritime anchor capable of stabilizing the island or guiding the cosmic intelligence into local time.')]
add_table(doc,['Site','Rule that makes it more than a tileset'],sites,[1.9,4.6])

add_h(doc,'10. The cosmic intelligence and its Champion',1)
add_p(doc,"Behind the island's temporal behavior is an intelligence larger than a villain with a face. Its categories disregard human survival. It observes relationships between histories, places and living choices, then tests arrangements as a scholar might test a mechanism. The ancient elves encountered it, built systems to negotiate or contain its attention, and eventually suffered a collapse whose moral facts remain disputed. Some tried to use it. Others believed the island was an instrument for sheltering realities from something worse.")
add_p(doc,"The intelligence adapts through a Champion: a local body, institution or composite agent capable of learning the party's habits. The Champion should not simply gain numerical resistance to whatever attack the player used most. It studies dependencies. If the party always relies on Betty to reverse collapse, it creates encounters that separate treatment from safety. If Ayla repeatedly rewrites tomb rules, it begins presenting false authority structures. If the protagonist abuses Echoes, it offers situations in which the mechanically tempting Echo also teaches the intelligence a dangerous new relationship.")
add_h(doc,'10.1 Adaptation model',2)
add_table(doc,['Observed pattern','Champion response','Counterplay'],[
('Repeated damage type or formation','Prepares armor, terrain or escorts that make the familiar line inefficient.','Broaden kits, use environmental interactions, accept retreat and return with new knowledge.'),('Dependence on one heroine','Targets access, trust or positioning rather than simply attacking her hit points.','Develop pair synergies and household preparation; protect capability through relationships.'),('Frequent Echo use','Introduces false opportunities and increases composure pressure.','Use ordinary human techniques, research the offered anomaly, or refuse the apparent shortcut.'),('Loot-first tomb behavior','Moves valuable objects into containment paths and wakes custodial systems.','Restore functions, negotiate custodianship, or enter for a non-extractive purpose.'),('Political alliance pattern','Uses rival institutions, rumors and hostages to make combat success socially expensive.','Maintain cross-faction relationships and expose the manipulation before escalation.')],[1.55,2.55,2.4])
add_callout(doc,'Fairness rule',"Adaptation must be foreshadowed, logged as evidence and reversible through play. The Champion is frightening because it learns, not because the designer silently invalidates the player’s build.")

add_h(doc,'11. Quests, factions and consequences',1)
add_p(doc,"Quests should begin as pressures in the world rather than as errands distributed by punctuation marks. A powder shipment has not arrived, which means the orc garrison is rationing ammunition and a pirate crew suddenly has too much. An elven community closes a route because excavators broke a water seal. A fox-folk house offers credit for a salvage claim whose legal owner will arrive from another century in three days. The player can discover these situations through conversation, travel changes, prices, household visitors and expedition evidence.")
add_p(doc,"Resolution is rarely a menu of abstract moral colors. The party can negotiate, expose evidence, steal, fight, repair infrastructure, choose a claimant, create a temporary compromise or leave. The game records concrete consequences: who controls a harbor, whether a road is patrolled, which community trusts the estate, what rumor people repeat, and which companion believes the protagonist understood what was at stake.")
add_h(doc,'11.1 Faction state',2)
add_bullets(doc,["Standing measures formal permission and public reputation.","Debt records concrete obligations, favors and material claims.","Fear records deterrence and the expectation of violence.","Knowledge records what a faction believes the protagonist knows about it.","Internal alignment records which subfaction currently benefits from the relationship."])

add_h(doc,'12. Economy, equipment and expedition preparation',1)
add_p(doc,"The economy should make expeditions feel supplied without turning the player into a warehouse clerk. The estate stores bulk goods. Before departure, the player chooses a bounded load plan: medicine, ammunition, food, tools, ritual materials and room for salvage. Weight matters at the category level, and companions comment on obviously reckless omissions. Rare consumables remain precious because their sources and alternatives are legible, not because the interface hides them in a pile of nearly identical icons.")
add_h(doc,'12.1 Equipment philosophy',2)
add_p(doc,"Signature weapons stay with their owners. Progress comes through named modifications with visible construction: a new mace head, spear fitting, powder lock, halberd hook, rapier guard or astrolabe ring. Armor and clothing similarly preserve silhouette. Items may be wondrous and narrow. A bell that repels one carrion species, boots that remain dry only in funerary water, or a pistol that fires accurately at a target who has broken a sworn contract can become memorable because the journal preserves their rules and encounters respect them.")

add_h(doc,'13. Visual direction and asset grammar',1)
add_p(doc,"The visual target is painterly 2D anime with the confidence of a fighting game and the environmental richness of a prestige illustrated RPG. Confidence here means scale, clarity and commitment. Characters are large enough that posture reads. Weapons have consistent dimensions. Effects are built around silhouettes rather than poured over them. Backgrounds have atmosphere and material detail, but the battle and exploration planes remain navigable. UI frames feel theatrical in blackened bronze, antique gold, dark leather and aged vellum without becoming a filigree contest.")
add_h(doc,'13.1 Layering standard',2)
add_numbered(doc,["Far background establishes weather, horizon, architecture and region with minimal high-frequency contrast.","Mid-background carries animated atmosphere: cloud shadow, distant rain, surf, smoke, birds and temporal disturbances.","Playable plane keeps collision edges, interactables and feet readable through value control and subtle grounding shadows.","Character layer uses stable scale, rim logic and contact lighting shared across the cast.","VFX layer separates anticipation, contact, travel, status and decay so timing can be revised without repainting the actor.","UI layer frames information at the edges and yields the center to bodies, targets and action arcs."])
add_h(doc,'13.2 Character library',2)
add_p(doc,"The production unit is a reusable character library rather than a pile of scenes. Every core heroine needs a turnaround, construction sheet, expression grammar, hand and prop standards, weapon measurements, card portrait, battle idle, guarded idle, locomotion between bands, hit reactions, downed state, seven skill sequences, assists, victory and retreat. The household library then adds sitting, reading, eating, sleeping, cleaning, changing a bandage, working at a desk, gardening, leaning, arguing, laughing, crying, embracing and paired blocking marks. Shared names make planning possible; distinct performance keeps the cast human.")
add_table(doc,['Library tier','Minimum contents','Acceptance standard'],[
('Identity','Turnaround, face sheet, palette, weapon and card.','Recognizable in silhouette and grayscale; no costume part changes between views.'),('Battle core','Idle, guard, band movement, basic attack, hit, downed, item, compress/expand.','Feet and weapon origins align; every contact frame has a readable silhouette.'),('Signature','Seven abilities plus ultimate variants and assists.','Consistent timing annotations; VFX separated; target and root-motion data documented.'),('Household','Twenty shared behaviors plus six personal behaviors.','Works at common furniture marks; gaze, hands and props align.'),('Conversation','Listening, speaking intensities, disagreement, emotional openness and intimate performance.','Transitions do not require teleporting props or costume pieces.')],[1.05,3.0,2.45])
add_h(doc,'13.3 Character consistency contract',2)
add_p(doc,"Character consistency is not achieved by asking every artist or generator to remember a beautiful reference image. Each heroine requires a controlled identity package with measurements and exclusions. The package fixes standing height in battle pixels, head-to-body ratio, shoulder and hip landmarks, face geometry, skin and hair values, costume layers, handedness, weapon dimensions, prop attachment points, default light direction and the maximum safe VFX envelope. A new action may exaggerate pose and perspective, but it may not casually redesign the woman performing it.")
add_table(doc,['Control','Required record','Failure that blocks approval'],[
('Body model','Front, side and three-quarter construction; joint landmarks; hand and foot scale; species traits.','Face, height, limb length, musculature, ears, tail or skin tone changes between skills.'),('Costume model','Exploded layers, closures, seams, material notes, palette values and damage states.','Missing belts, swapped coat lengths, invented armor, unexplained neckline or boot changes.'),('Weapon model','Orthographic view, absolute length against body, grip zones, handedness, contact and VFX origins.','Blade or haft changes size; hands teleport; muzzle, spear tip or mace head origin drifts.'),('Prop model','Satchel, bottles, astrolabes, charms, flags and medical beacons with attachment sockets.','Props appear from nowhere without a readable retrieval beat or change size by shot.'),('Performance model','Neutral posture, line of action, weight distribution, signature gesture and forbidden generic poses.','Every heroine shares the same stance or loses cultural and professional specificity.'),('Frame envelope','Eight percent overscan beyond body, weapon, partner, target proxy and full effect arc.','Any required information touches or crosses the panel or capture boundary.')],[1.15,2.75,2.6])
add_p(doc,"The master identity art is approved before the complete skill library. Subsequent boards begin from that package, are compared at matching scale, and receive a consistency pass separate from the choreography pass. It is possible for an action to be thrilling and still fail because the coat, face, spear or tail changed. It is also possible for a consistent drawing to fail because the motion has no obvious anticipation or hero frame. Both reviews are required.")
add_h(doc,'13.4 Camera and overscan policy',2)
add_p(doc,"The battle camera frames the largest expected legal combination, not the average idle. During authoring, each animation exports a body bounding box, a weapon bounding box and a VFX bounding box across its complete timeline. The presentation layer computes the union, adds the eight-percent safety margin, and chooses from a small set of authored camera scales. It never zooms so tightly that a fast spear thrust, Grisha’s halberd, Vix’s tail, Isabella’s rapier, a carried ally or Nara’s dome is clipped. If the move still exceeds the widest camera, the action is redesigned before production approval.")

add_h(doc,'13.5 Visual evidence is required for every visual idea',2)
add_p(doc,"No visually consequential idea is implementation-ready because prose sounds convincing. It needs reference art that answers the production questions raised by that idea. This includes every character, costume state, signature weapon, skill, Echo, enemy family, boss phase, location, route type, tomb mechanism, estate room, household activity, interface mode, traversal verb, important prop, status effect and major transformation. The purpose is not to fill the bible with attractive pictures. The purpose is to make the intended object, action, staging, scale, materials, readable silhouette and implementation boundary visible before animation, environment assembly or UI engineering begins.")
add_callout(doc,'Visual-proof gate',"An item with no approved visual package remains DESIGNED, never ART_LOCKED or BUILD_READY. A single mood painting cannot approve animation, navigation, construction, UI behavior or character identity. Each plate must cite the stable content IDs it proves, while each content record must cite its approved plates.")
add_table(doc,['Design category','Required coherent reference package','What the package must prove'],[
('Playable heroine','Identity turnaround; face and expression sheet; costume construction; weapon orthographic; card portrait; battle-scale lineup; color and material key.','The same adult woman remains recognizable across angles, scales, expressions, lighting and later action boards.'),
('Each of 42 heroine skills','Five-panel action board; isolated VFX sheet; battle-camera composition; target or ally proxy; timing strip; thumbnail at actual game size.','The function is obvious without tooltip text; anticipation, contact and recovery read; body, weapon, partner and complete effect remain inside the safety frame.'),
('Each protagonist Echo','Source-relationship comparison board; distinct silhouette and timing; corruption or instability states; camera/VFX proof.','The Echo is recognizably related to a heroine technique without copying her pose, function or visual ownership.'),
('Enemy family','Adult-scale lineup; front/side construction; locomotion; basic attack; tell/contact/recovery; hit/downed; habitat encounter composition.','Species construction, tactical tell, collision volume, danger range and environmental belonging are unmistakable.'),
('Boss or Champion phase','Phase lineup; arena plan; attack boards; adaptation tells; counter window; damage-state transitions.','The player can visually learn what changed, why it changed and which response is legal.'),
('Region and route','Map tile; horizon key; playable-plane paintover; route entrance/exit; weather/time variants; encounter thumbnail.','Wildness, landmarks, traversal affordances, scale and tactical readability survive the painterly detail.'),
('Tomb mechanism','Architectural plan and section; intact/damaged/active states; interaction close-up; readable hazard sequence; repair result.','The mechanism has a physical function, causal sequence, safe reading distance and reusable kit parts.'),
('Estate room and activity','Room plan; three upgrade states; shared blocking marks; prop sheet; day/night keys; one occupied household scene.','Furniture, navigation, interaction anchors, character activities and visible progression agree spatially.'),
('Interface mode','Wireframe; styled frame; focus order; controller state; common/error/blocked states; game-scale screenshot.','Information hierarchy and transitions remain readable without covering bodies, targets or action effects.'),
('Narrative transformation','Before/after environment or character key; triggering event frame; persistent-state comparison.','The consequence is visible in the world and not confined to journal prose.')],[1.2,3.0,2.3])

add_h(doc,'13.6 Five-panel action-board contract',2)
add_p(doc,"Every skill board uses the same five functional images so the complete library can be compared. Panel 1 is the neutral entry pose and shows how the actor leaves her standard battle state. Panel 2 is anticipation and makes the target, direction and type of action readable. Panel 3 is the hero contact frame: weapon, hand, target proxy and effect origin align at the decisive instant. Panel 4 shows gameplay consequence, including displacement, guard, healing, ward geometry, status removal or affected bands. Panel 5 is recovery and states the actor’s final band, facing, prop state and route back to idle or card compression. A separate narrow timing strip records frames, root motion, hit pause, camera cue, sound cue and cancel window.")
add_table(doc,['Board field','Required value','Rejection condition'],[
('Identity anchors','Character ID, approved identity-plate ID, costume-state ID, weapon ID and exact body height at battle scale.','The board invents a face, body, costume, prop or weapon variation.'),
('Function anchors','Skill ID, valid targets, affected bands, costs, state deltas and final actor position.','The pictures suggest a different mechanic from the authoritative skill record.'),
('Frame envelope','Union of actor, hair, tail, coat, weapon, partner/target and all VFX, plus eight percent empty safety margin.','Any required body part, weapon extremity, partner, target response or effect touches the frame.'),
('Choreography','Entry, anticipation, contact, consequence and recovery form one continuous action.','The board is five unrelated cool poses or requires an unexplained teleport.'),
('Rule-of-cool test','One dominant silhouette and one memorable physical event at contact.','The move is mechanically legible yet visually timid, generic or interchangeable with another heroine.'),
('Animation feasibility','Named reusable clips, bespoke frames, sockets, root-motion distance, target proxy class and maximum simultaneous effects.','The illustrator depends on anatomy changes, unlimited camera movement, fluid simulation or bespoke partner animation outside the approved budget.'),
('Game-size proof','Board includes a screenshot-sized composite with cards, battle plane and UI.','The action reads only when enlarged as concept art.')],[1.25,2.85,2.4])
add_p(doc,"If a skill cannot produce a clear five-panel board inside the camera and animation budgets, redesign the skill before producing more art. Preserve the heroine’s combat role and fantasy, then simplify targets, root motion, partner handling, effect geometry or environmental dependence until the action becomes obvious and achievable. Do not hide an unworkable mechanic behind a beautiful impact frame.")

add_h(doc,'13.7 Reference ledger and consistency graph',2)
add_p(doc,"Reference art is managed as a graph, not as loose pages. Each plate has one stable asset ID, a status, a revision, a list of content IDs demonstrated, a list of source plates inherited, explicit invariants, explicit questions answered and explicit questions still open. Derived plates name the master identity, weapon, palette, camera and environment plates they inherit. When a master changes, the ledger reports every dependent board that must be reviewed. Images are never silently replaced; superseded plates remain addressable so an AI can understand why an old decision no longer governs production.")
add_table(doc,['Ledger field','Meaning'],[
('asset_id','Stable ID such as asset.ref.heroine.betty.identity.v03 or asset.ref.skill.betty.healing_impact.v02.'),
('proves_ids','Exact character, skill, location, UI, enemy or requirement IDs demonstrated by the image.'),
('inherits_assets','Approved master plates whose identity, measurements, palette, materials, camera or architecture are mandatory.'),
('status','DRAFT, REVIEWED, APPROVED, SUPERSEDED or REJECTED. Only APPROVED art can satisfy a build gate.'),
('invariants','Features that may not drift: face, body ratios, weapon dimensions, cultural construction, effect color ownership, camera scale and frame margin.'),
('questions_answered','Concrete decisions visible in the plate, stated as testable assertions rather than mood words.'),
('open_questions','Unresolved decisions that this plate does not authorize an implementer to invent.'),
('implementation_extract','Sprites, dimensions, sockets, tiles, materials, timings, UI states or reusable kit pieces to be produced from the plate.'),
('review_evidence','Identity, choreography, function, camera, cultural, technical and game-size review results with reviewer and date.')],[1.55,4.95])

add_h(doc,'13.8 Visual production order',2)
add_numbered(doc,[
"Approve the global scale sheet, battle camera, exploration camera, value hierarchy, bronze UI material grammar and region palette families.",
"Approve all six heroine identity packages and the protagonist package together at shared scale; resolve silhouette, body, face, costume and weapon collisions before skill art.",
"Create black-silhouette thumbnails for all forty-two heroine skills and all planned Echoes. Reject duplicates and actions whose function cannot be guessed.",
"Create five-panel rough boards for one complete heroine kit. Prove camera envelopes, proxies, root motion and reusable animation vocabulary before rendering polished boards.",
"Revise the remaining five kits from the pilot findings, then board all skills. Run character-consistency and implementation-feasibility reviews separately from spectacle review.",
"Approve enemy-family construction and tell libraries before boss boards so Champion adaptations reuse a known visual language.",
"Approve the island topology and region keys before individual sites; approve architectural kits and mechanism diagrams before polished tomb paintings.",
"Approve estate plans and upgrade-state blocking before household scene illustrations so furniture and bodies occupy repeatable coordinates.",
"Compose game-size integration frames combining the approved character, environment, VFX and UI layers. Treat these as integration tests, not new opportunities to redesign their parts.",
"Lock a visual item only when its ledger entry, dependent IDs, extraction list and review evidence are complete."
])
add_callout(doc,'Current document status',"The six integrated paintings are direction-finding plates, not complete visual proof. They demonstrate several global compositions. They do not satisfy the new per-idea coverage gate. Production must build the reference ledger in the order above, beginning with shared scale and identity, then action silhouettes, before anyone treats the present skill art as final.")

add_h(doc,'13.9 Newly generated visual anchors',2)
add_p(doc,"These plates are the first completed assets from the visual-atlas manifest. They answer three different production questions: what proportion of the island is wilderness, what the world looks like at the playable battle camera, and how the estate communicates work, affection and expedition history. They are direction anchors rather than final character model sheets. Any incidental face, costume or background extra that conflicts with a locked heroine record is discarded; camera, density, material and activity decisions remain useful.")
add_figure(doc,'world-visual-grammar-v1.png','Figure 7 — island wilderness and settlement ratio.','This plate locks the broad world ratio. Tropical wilderness occupies nearly the entire view. High-magical Bronze Age elven roads, gates and funerary towers dominate the built past. A human harbor and fox-folk cove remain small coastal footholds. Dinosaurs cross roads, rivers and canopy as wild inhabitants. Use this density and scale when designing regional establishing views.')
add_figure(doc,'jungle-route-gameplay-v1.png','Figure 8 — playable elven jungle route and single-monster threat.','This plate locks the side-view camera, active-actor scale, inactive portrait-card footprint and individual-monster encounter grammar. Ayla and the razorbeak remain fully readable on one battle plane while upper and flooded lower routes preserve exploration depth. Final UI passes must add the fourth party-state slot without covering the playable floor or reducing full-body clearance.')
add_figure(doc,'estate-household-gameplay-v1.png','Figure 9 — estate work floor and expedition household.','This plate locks the estate as a navigable working place rather than a static menu. Medicine, elven research, training, navigation, recovered artifacts, ship access and heroine-pair flirtation coexist in one readable floor. Incidental guards and animal life are atmosphere, not additional heroines; final character production must replace every principal figure with the approved six identity rigs.')

add_h(doc,'13.10 Two-hundred-image visual atlas manifest',2)
add_p(doc,"This is the first complete visual-development allocation. It contains exactly two hundred distinct production images. Each row produces one separately generated and reviewed image, not one thumbnail inside a catch-all collage. Skill images may contain their required five action panels because those panels explain one continuous asset. Every image inherits approved global, camera, character and faction masters through the reference ledger.")
visual_atlas=[]
global_plates=[
('global.scale_lineup','All playable adults, human reference, orc, fox-folk, jungle elf, eight enemy bodies and one large dinosaur at shared battle scale.'),
('global.battle_camera','Five battle bands, four cards, one expanded heroine, three enemy sizes and the widest legal VFX envelope.'),
('global.exploration_camera','Side-view jungle, estate and tomb spaces with feet, exits, collision edges and interaction anchors.'),
('global.overworld_grammar','Whole island as mostly wild terrain; two compact colonial cities; dominant elven infrastructure; sparse roads and sea routes.'),
('global.materials','Bronze, malachite, wet limestone, tropical timber, sailcloth, black powder, leather UI and temporal glass.'),
('global.light_weather','Noon heat, monsoon, dawn mist, sunset, moonlight and midnight Return flash across one controlled scene.'),
('global.ui_material','Dark bronze and antique-gold frames, cards, buttons, status plaques, tooltips and focus states at final scale.'),
('global.vfx_ownership','Heroine accent colors, physical impact, medicine, ward, powder, command, navigation and Echo effects without silhouette loss.'),
('global.faction_lineup','Colonial human, colonial orc, fox-folk maritime house, jungle-elf community, pirates and estate household.'),
('global.wildness','Jungle route with one dangerous dinosaur, old elven ruin, no settlement, no pen and minimal path maintenance.'),
('global.midnight_return','Island-wide midnight arrival flashes across jungle, road, tomb edge, shore and distant mountain.'),
('global.integration','One final-quality screenshot combining approved heroine, enemy, wild background, cards, command UI and VFX.')]
for slug,desc in global_plates: visual_atlas.append(('GLOBAL',f'asset.ref.{slug}',desc,'Approves shared visual grammar.'))
identity_views=[
('identity','Front, side, back and three-quarter body construction with fixed measurements.'),
('face','Face rotation, twelve expressions, hair construction and species traits.'),
('costume','Exploded costume layers, fasteners, materials, damage state and color values.'),
('weapon','Weapon orthographic, absolute dimensions, grips, sockets, contact points and VFX origins.'),
('performance','Neutral stance, combat idle, card pose, household posture, flirtation and victory gesture.')]
for heroine in ['betty','ayla','vix','grisha','isabella','nara']:
    for view,desc in identity_views:
        visual_atlas.append(('HEROINE',f'asset.ref.heroine.{heroine}.{view}',f'{heroine.title()}: {desc}','Locks identity before skill art.'))
for heroine,rows in skills.items():
    slug=heroine.split(' — ')[0].lower()
    for rank,name,effect in rows:
        skill_slug=name.lower().replace(' ','_')
        visual_atlas.append(('SKILL',f'asset.ref.skill.{slug}.{skill_slug}',f'{slug.title()} {rank} {name}: five-panel entry, anticipation, contact, consequence and recovery board; {effect}','Proves rule-of-cool, function, framing and feasibility.'))
echo_plates=[
('betty.borrowed_tomorrow','Borrowed Tomorrow: defers an ally injury; medicine geometry becomes alien time debt.'),
('betty.trauma_due','Trauma Due: the deferred wound returns with a visible warning and counter window.'),
('ayla.profane_exception','Profane Exception: suspends one known ward rule and marks the protagonist for tomb attention.'),
('ayla.rule_rebound','Rule Rebound: the suspended tomb authority snaps back through bronze lines.'),
('vix.unpaid_favor','Unpaid Favor: produces an improbable resource with a visible future obligation.'),
('vix.collector_arrives','Collector Arrives: alien debt manifests as a tactical claimant.'),
('grisha.false_command','False Command: an alien order moves enemies while damaging composure.'),
('grisha.command_reversal','Command Reversal: disciplined enemies identify and turn the false order.'),
('isabella.stolen_initiative','Stolen Initiative: protagonist acts inside an enemy phase while becoming the primary target.'),
('isabella.attention_claim','Attention Claim: enemy focus converges on the protagonist after the stolen beat.'),
('nara.echo_of_arrival','Echo of Arrival: returns to a previous position-state while one consequence remains.'),
('nara.anchor_scar','Anchor Scar: the abandoned consequence remains visible in space and memory.')]
for slug,desc in echo_plates: visual_atlas.append(('ECHO',f'asset.ref.echo.{slug}',desc,'Separates Echo from heroine ownership.'))
location_sites=['black_beach','estate','fort_calder','nora_gate','veyra_tidehouse','bellamy_house','pallidwood_basin','saint_orra_cathedral','hotel_bellevue','mount_kestrel_quarry','talar_lower_city','ilyon_necropolis']
location_views=[
('approach','Long approach showing region, landmark, wildness, route choices and threat scale.'),
('playable','Final side-view playable plane with exits, feet, hazards, cover and interaction anchors.'),
('mechanism','Close production view of the site-specific machine, civic function, prop or traversal rule.'),
('changed','Persistent changed-state view after the site’s major player consequence.')]
for site in location_sites:
    for view,desc in location_views:
        visual_atlas.append(('LOCATION',f'asset.ref.location.{site}.{view}',f'{site.replace("_"," ").title()}: {desc}','Defines a buildable location state.'))
enemy_families=['razorbeak_raptor','river_tyrant','bronze_burial_servitor','ash_revenant','canopy_stalker','powder_ape','tide_serpent','temporal_moth_knight']
enemy_views=[
('identity','Adult-scale construction, physical variations, level-1/6/12 development and readable anatomy.'),
('action','Basic attack, level-unlocked action, reaction, hit, stagger, retreat and death/Return flash.'),
('habitat','One individual in its habitat with patrol purpose, pre-contact evidence and encounter distance.')]
for enemy in enemy_families:
    for view,desc in enemy_views:
        visual_atlas.append(('ENEMY',f'asset.ref.enemy.{enemy}.{view}',f'{enemy.replace("_"," ").title()}: {desc}','Makes one monster an individual threat.'))
ui_plates=[
('battle.idle','Four cards, five bands, initiative, intents, resources and one active actor.'),
('battle.expand','Card-to-full-body expansion at start, middle and locked active state.'),
('battle.target','Legal and blocked targets, path, affected bands, cost and predicted consequence.'),
('battle.impact','Contact, hit pause, damage, condition, guard loss and readable event log.'),
('battle.retreat','Retreat target, exit band, pursuit requirement, cost and confirmation.'),
('battle.boss','Large Champion phase, adaptation evidence, counter tell and protected UI space.'),
('exploration.arrival','Authored location text, controllable leader, party cards, exits and inspectable features.'),
('exploration.inspect','Selected phrase, companion response, tool verbs, blocked reason and journal record.'),
('overworld.route','Known, rumored and inferred nodes; route time, supplies, weather, monster evidence and retreat anchor.'),
('estate.home','Time, residents, active projects, heroine activities, available dates and departure status.'),
('party.load','Four active slots, reserves, equipment, load, heroine contributions and route warnings.'),
('journal.evidence','Observation, testimony, inference, contradiction, confidence and actionable comparison.'),
('romance.milestone','Heroine portrait, named story milestone, initiated scene and next authored opportunity without meters.'),
('recruitment','Prospective heroine, current recruiters, attraction/courtship/join state and available scene.'),
('midnight.return','World flash, returned people, refreshed monster evidence and changed morning map.'),
('accessibility','Text scale, reduced motion, high contrast, color-independent statuses and controller focus.')]
for slug,desc in ui_plates: visual_atlas.append(('UI',f'asset.ref.ui.{slug}',desc,'Approves one complete player-facing state.'))
household_plates=[
('arrival_first_night','First night at damaged estate: Betty, protagonist, one safe room and clear romantic promise.'),
('betty_infirmary','Betty runs the infirmary, then initiates private affection after treatment.'),
('ayla_ritual_room','Ayla installs funerary objects, teaches a rite and requests a private evening.'),
('vix_dock_date','Vix turns dock inspection into a flirtatious date and proposes recruiting a woman she noticed.'),
('grisha_build_day','Grisha commands repairs, admires the protagonist’s work and volunteers a celebration.'),
('isabella_party','Isabella stages a household party, dresses the group and makes the protagonist the admired center.'),
('nara_night_chart','Nara maps temporal stars with the protagonist and initiates intimacy.'),
('group_breakfast','Entire household at breakfast with touch, jokes, work plans and visible pair chemistry.'),
('group_bath','Tasteful adult fanservice: household bathing, flirtation, distinct bodies and voluntary participation.'),
('victory_return','Heroines welcome the protagonist after a heroic act with admiration, affection and celebration.'),
('recruit_notice','One heroine notices a compatible woman and clearly begins recruitment.'),
('recruit_courtship','Protagonist and two heroines court a prospective member during an adventure.'),
('recruit_join','Prospective heroine chooses the household; room activation, kisses and group welcome.'),
('pair_romance','Two heroines initiate a female-female romantic scene inside the household.'),
('full_harem_evening','MFFF+ household at leisure: protagonist admired, women affectionate with him and one another.'),
('finale_epilogue','Permanent happy household after the finale, expanded estate, future expedition and no romantic rugpull.')]
for slug,desc in household_plates: visual_atlas.append(('HOUSEHOLD',f'asset.ref.household.{slug}',desc,'Proves the haremlit promise on screen.'))
assert len(visual_atlas)==200
add_table(doc,['Class','Asset ID','Exact image','Approval job'],visual_atlas,[.75,1.85,3.1,.8])
add_callout(doc,'Atlas completion rule',"The atlas is complete only when all two hundred asset IDs have an approved image file, prompt record, inherited master list, review evidence and implementation extraction. A missing image is visible debt. A collage cannot satisfy several IDs unless each required subject remains separately legible at game-production scale.")

add_h(doc,'14. Audio and music',1)
add_p(doc,"Music should treat the island's cultural mixture as history rather than as a shuffle playlist. Regions have instrumental families and rhythmic assumptions that can be transformed by anomaly, occupation or memory. The estate theme gains parts as rooms are restored and people join the household. Combat layers should respond to actor expansion, enemy phase, composure danger and decisive bond skills without restarting the composition every turn. Sound effects carry much of the combat's physical credibility: leather frame movement, bronze latches, boots arriving on the plane, powder mechanisms, weapon air, impact, medical glass and the unnaturally clean tones of Echoes.")

add_h(doc,'15. The prototype slice',1)
add_p(doc,"The prototype must prove the game's unusual conjunction of systems in one continuous loop. A truthful slice begins at the damaged estate, allows a short preparation with Betty and Ayla, travels across two overworld routes, explores one compact funerary facility, resolves two ordinary encounters and one adaptive Champion encounter, returns home, and triggers consequences in both a relationship and the physical house. It should take forty-five to seventy-five minutes on a first playthrough and remain worth replaying because route knowledge and tactical choices alter the result.")
add_h(doc,'15.1 Slice content',2)
add_table(doc,['Area','Included proof'],[
('Estate','Infirmary corner, chart table, two companion behaviors, one upgrade choice and one private conversation.'),('Overworld','Three nodes, a safe road, a risky jungle trail, one anomaly warning and a retreat path.'),('Tomb','Reception threshold, processional ramp, maintenance route, water rule, sealed containment room and shortcut.'),('Combat','Four party cards, one expanded actor at a time, five bands, two enemy families, a miniboss and clean retreat.'),('Progression','Betty D-C skills, Ayla D-C skills, two protagonist Echoes and one bond-state consequence.'),('Champion','Tracks one repeated tactic, telegraphs adaptation and allows the player to counter it through evidence.'),('Return','Injury treatment, visible estate change, journal update and a companion-to-companion reaction.')],[1.25,5.25])
add_h(doc,'15.2 Slice acceptance tests',2)
add_bullets(doc,[
"A new player can explain why party members are cards and predict who will expand before confirming an action.",
"The player can identify at least one tomb rule from environment and journal evidence without a quest marker stating the answer.",
"Retreat preserves meaningful progress and produces authored aftermath rather than feeling like a reload prompt.",
"At least one relationship choice changes tactical or exploration availability without functioning as a simple affection payment.",
"The Champion’s adaptation is noticed, understood and countered by most test players after sufficient evidence.",
"Returning to the estate feels different in sight, sound and behavior from leaving it."
])

add_h(doc,'16. Godot 4 architecture',1)
add_p(doc,"The architecture should keep authored content in resources and data, simulation in plain systems, and presentation in scenes that can be replaced without rewriting rules. Battles, travel and household events all need deterministic state transitions so saves, replays, testing and adaptation logs remain comprehensible. Signals are useful at presentation boundaries, but global event soup will make the Champion and relationship systems impossible to reason about. Prefer explicit commands, typed events and a small set of state owners.")
add_h(doc,'16.1 Scene tree',2)
add_table(doc,['Scene / system','Responsibility'],[
('GameRoot','Boot, service ownership, save slot, scene transitions and error boundary.'),('WorldMapScene','Location graph, route previews, travel presentation and encounter handoff.'),('TravelScene','Route-event presentation, decision input and projection of ExpeditionState changes.'),('ExplorationScene','Side-view site, navigation anchors, interactables, hazards and local persistence.'),('CampScene','Anchor quality, assignments, treatment, recovery, conversation eligibility and ambush handoff.'),('BattleScene','Battle state projection, actor cards, active plane, targeting, animation timeline and results.'),('EstateScene','Upgrade state, resident schedules, ambient behaviors, conversations and crafting entry points.'),('ContentDB','Loads validated resources for characters, abilities, items, enemies, sites and events.'),('CampaignState','Authoritative chapter, clocks, faction state, relationships, knowledge, inventory and site persistence.'),('ExpeditionState','Current party, objective, edge/site position, packed load, fatigue, injuries, time, alert, seed and event ledger.'),('WorldService','Commits midnight Return, death records, daily habitats, world day and population events.'),('AdaptiveDirector','Consumes explicit observation events and selects allowed Champion adaptations.'),('SaveService','Versioned serialization, migrations, atomic writes, autosave policy and debug snapshots.')],[1.55,4.95])
add_h(doc,'16.2 Suggested folders',2)
add_p(doc,"A practical project layout separates `scenes/` by mode, `systems/` by authoritative domain, `content/resources/` by data type, `art/` by production family, `audio/`, `ui/`, `tests/`, and `tools/validation/`. Generated art should never be placed directly into shipping folders without an asset record that identifies source, dimensions, crop, character version, approval state and required derivatives. The same rule applies to dialogue and data generated with assistance: provenance and review status belong in the pipeline, not in somebody's memory.")
add_h(doc,'16.3 Data resources',2)
add_table(doc,['Resource','Key fields'],[
('CharacterDef','id, display_name, signature_weapon, base_stats, card_scene, actor_scene, skill_ids, palette_id, animation_set'),('AbilityDef','id, owner_id, bond_rank, costs, target_rule, command_steps, animation_id, vfx_ids, ai_tags'),('RelationshipState','character_id, household_member, protagonist_romance_milestone, pair_arc_states, bond_rank, scene_flags, recruitment_focus_id'),('DeathMemory','death_id, character_id, cause_id, location_id, world_time, responsible_actor_ids, witness_ids, return_outcome'),('DailyHabitatState','world_day, habitat_id, daily_seed, individual_ids, species_ids, levels, trait_sets, conditions, patrol_purposes, anchors, alive_states, loot_claims'),('WorldNodeDef','id, region, map_position, certainty_rules, service_ids, site_id, outgoing_edge_ids'),('RouteEdgeDef','id, endpoints, directionality, duration_range, terrain_tags, transport_rules, camp_anchors, event_budget'),('TravelEventDef','id, eligibility_query, repetition_rule, text_blocks, response_commands, state_deltas, observation_ids'),('SiteDef','id, region, scene, space_ids, entry_rules, site_rules, persistence_schema, encounter_ids'),('SpaceDef','id, site_id, scene, entry_anchors, exit_ids, feature_ids, hazards, ambience, camera_profile'),('InteractionDef','id, feature_id, verb, condition_groups, blocked_reason, prose_blocks, cost_preview, command, state_delta'),('ObservationDef','id, proposition, evidence_class, source_rule, confidence_delta, journal_links, localization_keys'),('CampDef','id, anchor, quality, risks, allowed_assignments, recovery_profile, encounter_rule'),('EncounterDef','participant_ids, threat_cost, environment, objectives, retreat_rules, observation_tags, rewards, aftermath_event'),('AdaptationDef','trigger_query, evidence_requirements, telegraph_event, modifiers, counters, cooldown, story_limits')],[1.55,4.95])
add_h(doc,'16.4 Pseudocode: commit an expedition command',2)
code0="""func commit_expedition_command(command: ExpeditionCommand) -> CommandResult:
    var before := expedition_state.snapshot()
    var validation := expedition_rules.validate(command, before, content_db)
    if not validation.allowed:
        return CommandResult.blocked(validation.player_reason)

    var result := expedition_rules.resolve(command, before, rng_stream)
    assert(result.delta.has_declared_costs(command.preview))
    assert(result.events.all(func(e): return e.has_stable_id()))

    expedition_state.apply(result.delta)
    campaign_state.apply(result.campaign_delta)
    event_ledger.append_transaction(command, result, before.hash())
    save_service.commit_boundary(result.save_boundary)
    return result"""
p=doc.add_paragraph(); p.paragraph_format.space_after=Pt(10); p.paragraph_format.left_indent=Inches(.18); p.paragraph_format.keep_together=True
for line in code0.splitlines(): font(p.add_run(line+'\n'),8.5,color='EDE7DC',name='Cascadia Mono')
shd=OxmlElement('w:shd'); shd.set(qn('w:fill'),DEEP); p._p.get_or_add_pPr().append(shd)
add_p(doc,"Presentation receives the returned events only after authoritative state commits. If an animation, scene transition or audio cue fails, the command remains explainable and recoverable from the ledger. Preview and resolution share the same command definition, so the UI cannot promise a cost that a different code path silently changes.")
add_h(doc,'16.5 Pseudocode: card-to-active action',2)
code="""func commit_and_present_player_command(command: BattleCommand) -> void:\n    input_lock.acquire(&\"combat_command\")\n\n    # BattleService validates and commits before presentation begins.\n    var result := battle_service.commit_action(command)\n    if result is CommandBlocked:\n        battle_hud.show_blocked_reason(result.reason_id)\n        input_lock.release(&\"combat_command\")\n        return\n\n    # Everything below consumes the immutable committed result.\n    var actor := content_db.character(result.actor_id)\n    var card := party_rail.card_for(result.actor_id)\n    await card.play_expand_transition(actor_stage.anchor_for(result.actor_id))\n    actor_stage.mount_actor(actor.presentation_scene_id)\n    targeting_layer.show_committed_targets(result.target_ids)\n    await action_timeline.play(result.animation_id, result.events)\n    battle_hud.project_snapshot(result.after_snapshot)\n\n    await actor_stage.play_recovery(result.recovery_state)\n    if result.after_snapshot.expanded_actor_id == result.actor_id:\n        actor_stage.hold(result.actor_id)\n    else:\n        await card.play_compress_transition(actor_stage.release(result.actor_id))\n\n    input_lock.release(&\"combat_command\")"""
p=doc.add_paragraph(); p.paragraph_format.space_after=Pt(10); p.paragraph_format.left_indent=Inches(.18); p.paragraph_format.keep_together=True; set_cell=None
for line in code.splitlines(): font(p.add_run(line+'\n'),8.5,color='EDE7DC',name='Cascadia Mono')
shd=OxmlElement('w:shd'); shd.set(qn('w:fill'),DEEP); p._p.get_or_add_pPr().append(shd)
add_h(doc,'16.6 Pseudocode: bond unlock',2)
code2="""func evaluate_bond_unlock(rel: RelationshipState, skill: AbilityDef) -> UnlockResult:\n    if rel.protagonist_romance_milestone != skill.required_romance_milestone:\n        return UnlockResult.blocked(\"romance_milestone\")\n    if not campaign.flags.has_all(skill.required_story_flags):\n        return UnlockResult.blocked(\"personal_story_milestone\")\n    if skill.requires_household_member and not rel.household_member:\n        return UnlockResult.blocked(\"household_milestone\")\n    return UnlockResult.available(skill.unlock_event_id)"""
p=doc.add_paragraph(); p.paragraph_format.space_after=Pt(10); p.paragraph_format.left_indent=Inches(.18); p.paragraph_format.keep_together=True
for line in code2.splitlines(): font(p.add_run(line+'\n'),8.5,color='EDE7DC',name='Cascadia Mono')
shd=OxmlElement('w:shd'); shd.set(qn('w:fill'),DEEP); p._p.get_or_add_pPr().append(shd)
add_h(doc,'16.7 Pseudocode: adaptive Champion',2)
code3="""func choose_adaptation(encounter_id: StringName) -> AdaptationDef:\n    var evidence := observation_log.window(last_n_encounters = 4)\n    var candidates := adaptation_db.query(evidence.tags)\n    candidates = candidates.filter(func(a): return a.is_story_legal(campaign))\n    candidates = candidates.filter(func(a): return a.has_telegraph_available(campaign))\n    candidates = candidates.filter(func(a): return not cooldowns.active(a.id))\n\n    var scored := candidates.map(func(a):\n        var adjusted := a.pattern_score(evidence)\n            - a.repetition_penalty(history)\n        return {\"def\": a, \"score\": adjusted})\n    var chosen := weighted_pick_top_band(scored)\n    campaign.queue_event(chosen.def.telegraph_event)\n    history.record(encounter_id, chosen.def.id, evidence.summary())\n    return chosen.def"""
p=doc.add_paragraph(); p.paragraph_format.space_after=Pt(10); p.paragraph_format.left_indent=Inches(.18); p.paragraph_format.keep_together=True
for line in code3.splitlines(): font(p.add_run(line+'\n'),8.5,color='EDE7DC',name='Cascadia Mono')
shd=OxmlElement('w:shd'); shd.set(qn('w:fill'),DEEP); p._p.get_or_add_pPr().append(shd)

add_h(doc,'16.8 Pseudocode: midnight Return transaction',2)
code4="""func resolve_midnight() -> CommandResult:\n    var before := campaign_state.snapshot()\n    assert(before.clock.is_midnight_boundary())\n    var next_day := before.world_day + 1\n    var daily_seed := hash([before.campaign_seed, next_day, before.island_pressure_version])\n    var delta := WorldDelta.new(next_day, daily_seed)\n\n    for character_id in before.dead_character_ids.sorted():\n        var record := world_rules.resolve_character_return(\n            character_id, before, rng.stream(daily_seed, character_id))\n        delta.add_character_return(record)\n\n    for habitat_id in content_db.active_habitat_ids().sorted():\n        var daily_state := world_rules.generate_daily_habitat(\n            habitat_id, before, rng.stream(daily_seed, habitat_id))\n        delta.add_habitat_state(daily_state)\n\n    delta.add_daily_campaign_changes(campaign_rules.resolve_new_day(before, daily_seed))\n    var result := world_service.commit_midnight(delta, before.hash())\n    save_service.commit_boundary(&\"midnight_resolved\")\n    return result"""
p=doc.add_paragraph(); p.paragraph_format.space_after=Pt(10); p.paragraph_format.left_indent=Inches(.18); p.paragraph_format.keep_together=True
for line in code4.splitlines(): font(p.add_run(line+'\n'),8.5,color='EDE7DC',name='Cascadia Mono')
shd=OxmlElement('w:shd'); shd.set(qn('w:fill'),DEEP); p._p.get_or_add_pPr().append(shd)

add_h(doc,'17. Save, validation and test strategy',1)
add_p(doc,"Campaign state should be serializable without scene nodes. Every save carries a schema version, build identifier, content manifest hash and migration history. Writes are atomic: serialize to a temporary sibling, validate, then replace. The complete autosave matrix in section 1.11 is authoritative; this architecture must support all of those boundaries rather than the smaller illustrative subset used in earlier drafts. Debug snapshots should make it possible to reproduce a route event, camp attack, Champion adaptation or broken tomb state without replaying hours of content.")
add_h(doc,'17.1 Test layers',2)
add_bullets(doc,["Unit tests for costs, targeting, state deltas, bond gates, faction arithmetic and adaptation queries.","Content validation for missing IDs, illegal rank sequences, absent animations, inconsistent weapon assets and unreachable site links.","Deterministic battle simulations with seeded randomness and golden event logs.","Scene integration tests for expand/compress cancellation, save during transitions, retreat, reload and controller focus.","Visual regression captures for cards, actor scale, VFX occlusion, common aspect ratios and accessibility modes.","Narrative state audits that enumerate household membership, romance milestones, heroine-pair milestones, mutually exclusive flags and household schedule dead ends."])

add_h(doc,'18. Production sequence',1)
add_p(doc,"The project should advance through proofs that retire the largest risks. Building a large overworld before the card-to-active presentation works would hide the central production question. Producing finished animations before weapon dimensions and actor anchors are fixed would create expensive inconsistency. Writing every romance scene before relationship state and household scheduling are playable would make the narrative impossible to test as a system.")
add_numbered(doc,[
"Prove one battle with gray-box cards, one expanded actor, legal targeting, damage, guard, retreat and deterministic logging.",
"Replace Betty with a production-quality identity set and Healing Impact, including card transition, separated VFX and timing data.",
"Add Ayla to prove that a second silhouette, weapon and rules-based ability can share the grammar without feeling reskinned.",
"Build the compact tomb and knowledge journal, then connect evidence to one combat and one traversal advantage.",
"Build the three-node overworld and estate return so preparation, travel, expedition and aftermath form a complete loop.",
"Add one admiration scene, one heroine-initiated romantic scene, one happy household group scene and a skill unlock whose requirements are visible in debug tools.",
"Add the Adaptive Director with one telegraphed response and at least two counterplays.",
"Run external playtests before expanding content; revise the grammar, then scale through validated libraries."
])
add_h(doc,'18.1 Prototype team backlog',2)
add_table(doc,['Discipline','First production deliverables'],[
('Design','Battle bands, command schema, two heroine kits, one enemy family, tomb rules, retreat and slice balance.'),('Narrative','Opening, Betty/Ayla field dialogue, one admiration scene, one heroine-initiated scene, one bond unlock, estate return scene and knowledge entries.'),('Character art','Betty and Ayla identity packs, cards, actor cutouts, core battle poses and household states.'),('Environment art','Estate room, overworld map segment, tomb reception/procession/containment layers and battle backdrop.'),('VFX/UI','Bronze frame kit, card transition, targeting, status language, Healing Impact and tomb mechanism effects.'),('Engineering','State model, content DB, battle runner, scene projection, saves, journal, route travel and validation tools.'),('Audio','Estate bed, jungle/tomb ambiences, card mechanics, Betty kit, two enemies and adaptive Champion motif.')],[1.1,5.4])

add_h(doc,'19. Reference-art notes and limitations',1)
add_p(doc,"The six images in this bible are visual development plates generated from the current written direction. They are useful because they establish composition, hierarchy, materials, scale, route readability, environment layering, silhouette differentiation and animation intent. They are not source-of-truth model sheets. Character faces, anatomy, cultural details, weapon construction, iconography, typography and UI dimensions must pass through controlled turnarounds and implementation tests before being treated as final.")
add_p(doc,"The battle screenshot referenced in the request was not available at its supplied path in the current workspace, so these plates do not claim pixel-level fidelity to it. The written elements preserved from the conversation—compressed cards, one expanded actor, painterly backgrounds, dramatic effects and dark bronze theatrical framing—were used as the stable direction. When the original screenshot is restored, the art director should perform a deliberate comparison for camera height, card proportions, active-actor scale, command placement, frame density and contrast, then record any decisions as updates to this bible rather than allowing silent drift.")
add_h(doc,'19.1 Approval checklist for future reference art',2)
add_bullets(doc,["Does the image answer a production question rather than merely repeat the mood?","Can character, environment, UI and VFX layers be separated conceptually and technically?","Are all adult characters distinct in silhouette, culture, posture and weapon handling?","Are playable routes and collision planes readable at expected screen size?","Does ornament preserve information hierarchy?","Can this image be converted into dimensions, anchors, states or a reusable asset library?","Have historical and cultural cues been reviewed for coherence rather than copied as decoration?"])

add_h(doc,'20. Final design principles',1)
principles=[
('Let the thought finish.','Worldbuilding, romance and mechanics should be given enough prose and enough play time to establish cause and consequence. The game does not need slogans where it needs explanations.'),
('Make mystery evidential.','The player may not know the answer, but should be able to name what was observed and why a hypothesis is reasonable.'),
('Keep bodies readable.','Cards exist to make room for the active performer. Camera, VFX and UI must protect the full-body silhouette.'),
('Let danger stay uneven.','Not every door is appropriate now. Retreat, preparation and remembered knowledge are legitimate forms of mastery.'),
('Treat the women as admired partners.','Each heroine is formidable, openly desires the hero in her own way, builds friendships and romances inside the harem, contributes essential work, and chooses a shared future with the household.'),
('Make home visible.','Upgrades, artifacts, injuries, habits and relationships should occupy space and change behavior.'),
('Give the ancient world a job.','Tombs, wards, roads and machines came from institutions with labor and purpose. Their current danger grows from that history.'),
('Allow the enemy to learn fairly.','Adaptation is telegraphed, evidenced and counterable. It should provoke new play, not punish investment.'),
('Build libraries, not accidents.','Names, anchors, palettes, timing and reusable states turn rich art into a sustainable game.'),
('Keep the interface honest.','The island can lie. The UI reports what the party knows, what an action costs and why a rule applied.')]
for title,text in principles: add_p(doc,title+' — '+text,bold_lead=title+' — ')
add_callout(doc,'The intended result',"A dangerous, gorgeous, sometimes strange party RPG in which the island becomes more intelligible without becoming tame, combat gives every heroine the stage she deserves, romance changes how people live and fight together, and the estate slowly becomes the one place in an impossible history that the player can honestly call home.")

# metadata
doc.core_properties.title='Project 42: Pirate Island RPG — Design Bible'
doc.core_properties.subject='World, combat, relationship, visual and Godot production design'
doc.core_properties.author='Project 42 Design Team'
doc.core_properties.keywords='Godot, RPG, pirate island, design bible, 2D, combat, haremlit'
OUT.parent.mkdir(parents=True, exist_ok=True)
doc.save(OUT)
print(OUT)
