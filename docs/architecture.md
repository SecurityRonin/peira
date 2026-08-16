# How peira works, and where it currently does not

> The reasoning this checker enforces is written out in **[`method/`](method/)** — the six
> structures of investigative error, the claim-grading standard, the register of silently-failing
> sources, and the tribunal-reporting discipline. peira stands alone: nothing in this repository
> depends on any private configuration of its author.

This document describes the system as built, and records — in the same place — what an adversarial
audit established about it on 2026-08-13. Documenting the design without the audit would be the
overstatement peira exists to prevent.

**Status of the central claim: repaired, and re-audited.** An audit on 2026-08-13 falsified it —
five criticals, reproduced against the binary. All five are fixed. A second audit against the
repaired system, with a **seeded false finding as a negative control** (rejected 0-5, every judge
refuting it by execution), found eighteen further defects; eleven are fixed, and what remains is
recorded below rather than described away.

The defects that mattered most were not wrong rules. They were correct rules losing force at a
join — a verdict discarded before the decision point, a check one caller could see and another
could not, a filter that made every gate evadable by moving the content one hop up the support
chain. **Read the register as the shape of how a checker fails, not as a list of bugs.**

## What you can rely on today

A register of defects without an operational boundary leaves a reader worse off than a short honest
list. This is the boundary. It is deliberately smaller than the coverage map, because the map
describes what each check *does* and this describes what its output *supports*.

**Sound enough to build on:**

- **The parser refuses authored `status:` and `confidence:`.** Claim state cannot be typed in. This
  one is structural and holds.
- **The graph and the derived index are queryable structure.** Nodes, edges, fields and grades are
  faithfully loaded, and the index answers questions over them.
- **An individual violation is a real signal to inspect.** When a gate blocks, it has found
  something. False *positives* are not the problem here.

**Not an assurance — do not rely on any of these as a verdict:**

| Output | Why not |
|---|---|
| "All enforced gates pass" | gates that reached no verdict are discarded before this is printed |
| `review_ready` | computed from an empty blocking list and grounded membership; nothing reads whether a reviewer signed |
| grounded membership | attack edges are author-declared; standing can be manufactured |
| a verified packet | `verify` compares the rendered projection only. A format-line edit no longer demotes tampering: if the format number is the SOLE difference from the current rendering, that is an edit and is named one. Where the body differs beyond it, staleness and alteration are indistinguishable from the artifact, and `verify` says so rather than picking one |
| a mismatch as evidence of tampering | a vault that GREW and one whose evidence was ALTERED give the same verdict. `verify` names the first differing line so you can tell them apart; it cannot tell them apart for you |
| the `by=` on a grade | an unauthenticated free string. Anyone able to write the vault can attach one. The packet now DISCLOSES this and names who is credited, but peira cannot check it — the gates are pure functions of the graph, so they cannot consult the version-control history that would answer it |
| a settled grade | the ceiling binds only edges that declare a means of knowing — an undeclared one now reaches no verdict rather than passing, but a *contradicted* declaration is only reported, never corrected |
| a claim standing because a rival was withdrawn | the packet and `peira status` now DISCLOSE this — read the line; it names the withdrawn attacks rather than claiming they were defeated |
| absence of a privilege warning on a packet | the lint fires on the node carrying the material; `freeze` examines the claim's evidential closure, so a leak on a *rendered rival* is scanned as text but its own gate profile is not examined |
| the generated safe statement | it quotes author-written term fields verbatim |

**The honest summary:** peira today is a **structured examination aid that catches specific named
mistakes**. It is not yet an assurance that a claim is safe to rely on, and no output of it should be
offered to a tribunal as though it were. Every row above is tracked in the register below.

---

## 1. The vault is the source; everything else is compiled

The discipline, stated normatively in [`method/claim-grading.md` §8](method/claim-grading.md):
findings, corrections and retractions land in the knowledge base *first*, and every report, chart
and memo is a build artifact regenerated from it. Correcting only a deliverable does not fix an
error — **it forks it**: the same fact then exists with two values, and the stale copy is the
dangerous one, because it reads as the record and gets inherited.

