# Vibe Hacking Village — Vibe Research Core

**Working proposition:** Bring the science back.

## Thesis

Vibe Research is not another deep-research chatbot. It is **epistemic CI/CD for technical knowledge**:

- hypotheses are branches;
- test protocols and source reviews are CI runs;
- observations and passages are evidence objects;
- atomic claims are mergeable units;
- contradiction is an issue, not an embarrassment;
- independent replication is a merge gate;
- supersession creates a new version rather than overwriting history;
- a court packet is a signed release artifact.

Perplexity optimizes time-to-answer. Cowork optimizes time-to-artifact. Vibe Research should optimize **time-to-defensible-claim**.

## What to learn from existing products

### Perplexity

Copy:

- explicit search/query events;
- iterative research with visible progress;
- citation IDs resolving to separate source records;
- source, domain, language and date controls;
- raw retrieval kept separate from synthesis;
- project workspaces, background jobs and structured outputs.

Do not copy the implication that a citation verifies a claim. Perplexity's audited public contract exposes URLs, source metadata, snippets, query/run data and inline citation pointers, but did not establish exact claim-to-passage locators, immutable source snapshots or hashes, source-version identity, native claim status/confidence, contradiction review, chain of custody or signed exports.

### Claude Cowork

Copy:

- explicit decomposition into bounded parallel workstreams;
- full task transcripts and produced-file inspection;
- local folder-backed contexts;
- reusable skills/plugins/connectors;
- human permission gates;
- operational telemetry as supplementary process evidence.

Do not use a Cowork project as the canonical shared evidence ledger. Official documentation says Cowork projects are local and non-shareable. Monitoring can be metadata-only and content/tool fields may be truncated. The audited public documentation did not establish claim-level provenance, evidence grading, contradiction state, calibrated confidence or a complete lossless audit export.

### Product distinction

- Perplexity shows **where an answer looked**.
- Cowork shows **what an agent did**.
- Vibe Research must show **why a claim may be stated, with what strength, under which boundaries, and what would change it**.

## Core architecture

1. **Capture layer**
   - Exact source bytes or lawful preservation record
   - UTC retrieval time, redirect chain, headers, version/commit/edition
   - Cryptographic digest and content-addressed storage
   - Test inputs, outputs, logs and environment manifests

2. **Evidence graph**
   - Research question
   - Competing hypotheses
   - Atomic claims
   - Source versions and exact passages
   - Test protocols/runs and observations
   - Support, contradiction, limitation, duplication and dependency edges
   - Boundary conditions and independence groups

3. **Scientific workbench**
   - Predeclared falsification criteria
   - Reproducible tests and negative controls
   - Contradiction search
   - Two-person review and structured dissent
   - Supersession/retraction without historical erasure

4. **Agent orchestration**
   - Separate discovery, contradiction, test-design and citation-verification agents
   - Protocol and scope frozen before autonomous work
   - AI-generated prose resolves to existing claim IDs
   - AI cannot create observations, citations or test results without inspectable source material and cannot upgrade confidence

5. **Court Mode**
   - Immediate claim-centered citation packet
   - Safe statement, support and contradiction
   - Exact quotation/locator and source version
   - Artifact and source hashes
   - OS/parser/applicability boundaries
   - Test method, result and deviations
   - Expert inference and unresolved alternatives
   - Reviewer/signature and supersession state

Court Mode should use pre-frozen packets, not depend on live web search during testimony. Live retrieval may discover updates, but new material remains provisional until captured and reviewed.

## Keep five grades separate

1. **Investigative severity:** operational priority or potential impact.
2. **Claim state:** draft, evidence-pending, accepted, contested, rejected, superseded or retracted.
3. **Source quality:** authority, authenticity, directness, method, independence, applicability and stability.
4. **Evidence grade:** strength of a particular evidence-to-claim edge.
5. **Claim confidence:** reviewed assessment of the whole current graph, with rationale and alternatives.

Suggested edge grades:

- `G0`: unsupported assertion or unverified pointer
- `G1`: relevant pinned passage or observation
- `G2`: directly applicable evidence with method and provenance
- `G3`: reproducible test or independently verified observation
- `G4`: multiple materially independent convergent lines with boundaries addressed

A severity of High does not imply high epistemic confidence. Citation count does not imply independence.

## Knowledge lifecycle

```text
Question
→ competing hypotheses
→ atomic claims
→ source passage / test protocol
→ observation
→ support / contradiction / limitation edge
→ reviewer assessment
→ citation packet
→ sealed court export
→ monitored supersession
```

Useful claim states:

```text
draft → atomized → evidence_pending → review_ready
review_ready → accepted | contested | rejected
accepted | contested | rejected → superseded
accepted → retracted
```

`Verified` should never mean eternally true; all acceptance is versioned and scoped.

## Amcache vertical slice

Current direct project source:

- Repository: https://github.com/SecurityRonin/amcache-forensic
- Audited commit: `a0a75b657893eb589f031e22d6838d896ecee74b`
- README states: “Amcache is evidence of presence, not proof of execution.”
- Validation uses real hives across four Windows systems and two independent parsing oracles.

That validation supports parser correctness and cross-parser agreement. It does not by itself establish every Windows population mechanism or prove execution semantics.

### Claim graph

- **Observation:** acquired `Amcache.hve` has digest H.
- **Observation:** parser P at commit V emitted path X and SHA-1 Y.
- **Observation:** manual verification located the corresponding registry value/bytes.
- **Supported claim:** Windows catalogued the file identity/path represented by that version-specific record.
- **Hypothesis:** the record was produced by execution.
- **Competing hypothesis:** installation, inventorying or scanning produced the record without user execution.
- **Bounded conclusion:** the Amcache record establishes catalogued presence and may contribute to an execution inference when combined with independent, version-appropriate evidence.

### Controlled test matrix

Across pinned Windows builds and repeatable VM snapshots:

- clean baseline;
- copy/introduction without launch;
- installation or application-inventory activity;
- antimalware/scanning activity where applicable;
- explicit execution;
- deletion after introduction;
- pre/post collection of Amcache and independent corroborative artifacts;
- manual structure review plus two pinned parsers;
- negative controls and repeated runs.

Every run preserves environment manifest, commands/actions, artifact hashes, logs, deviations and alternative background mechanisms. “The operator did not double-click it” is not a sufficient control.

### Existing semantic risk to review

`AMCACHE-SUSPICIOUS-PATH` currently renders “consistent with suspicious execution,” while the surrounding design correctly caps Amcache at presence. A safer claim-centered rendering is:

> Amcache recorded the file at a path commonly associated with staging. This raises investigative priority; assess execution through independent, version-appropriate evidence.

This preserves severity without silently upgrading presence into execution.

## Village mechanics

- **Claim Jam:** convert folklore and tool labels into atomic, scoped propositions.
- **Replication Bounties:** reproduce a decisive test on another OS build/toolchain.
- **Contradiction Bounties:** reward counterexamples and alternative population mechanisms.
- **Citation Rescue:** replace mutable or secondary references with pinned primary passages.
- **Boundary Mapping:** identify versions/configurations where a claim changes.
- **Negative Result Credit:** reward valid falsification and inconclusive findings.
- **Courtroom Challenge:** answer a hostile question from a frozen packet within 90 seconds, including qualification and exact citation.
- **Supersession Drill:** introduce a parser defect or new OS build and identify only affected downstream claims.

Reputation attaches to validated contribution types—source capture, citation verification, protocol design, replication, contradiction, boundary discovery and review—not to majority votes on what is true.

## Minimum MVP

- PostgreSQL claim/evidence graph
- Content-addressed S3/MinIO object store
- Append-only audit events
- JSON Schema records and validator
- Worker queue for capture, hashing, citation checks, test execution and export
- One web interface with claim card, evidence edges, contradiction view and Court Mode
- One vertical domain: modern Windows Amcache

Do not begin with general-purpose chat. Begin with one hard acceptance test:

> Given the claim “this Amcache entry proves execution,” can the system immediately show the precise observation, alternative mechanisms, version boundaries, supporting and contradicting evidence, exact passages, reproducible tests, reviewer state and courtroom-safe formulation?

## Village manifesto

1. Every vibe begins as a hypothesis.
2. Every conclusion resolves to atomic claims.
3. Every citation is inspectable and versioned.
4. Evidence and inference remain distinct.
5. Contradictions and failed tests are first-class contributions.
6. Scope and boundary conditions are mandatory.
7. Independent tools are not automatically independent evidence.
8. AI may discover, organize and draft; it may not manufacture evidence or elevate confidence.
9. Released knowledge is immutable and superseded transparently.
10. In Court Mode, provenance is one click away.

## Official product sources audited

### Perplexity

- https://www.perplexity.ai/help-center/en/articles/10352895-how-does-perplexity-work.html
- https://www.perplexity.ai/help-center/en/articles/10738684-what-is-research-mode
- https://docs.perplexity.ai/docs/cookbook/articles/streaming-citations/README
- https://docs.perplexity.ai/api-reference/sonar-post
- https://docs.perplexity.ai/docs/agent-api/output-control
- https://www.perplexity.ai/help-center/en/articles/10352961-what-are-spaces.html

### Anthropic / Claude

- https://claude.com/docs/cowork/overview
- https://claude.com/docs/cowork/guide/dispatch
- https://claude.com/docs/cowork/guide/projects
- https://claude.com/docs/cowork/guide/plugins
- https://claude.com/docs/cowork/monitoring
- https://support.claude.com/en/articles/13364135-use-claude-cowork-safely
- https://support.claude.com/en/articles/11088861-use-research-on-claude

**Audit date:** 2026-08-08. Negative product-capability findings mean “not found in the accessible official public contract,” not a universal claim about private or future capabilities.
