# Feature Comparison Matrix: Legal AI Market vs. murdock

**Date:** 2026-05-05  
**Purpose:** Competitive analysis informing murdock product design  
**Sources:** GitHub repo analysis (willchen96/mike, veronica-builds/emilie, rafal-fryc/mikelocal) + web research

---

## Market Snapshot

| Player | Valuation | ARR | Price/seat/mo | Min seats | Target |
|--------|-----------|-----|---------------|-----------|--------|
| Harvey | $11B | $190M | $1,200–2,000 | 20 | Am Law 100, BigLaw |
| Legora | $5.6B | $100M+ | $250 | 10 | EU large firms |
| CoCounsel (TR) | (RELX) | N/A | $100–200 | 1 | Westlaw subscribers |
| Lexis+ AI | (RELX) | N/A | $128–494 | 1 | LexisNexis subscribers |
| Spellbook | Private | N/A | $99–350 | 1 | In-house, contracts |
| Luminance | Private | N/A | ~$50K–250K/yr | Firm | Enterprise, 70+ countries |
| Mike (OSS) | N/A | N/A | Free (BYO API) | 1 | Law firms (self-host) |
| **murdock** | — | — | Free (CC session) | 1 | Solo barristers |

---

## Core Feature Matrix

| Feature | Harvey | Legora | Mike OSS | **murdock** |
|---------|:------:|:------:|:--------:|:----------:|
| **Interface** | Web | Web + Word | Web | **Terminal / TUI** |
| **Document chat + citations** | ✅ | ✅ | ✅ | ✅ |
| **Tabular / bulk extraction** | ✅ | ✅ (signature) | ✅ | ✅ |
| **DOCX tracked changes** | ✅ (Apr 2026) | ✅ | ✅ | ✅ |
| **Document generation** | ✅ | ✅ | ✅ | ✅ |
| **Workflow templates** | ✅ (25K+) | ✅ | ✅ (3 built-in) | ✅ (skills) |
| **Agentic multi-step** | ✅ | ✅ | ❌ | ✅ (CC session) |
| **Case law research** | ✅ (200+ sources) | ✅ (Qura, 27 juris) | ❌ | ✅ (HKLII + BAILII) |
| **Web research** | ❌ | ❌ | ❌ | ✅ (WebSearch) |
| **MCP integration** | ❌ | ✅ | ✅ (Emilie fork) | ✅ |
| **DMS integration** | ✅ (iManage, NetDoc) | ✅ | ❌ | Local FS (Phase 1) |
| **Word Add-in** | ✅ | ✅ | ❌ | ❌ (terminal) |
| **Mobile** | ✅ | ❌ | ❌ | ❌ |
| **Offline / local** | ❌ | ❌ | ✅ (MikeLocal) | ✅ |
| **Open source** | ❌ | ❌ | ✅ (AGPL-3.0) | ✅ |
| **Self-hosted / no cloud** | ❌ | ❌ | ✅ | ✅ |
| **BYO API keys** | ❌ | ❌ | ✅ | ✅ (CC session) |
| **Zero API cost for reasoning** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Free trial / self-serve** | ❌ | ❌ | ✅ | ✅ |
| **Min 1 seat** | ❌ (min 20) | ❌ (min 10) | ✅ | ✅ |

---

## Legal Domain Focus

| Capability | Harvey | Legora | Mike OSS | **murdock** |
|-----------|:------:|:------:|:--------:|:----------:|
| **M&A / transactional** | ✅ | ✅ | ✅ | ❌ (not focus) |
| **Contract review / CLM** | ✅ | ✅ | ✅ | ❌ (not focus) |
| **Litigation support** | ✅ | Partial | Partial | ✅ ← primary |
| **Regulatory research** | ✅ | ✅ | ❌ | ✅ |
| **Tax / EDGAR** | ✅ | ✅ | ❌ | ❌ |
| **Barrister-specific workflow** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Skeleton arguments** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Opinions / advice** | Partial | Partial | Partial | ✅ ← focus |
| **Brief analysis (instructions)** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Pleadings (statement of claim)** | Partial | Partial | Partial | ✅ ← focus |
| **Written submissions** | Partial | Partial | Partial | ✅ ← focus |
| **Expert witness reports** | ❌ | ❌ | ❌ | ✅ ← unique |

---

## Jurisdiction & Language

| Capability | Harvey | Legora | Mike OSS | **murdock** |
|-----------|:------:|:------:|:--------:|:----------:|
| **US law depth** | ✅✅ (custom model) | ✅ (via Qura) | ❌ | ❌ |
| **UK / England & Wales** | ✅ | ✅ | ❌ | ✅ |
| **Hong Kong law (HKSAR)** | ❌ | ❌ | ❌ | ✅ ← unique |
| **BAILII case law** | ✅ | ✅ | ❌ | ✅ |
| **HKLII case law** | ❌ | ❌ | ❌ | ✅ ← unique |
| **HK Legislation (e-Legis)** | ❌ | ❌ | ❌ | ✅ ← unique |
| **PRC cross-border nexus** | ❌ | ❌ | ❌ | ✅ ← unique |
| **CJKV multilingual** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Traditional Chinese** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Bilingual proceedings** | ❌ | ❌ | ❌ | ✅ ← unique |
| **EU law (CJEU)** | ✅ | ✅ (native) | ❌ | ❌ (Phase 2) |
| **Multi-jurisdiction** | 60+ countries | 50+ markets | Agnostic | HK + UK (Phase 1) |