peira is the mechanical form of this rule. The vault is the source. `peira packet` is the
generator. `peira verify` re-derives the packet from the vault as it stands and compares
digests — which converts *"the rendered projection agrees with the record"* from a promise into a
build step. The projection is narrow and the digest covers exactly it; the fourth row below says
what is inside it and what is not.

```mermaid
flowchart LR
    V["vault/ — the source<br/>hand-edited markdown"] --> P["parse<br/>refuses status: and confidence:"]
    P --> G["Graph<br/>typed nodes and edges"]
    G --> C["gates, lints,<br/>grounded extension"]
    C --> F["freeze"]
    F --> PK["packet — a projection<br/>sha256 over the rendered body"]
    G --> IX["index.sqlite — disposable<br/>dropped and rebuilt each run"]

    PK --> VY["verify"]
    V -.->|"re-derive from the<br/>vault as it stands"| VY
    VY -->|"byte-identical"| OK["Verified"]
    VY -->|"differs"| DM["DigestMismatch —<br/>a fork exists"]
    VY -->|"stored body declares<br/>another format"| FS["FormatSuperseded —<br/>no verdict, exit 2"]
```

How each half of the discipline maps onto the machinery, honestly:

| Rule | Mechanism | Status |
|---|---|---|
| Record before you report | There is no prose write path into a packet: `freeze` renders it from the graph, and the parser refuses `status:` and `confidence:` outright | **enforced** |
| Correct at the source, then rebuild | `verify` re-derives from the vault and compares digests. An edit that reaches the rendered body surfaces as `DigestMismatch`; one confined to unrendered fields — a grade, a grader, a `via=`, a `measured_by:` — changes no digest, and a packet edit that touches the `Packet format:` line surfaces as `FormatSuperseded`, which is no verdict at all | **enforced, narrow — defects 6 and 8** |
| Mark supersession, never silently overwrite | `supersedes:` and `retracts:` edges are read by `PEIR-LINT-RETRACTED`, by the grounded extension, and by the packet's standing line | **enforced** |
| The deliverable is a projection, never the whole | A packet renders a fixed projection: the claim's title and warrant, the ids and titles of its direct supporters, contradictors and limiters, boundaries, falsifiers, and the term moments. Grades, graders, pramāṇas and instrument links are not in it — so they are not under the digest either | by construction — **and the digest inherits the narrowness (defect 8)** |
| If it cannot be regenerated, it is not compiled — it is a fork | `Verified` means exactly: the rendered body re-derived byte-identically from the source | **enforced** |

The derived index is the same discipline in miniature: `peira index` drops and rebuilds the
database from the markdown on every run, it is gitignored, and it is never updated incrementally —
an incrementally-updated index can drift from the files it describes, and once it can drift it is a
second source of truth nobody reviewed.

### The supersession gap

The third row is the honest hole in this section. The vault can *say* that a claim is withdrawn —
`retracts:` is an edge kind, the loader builds it, and `is_attack()` deliberately excludes it so a
retraction cannot be "defended against" by a third claim. For a long time nothing else read it: the
record was kept and its meaning dropped, which is the **structural synonym** for the `status:
withdrawn` the parser refuses. Three things read it now, and the third is the one that matters —
a retraction is an *authored assertion*, so removing an attack silently would let the last writer
win. It is disclosed instead:

```mermaid
flowchart LR
    W["retracts: c1<br/>written in a note"] -->|"parsed by the loader"| E["Retracts edge<br/>in the graph"]
    E -->|"is_attack is false,<br/>by design"| A["not an attack —<br/>correct"]
    E -->|"PEIR-LINT-RETRACTED,<br/>while load-bearing"| L["reported —<br/>c1 will not freeze"]
    E -->|"leaves the attack relation"| G2["a withdrawn rival<br/>no longer defeats"]
    E -->|"named in the packet"| S["standing line says WITHDRAWN,<br/>never 'defeated'"]

    L:::mended
    G2:::mended
    S:::mended
    classDef mended fill:#1a3a2a,stroke:#3a7,color:#fff
```

A packet therefore freezes for a claim the vault records as withdrawn. This is the structural
synonym for the hand-written `status: withdrawn` the parser refuses at the front door — refused as
a field, accepted as an edge, honoured as neither.

---

## 2. The claim graph

Everything is a typed node in one graph. The load-bearing distinction is between nodes that make
assertions and can therefore be attacked, and nodes that qualify assertions without competing.

```mermaid
flowchart TB
    subgraph ARG["Arguments — compete in the grounded extension"]
        direction LR
        CL["Claim<br/>the mergeable unit"]
        HY["Hypothesis<br/>a candidate explanation"]
        OB["Observation<br/>points at sealed evidence by hash"]
        DI["Dissent<br/>a preserved minority position"]
    end

    subgraph REF["Reference — qualifies, never competes"]
        direction LR
        TE["Term<br/>meaning stipulated before use"]
        CR["Criterion<br/>the standard applied"]
        IN["Instrument<br/>the detector behind an observation"]
    end

    subgraph PROC["Process"]
        direction LR
        PR["Protocol<br/>a reproducible test"]
        RU["Run<br/>one execution of a protocol"]
        EX["Examination<br/>a lens having been run"]
    end

    subgraph ART["Artifacts"]
        QU["Question"]
        PK["Packet<br/>a frozen citation export"]
    end

    OB -->|"supports"| CL
    RU -->|"supports"| CL
    HY -.->|"limits"| CL
    DI -->|"attacks"| CL
    CL -->|"uses_term"| TE
    CL -->|"judged_by"| CR
    OB -->|"measured_by"| IN
    RU -->|"instance_of"| PR
```

Two rules give the shape its force:

- **A node has no `status` and no `confidence` field.** The parser *refuses* a document carrying
  one, naming the offending key. Claim state is derived, never written.
- **A settled grade is stored inseparably from its grader.** `Edge` holds `(Grade, String)` as one
  value, so an unattributed grade is not a lint failure caught later — it is a value that cannot be
  constructed. A grade written without a grader degrades to a *proposal*, which asserts nothing.

---

## 3. Two axes that are routinely conflated

Most claim-tagging schemes collapse *how you know* and *what state the claim is in* onto one axis.
peira keeps them apart, because a claim can be quoted **and** unverified at once.

```mermaid
flowchart LR
    subgraph MOK["How you know — rides on the EDGE, authored"]
        direction TB
        P1["perception — pratyakṣa"] --> C1["ceiling G3"]
        P2["inference — anumāna"] --> C2["ceiling G2"]
        P3["comparison — upamāna"] --> C3["ceiling G1"]
        P4["testimony — śabda"] --> C4["ceiling G1"]
    end

    subgraph ST["What state it is in — DERIVED, never authored"]
        direction TB
        S1["the blocking list,<br/>empty or not"]
        S2["in or out of the<br/>grounded extension"]
    end

    MOK -->|"caps the grade an edge may<br/>carry — when via= is declared"| GR["settled grade<br/>G0 to G4"]
    ST -->|"computed on demand —<br/>nothing reads whether<br/>a reviewer signed"| DS["derived state, e.g. review_ready:<br/>ready FOR review, not reviewed"]
```

The derived half has exactly two inputs — the blocking list and grounded membership. `peira
status` never reads whether anyone signed; `review_ready` asserts readiness for review, not that
review happened. And because defect 1 discards `Unassessed`, an empty blocking list can also mean
gates that reached no verdict.

**The ceiling refuses G4 to any single edge that declares a pramāṇa** — multiple materially
independent convergent lines are a property of the graph, not of one piece of evidence, and no
pramāṇa's ceiling reaches G4. That refusal is the entire mechanism. **Retraction:** this document
previously said a G4 "can never be asserted — only earned". Both halves were false. No graph
operation computes convergence or derives a grade from it — nothing earns G4 — and the loader
constructs a settled G4 edge directly from `grade=G4 by=…`, so the only G4 the system can hold is
an asserted one. The same false sentence stands in the doc comment on `Pramana::grade_ceiling`
(defect register).

**And the refusal is opt-in.** `exceeds_pramana_ceiling` compares a grade against a ceiling only
when the edge *declares* a pramāṇa; omit `via=` and a single settled edge sits at G4 unchallenged.
See defect 2.

---

## 4. Promotion

<a name="promotion"></a>