---

## DFIR / e-Discovery (murdock's Unchallengeable Pivot)

| Capability | Harvey | Legora | Relativity | **murdock** |
|-----------|:------:|:------:|:----------:|:----------:|
| **ESI processing** | ❌ | ❌ | ✅ | ✅ |
| **EML / PST ingestion** | ❌ | ❌ | ✅ | ✅ |
| **Hash manifest verification** | ❌ | ❌ | Partial | ✅ ← unique |
| **Forensic timeline analysis** | ❌ | ❌ | Partial | ✅ ← unique |
| **Log file analysis** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Chain of custody docs** | ❌ | ❌ | ❌ | ✅ ← unique |
| **CJKV metadata in files** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Privilege review** | ✅ | Partial | ✅ | ✅ |
| **Review coding (relevant/irrelevant)** | Partial | Partial | ✅ | ✅ |
| **Review protocol drafting** | ❌ | ❌ | ❌ | ✅ |
| **Expert witness report (CPR Pt 35)** | ❌ | ❌ | ❌ | ✅ ← unique |
| **HK O.38 r.37A expert format** | ❌ | ❌ | ❌ | ✅ ← unique |
| **Chronology / timeline memo** | Partial | ❌ | ❌ | ✅ |
| **Forensic annotation in docs** | ❌ | ❌ | ❌ | ✅ ← unique |
| **PCAP / network log parsing** | ❌ | ❌ | ❌ | ✅ (Phase 2) |

---

## Security & Deployment

| Capability | Harvey | Legora | Mike OSS | **murdock** |
|-----------|:------:|:------:|:--------:|:----------:|
| **SOC 2 Type II** | ✅ | ✅ | Self-host | Self-host |
| **ISO 27001** | ✅ | ✅ | Self-host | Self-host |
| **ISO 42001 (AI governance)** | ❌ | ✅ | ❌ | — |
| **GDPR** | ✅ | ✅ (native) | ❌ | ❌ (local) |
| **PDPO (HK)** | ❌ | ❌ | ❌ | ✅ ← aware |
| **SAML 2.0 SSO** | ✅ | ✅ | ❌ | ❌ |
| **Data residency** | US/EU/UK | Regional | User-controlled | Local machine |
| **Zero data to vendor** | ✅ (contractual) | ✅ (contractual) | N/A | ✅ (local) |
| **Attorney-client privilege** | Contractual | Contractual | — | Local = protected |
| **Air-gapped / offline** | ❌ | ❌ | ✅ (MikeLocal) | ✅ |

---

## Architecture & Extensibility

| Capability | Harvey | Legora | Mike OSS | **murdock** |
|-----------|:------:|:------:|:--------:|:----------:|
| **Custom LLM** | ✅ (OpenAI fine-tune) | ❌ | ✅ (OpenRouter) | Claude Code session |
| **MCP servers** | ❌ | ✅ | ✅ (Emilie) | ✅ |
| **Plugin / extension** | ✅ (Workflow Builder) | Partial | ❌ | ✅ (skills) |
| **API access** | ✅ | ✅ | ✅ | ✅ |
| **CLI / terminal** | ❌ | ❌ | ❌ | ✅ ← core |
| **Codex compatible** | ❌ | ❌ | ❌ | ✅ ← goal |
| **Skills as slash commands** | ❌ | ❌ | ❌ | ✅ ← core |
| **MD output → Typora render** | ❌ | ❌ | ❌ | ✅ ← workflow |
| **Line-drawing file trees** | ❌ | ❌ | ❌ | ✅ ← core |
| **Git-native versioning** | ❌ | ❌ | ❌ | ✅ |
| **Self-hostable** | ❌ | ❌ | ✅ | ✅ |

---

## murdock's Unique Positioning Summary

**murdock** occupies a completely unclaimed market position at the intersection of:

1. **Barrister workflow** — no existing tool is built for chambers practice (instructions → opinions → pleadings → court)
2. **HK + London dual jurisdiction** — HKLII + BAILII + HK e-Legislation, bilingual proceedings
3. **CJKV-first** — Traditional Chinese as a core language, not an afterthought
4. **DFIR + e-Discovery expert system** — forensic artifact ingestion, chain of custody, expert reports in CPR/HK format
5. **Terminal-native** — Claude Code paradigm, zero extra API cost (uses CC session reasoning)
6. **Skills architecture** — works inside Claude Code AND Codex, modular and composable
7. **Solo-friendly** — no minimums, no sales process, no vendor lock-in

**No competitor addresses even two of these simultaneously.** murdock addresses all seven.