```mermaid
flowchart TB
    V["vault/ — markdown, hand-edited"] --> P["parse_node<br/>REFUSES status and confidence"]
    P --> G["Graph — nodes plus typed edges"]
    G --> E["examine_graph<br/>run every enforced gate"]
    E --> R{"GateResult"}
    R -->|"Pass"| OK["permits promotion"]
    R -->|"NotApplicable"| OK
    R -->|"Block"| BL["violation, blocks"]
    R -->|"Unassessed"| UN["no verdict reached<br/>MUST NOT permit promotion"]

    BL --> AGG["collected as violations"]
    UN -->|"PEIR-GATE-UNASSESSED"| AGG

    AGG --> LN["lint pack"]
    LN --> GE["grounded extension —<br/>withdrawn claims do not argue"]
    GE --> DS["derived state —<br/>reads gates and<br/>extension only"]
    DS --> FR["freeze a packet<br/>scans the rendered body"]

    X["was: dropped, carried no Violation<br/>the filter is now permits_promotion()"]
    UN -.-> X
    X:::mended
    classDef mended fill:#1a3a2a,stroke:#3a7,color:#fff,stroke-dasharray: 4 3
```

The dashed edge **was** the central defect, and it is the shape worth remembering rather than the
bug. `examine_graph` kept only `result.violation()`; `Unassessed` produces no violation, so it never
reached `freeze`, `status` or `gates`. The method encoding the rule — `GateResult::permits_promotion()`
— had **no production caller**: every call site was a test. A claim declaring no `uses_term`, no
`quantifier` and no `causal_rung` collected three `Unassessed` results, all of which vanished, and
the packet froze stating "All enforced gates pass".

**It is now the filter.** `examine_graph` emits a `PEIR-GATE-UNASSESSED` violation for every result
where `permits_promotion()` is false, so the rule reaches the decision point and every consumer
inherits it unchanged. The lesson generalises past this defect: *a state your types model can be
discarded in transit*, and a predicate whose only callers are tests is a rule you believe you
have.

---

## 5. Court Mode: freeze and verify

A packet is refused while any violation is attributed to the claim itself, and while the claim is
defeated in the grounded extension. There is no override flag. The filter is by subject: a
violation on a supporting node — a privilege leak, forbidden prose, a dangling edge — does not
block the claim's packet unless a claim-scoped gate independently attributes the defect to the
claim (defect 7). The safe statement is rendered from the graph in the 金剛經
three-moment form rather than authored — see
[`method/expert-witness.md`](method/expert-witness.md) for why.

```mermaid
stateDiagram-v2
    [*] --> Freezing
    Freezing --> Refused: not a claim
    Freezing --> Refused: a violation attributed to the claim
    Freezing --> Refused: defeated in the grounded extension
    Freezing --> Frozen: body rendered, sha256 sealed

    Frozen --> Verified: re-derives identically
    Frozen --> DigestMismatch: the rendered body changed under it
    Frozen --> FormatSuperseded: body declares another format
    Frozen --> NoLongerFreezable: a gate now blocks it

    note right of DigestMismatch
        The only outcome that is
        an accusation. Exit 1.
    end note
    note right of FormatSuperseded
        Exit 2 — the same code an
        absent vault returns.
        Could not look, not a verdict.
        An edited format line lands
        here too — defect 6.
    end note
```

Reusing exit 2 for *"could not reach a verdict"* is deliberate: it is already the code for an
absent vault, and the acceptance controls exist to prove *found nothing* is distinguishable from
*could not look*. The packet's format number is declared inside the body `freeze` hashes, so a
re-render costs a digest change — but that protects nothing at verification. The stored file
carries no digest of its own: `verify` recomputes one from whatever bytes are on disk, reads the
declared format from those same untrusted bytes, and returns `FormatSuperseded` before any
comparison when it differs. A hand edit that touches the format line is therefore reported as an
old renderer rather than as drift, and nothing distinguishes the two — the one accusatory verdict
is the one a tamperer can opt out of (defect 6).

---

## 6. Instruments: an address for the register, with no checks behind it

A recurring manufacturer of false findings is a source that answers *successfully with the wrong
thing* — the discipline, and the register it requires, are in
[`method/source-register.md`](method/source-register.md). peira gives that register an address in
the graph:

- an `instrument` node kind exists — a detector an observation came off: a tool, a query, a
  procedure someone else's software performs. Reference material, never an argument;
- a `measured_by:` edge exists, and the loader builds it, so an observation can name the
  instrument that produced it and the instrument's history travels with the evidence;
- an instrument's controls are ordinary frontmatter (`positive_control:`, `negative_control:`,
  or any key you choose), stored and queryable through the derived index.

**And that is all.** There is currently **no enforced check on any of it**: nothing requires an
observation to declare `measured_by:`, nothing requires an instrument to record a positive or a
negative control, and nothing counts refusals separately from zeros. The node kind is a place to
keep the register; the discipline of keeping it is, for now, entirely yours. The method page says
what to record and when.

---

## 7. How the claims in this document were tested

Two lineages, deliberately not one. Five agents from one model family share a blind spot and count
as **one method**; the outside critic is the second.

```mermaid
flowchart TB
    B["Adversarial brief —<br/>refute, do not review"]
    B --> A["5 attack lenses<br/>evasion, vacuity, court,<br/>architecture, completeness"]
    B --> C["Outside critic —<br/>different model family"]
    A --> J["5 judges vote 3-of-5<br/>reproduce, severity, fix-soundness,<br/>novelty, defence"]
    J --> S["steelman the survivors"]
    S --> RA["3 agents re-attack<br/>the proposed fixes"]
    C --> M["merge across lineages"]
    RA --> M
    M --> RE["reproduce against<br/>the built binary"]
```

**The panel saturated: 25 sustained, 0 rejected.** The threshold never bound, so this run provides
no evidence that a false finding would have been rejected — no negative control was seeded. Vote
splits carry what signal there is: 16 unanimous, 7 at 4-1, 2 at 3-2.

What makes the top findings credible is not the vote count but **convergence across independent
finders**, then direct reproduction:

| Defect | Lenses finding it independently | Outside critic | Reproduced |
|---|---|---|---|
| `Unassessed` dropped | 4 of 5 | yes | yes |
| pramāṇa ceiling opt-in | 5 of 5 | yes | yes |
| safe statement quotes authored prose | 3 of 5 | yes | mechanism confirmed in source; not yet run |

---

## Defect register

<a name="defect-register"></a>

Twenty-five findings survived adjudication; fifteen proposed fixes drew twenty-three holes on
re-attack, so **no fix set is ready to implement**. The five that matter most:

| # | Defect | Status |
|---|---|---|
| 1 | `Unassessed` is discarded at aggregation; a packet freezes over gates that reached no verdict, asserting "All enforced gates pass" — and `peira status` prints the same sentence over an empty blocking list. `permits_promotion()` has no production caller | **reproduced, FIXED** |
| 2 | The pramāṇa ceiling binds only edges that declare `via=`; omit it and one settled edge sits at G4 | **reproduced, FIXED** |
| 3 | `supersedes:` and `retracts:` edges are accepted, recorded and read by nothing — the structural synonym for the refused `status: withdrawn`. A withdrawn claim still freezes | **reproduced, FIXED** |
| 4 | Settled grades are operationally vacuous: an ungraded, unattributed edge supports promotion as well as reviewed perception. The `UNREVIEWED-GRADE` lint fires only on a *proposed* grade; a wholly ungraded edge trips nothing | **reproduced, FIXED** |
| 5 | The generated safe statement renders author-written `as_used`/`not_essence`/`stipulated` prose verbatim, and the forbidden-verb lint scans only a node's title and body — never those fields | **reproduced, FIXED** — the scan now reads the rendered body before sealing, so "rendered but unscanned" is impossible rather than enumerable |

Four smaller findings worth naming because of their shape:

- **`quantifier: all`** — an unrecognised spelling — makes the 白馬非馬 gate return
  `NotApplicable`. An unrecognised value silently disables a check instead of being shown
  verbatim, which is the repository's own stated rule for unrecognised input.
- A test comment in `core/src/edge.rs` states that the gates report an undeclared pramāṇa
  "separately as unassessed". **They do not** — `grades_within_pramana_ceiling` never returns
  `Unassessed`. A comment describing behaviour the code does not have.
- The crate doc comment in `lens/src/lib.rs` states that domain packs "depend down onto this
  crate". **No domain pack exists.** Unbuilt architecture stated in the present tense.
- The doc comment on `Pramana::grade_ceiling` in `core/src/edge.rs` states that encoding the
  ceiling "means a G4 can never be asserted, only earned". **It can be asserted, and nothing earns
  it** — the loader constructs a settled G4 edge directly, and no graph operation computes
  convergence. A comment describing behaviour the code does not have, in the same file as the one
  above.

### Established by a second outside critique (2026-08-13)

A critique of this document against the source, from a different model family than the five-lens
panel, established four more — each checked against the code it cites before being accepted here:

| # | Defect | Status |
|---|---|---|
| 6 | `verify` reads `Packet format:` from the stored body — untrusted input — and returns `FormatSuperseded`, exit 2, before any comparison when it differs. No digest is stored independently of the packet file, so a hand edit that touches the format line is reported as an old renderer, never as `DigestMismatch` | **confirmed in source; the crate's own test constructs exactly this edit and asserts `FormatSuperseded`** |
| 7 | `freeze` blocks only on violations whose subject is the claim being frozen. A defect on a supporting node — a privilege leak, forbidden prose, a dangling edge — stops nothing unless a claim-scoped gate re-attributes it to the claim | **confirmed in source** |
| 8 | The digest covers only the rendered projection. Grades, graders, pramāṇas and `measured_by:` links are not rendered, so they change in the vault without disturbing a frozen packet; the change surfaces only if it now trips a gate, as `NoLongerFreezable` — otherwise `Verified` | **confirmed in source** |
| 9 | The loader silently degrades malformed edge metadata: an unknown attribute key, an invalid `grade=` and a misspelt `via=` are dropped without a diagnostic. `via=percpetion` removes the pramāṇa ceiling; a mangled `grade=` removes review semantics and trips no lint — the same shape as the `quantifier: all` finding above, one layer down | **confirmed in source** |

### The second audit, and what it changed about method

The first audit's panel **sustained 25 of 25 findings** — the 3-of-5 threshold never bound, so
nothing established that a bad finding would have been rejected. The second seeded a plausible
falsehood into the docket and told the judges one was fabricated. It was rejected **0-5**, every
judge refuting it by running the binary. That number is why the second round's findings carry
weight the first round's counts did not.

**All three of the previously-open findings are now closed**, and the closing changed two of them:

| Was open | Outcome |
|---|---|
| A support hop through a `hypothesis` reaches fewer gates | **fixed** — promotion obligations attach to being *load-bearing*, not to the node kind. A hypothesis nothing leans on is still a candidate; one supporting a claim answers for itself |
| `verify` cannot know about a retraction added after freezing | **was already closed** by the evidential-closure walk, and this document was stale. `verify` re-derives from the live vault, so it reports "the claim no longer qualifies" and names the retraction. The *packet* is ignorant; the *tool* is not |
| The forbidden-verb list is finite | **the panel rejected this 1-4 and the panel was half right.** A denylist being finite is not a defect. But *"the suspect is guilty of unauthorised access"* contains no forbidden verb, passed every check, and froze into a packet — so `PEIR-LINT-LEGAL-CONCLUSION` now catches the ultimate issue, which unlike overstatement is a closed vocabulary |

The layer-3 lint uses **sentence-level** negation rather than the verb lint's short lookback, and the
trade costs real coverage: *"X is guilty, and nothing contradicts it"* is missed because it contains
a negator. That is the right direction for a check that blocks — a false positive here refuses the
carefully hedged sentence an expert is obliged to write, and what it misses was caught by nothing
before.

**Only one lineage ran the second round.** The outside critic was blocked by its provider's safety
classifier — the session shape (an agent building a tool, then constructing inputs to defeat its
checks) reads as offensive tooling regardless of wording. Its transcript shows it started and was
killed mid-run, so that is a refusal, not a null result, and these findings lack the cross-lineage
convergence the first round's had.

### What this does not overturn

The type-level invariants hold as designed and were not falsified: the parser does refuse `status`
and `confidence`; a grade genuinely cannot be constructed without a grader; there is no override
flag on freezing; the checker runs no model. The audit's finding is narrower and worse than "the
design is wrong" — **the design is sound and the pipeline discards it.** Every defect above is a
correct rule losing its force at a join.
