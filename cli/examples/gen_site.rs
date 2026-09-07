//! Site generator — emits the Jekyll lens reference pages from the live catalogue.
//!
//! ```console
//! cargo run -p peira-cli --example gen_site -- docs
//! ```
//!
//! It reads [`peira_lens::CATALOG`] in-process, so the pages it writes cannot drift
//! from the code: a lens edited in the catalogue is a lens re-rendered here on the next
//! run. The committed output is locked by a CI drift-check (regenerate, then
//! `git diff --cached --quiet`), the same shape as `cargo fmt --check`.
//!
//! Presentation — the English display name and the original script — lives in this
//! generator, not the catalogue: the catalogue owns substance (failure mode, gates,
//! sources), the site owns how it is shown. Romanization is never a headline; the
//! English name leads and the script is etymology.

use peira_lens::{Lens, Phase, Tradition, CATALOG};
use std::fmt::Write as _;
use std::{fs, path::PathBuf, process::ExitCode};

/// English display name and original script for a lens code. Panics on an unmapped
/// code so that adding a lens without naming it fails loudly here (and in CI) rather
/// than shipping a page with an empty title.
fn display(code: &str) -> (&'static str, &'static str) {
    match code {
        "CRITERION" => ("Set the Pole", "立極"),
        "RECTIFY-NAME" => ("Rectification of Names", "正名"),
        "SUBSTANCE-FUNCTION" => ("Substance and Function", "體用"),
        "WHITE-HORSE" => ("The White Horse Is Not a Horse", "白馬非馬"),
        "FOUR-CORNERS" => ("The Four Corners", "四句"),
        "TOULMIN" => ("Name the Warrant", ""),
        "MEANS-OF-KNOWING" => ("The Means of Knowing", "प्रमाण"),
        "RUNG" => ("Earn the Rung", ""),
        "CROSS-EXAMINE" => ("Socratic Cross-Examination", "ἔλεγχος"),
        "ACH" => ("Analysis of Competing Hypotheses", ""),
        "FIVE-MEMBERS" => ("The Five-Membered Argument", "पञ्चावयव"),
        "STEELMAN" => ("Steelman First", ""),
        "DOUBLECRUX" => ("Double Crux", ""),
        "PRESERVE-MINORITY" => ("Preserve the Minority", "מחלוקת"),
        "SYNTHESIS" => ("Synthesis That Preserves", ""),
        "THESEUS" => ("Ship of Theseus — Amend or Supersede", ""),
        "CHESTERTON" => ("Chesterton's Fence", ""),
        "PREMORTEM" => ("Premortem / Inversion", ""),
        "THREE-MARKS" => ("The Three Marks of a Valid Reason", "因三相"),
        "NON-PERCEPTION" => ("Non-Perception as a Reason", "不可得因"),
        "SEMBLANCE" => ("The Semblances of Proof", "似因・似宗"),
        "TWO-TRUTHS" => ("The Two Truths", "二諦"),
        "DUNG" => ("Grounded Extension — Compute, Don't Assert", ""),
        "KNOW-BY-DOING" => (
            "Investigate Each Thing; Knowing Proven in Doing",
            "格物致知・知行合一",
        ),
        "LACUNA" => ("The Dog That Didn't Bark", "闕文"),
        "IDOLA" => ("Bacon's Idols of the Mind", ""),
        "BLACKSTONE" => ("The Asymmetry of Error", ""),
        other => panic!("no display name for lens code `{other}` — add it to gen_site.rs"),
    }
}

/// The tradition, spelled for a reader rather than as the enum's short label.
fn tradition_long(t: Tradition) -> &'static str {
    match t {
        Tradition::Greek => "Greek — Socratic and Aristotelian",
        Tradition::Chinese => "Chinese — 名家, 宋明理學, and the classics",
        Tradition::Indian => "Indian — classical logic and the epistemology of प्रमाण",
        Tradition::Buddhist => "Buddhist — 中觀 and 因明 (Buddhist logic)",
        Tradition::Jewish => "Jewish — Talmudic dispute",
        Tradition::Modern => "Modern — analytic and scientific method",
        Tradition::Formal => "Formal — argumentation theory",
        other => panic!("unhandled tradition {other:?} — add it to gen_site.rs"),
    }
}

/// Render one source line: a citation stays as text; a bare URL becomes an autolink; a
/// URL followed by an annotation (`https://… (論語·子路…)`) becomes a proper link whose
/// text is the annotation — never a broken autolink with a space inside.
fn source_line(s: &str) -> String {
    if s.starts_with("http://") || s.starts_with("https://") {
        match s.split_once(char::is_whitespace) {
            Some((url, rest)) => {
                let text = rest.trim().trim_start_matches('(').trim_end_matches(')');
                format!("- [{text}]({url})")
            }
            None => format!("- <{s}>"),
        }
    } else {
        format!("- {s}")
    }
}

/// Double-quote and escape a YAML front-matter string value.
fn yaml(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn lens_page(lens: &Lens) -> String {
    let (english, script) = display(lens.id);
    let enforced = lens.phase == Phase::Enforced;
    let mut out = String::new();

    out.push_str("---\nlayout: lens\n");
    let _ = writeln!(out, "title: {}", yaml(english));
    let _ = writeln!(out, "code: {}", yaml(lens.id));
    let _ = writeln!(out, "script: {}", yaml(script));
    let _ = writeln!(out, "tradition: {}", yaml(lens.tradition.as_str()));
    let _ = writeln!(
        out,
        "phase: {}",
        if enforced { "Enforced" } else { "Catalogued" }
    );
    out.push_str("nav_exclude: true\n---\n\n");

    out.push_str(
        "<!-- Generated from peira_lens::CATALOG by cli/examples/gen_site.rs. Do not edit by hand: edit the lens in lens/src/lib.rs and regenerate. -->\n\n",
    );

    if script.is_empty() {
        let _ = writeln!(out, "# {english}\n");
    } else {
        let _ = writeln!(out, "# {english} <span class=\"script\">{script}</span>\n");
    }
    let _ = writeln!(
        out,
        "<p class=\"lens-meta\"><code class=\"lens-code\">{}</code> · {} · <strong>{}</strong></p>\n",
        lens.id,
        tradition_long(lens.tradition),
        if enforced { "Enforced — owns a gate" } else { "Catalogued — a reading, owns no gate" }
    );

    let _ = writeln!(out, "## The failure it names\n\n{}\n", lens.failure_mode);
    let _ = writeln!(out, "## What it does\n\n{}\n", lens.operation);

    if enforced && !lens.gates.is_empty() {
        out.push_str("## Enforced gates\n\nThese block a citation packet deterministically — no model in the loop:\n\n");
        for g in lens.gates {
            let _ = writeln!(out, "- `{}`", g.code);
        }
        out.push('\n');
    }

    let _ = writeln!(out, "## Worked example\n\n{}\n", lens.worked_example);

    if !lens.background.is_empty() {
        let _ = writeln!(out, "## In the tradition\n\n{}\n", lens.background);
    }

    out.push_str("## Sources\n\n");
    for s in lens.sources {
        let _ = writeln!(out, "{}", source_line(s));
    }
    out.push('\n');

    out.push_str(
        "<p class=\"back\"><a href=\"{{ '/lenses/' | relative_url }}\">← All lenses</a> · <a href=\"{{ '/doors/' | relative_url }}\">The seven doors</a></p>\n",
    );
    out
}

/// The catalogue index, grouped Enforced then Catalogued, each by tradition.
fn index_page() -> String {
    let mut out = String::new();
    out.push_str("---\nlayout: default\ntitle: The lens catalogue\nnav_order: 3\n---\n\n");
    out.push_str("<!-- Generated by cli/examples/gen_site.rs. Do not edit by hand. -->\n\n");
    out.push_str("# The lens catalogue\n\n");
    let _ = writeln!(
        out,
        "Each lens names a **specific way of being wrong** that a critical-thinking tradition \
identified. {} are catalogued; {} are **enforced** as deterministic gates today, and the rest are \
**catalogued** — named, sourced, and given a worked example, a reading for what to ask by hand.\n",
        CATALOG.len(),
        CATALOG
            .iter()
            .filter(|l| l.phase == Phase::Enforced)
            .count()
    );

    for (phase, heading, blurb) in [
        (
            Phase::Enforced,
            "Enforced — these block a claim",
            "A gate that fails on a known-bad input, run before the conclusion is trusted.",
        ),
        (
            Phase::Catalogued,
            "Catalogued — these are read by hand",
            "Specified and sourced, owning no gate; a checklist for what a machine cannot settle.",
        ),
    ] {
        let _ = writeln!(out, "## {heading}\n\n{blurb}\n");
        out.push_str("| Lens | Tradition | peira code |\n|---|---|---|\n");
        let mut rows: Vec<&Lens> = CATALOG.iter().filter(|l| l.phase == phase).collect();
        rows.sort_by_key(|l| (l.tradition.as_str(), display(l.id).0));
        for l in rows {
            let (english, script) = display(l.id);
            let name = if script.is_empty() {
                english.to_owned()
            } else {
                format!("{english} <span class=\"script\">{script}</span>")
            };
            let _ = writeln!(
                out,
                "| [{name}]({{{{ '/lenses/{}/' | relative_url }}}}) | {} | `{}` |",
                l.id.to_lowercase(),
                l.tradition.as_str(),
                l.id
            );
        }
        out.push('\n');
    }
    out
}

/// Link every `PEIR-…` gate/lint code to the enforcement table. These are not lenses,
/// so they own no page; the table in the method doc documents each one. A plain backtick
/// scan — no regex dependency.
fn link_gate_codes(body: &str) -> String {
    const ANCHOR: &str =
        "{{ '/method/six-structures/' | relative_url }}#what-peira-actually-enforces";
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("`PEIR-") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..]; // past the opening backtick
        if let Some(end) = after.find('`') {
            let code = &after[..end];
            let _ = write!(out, "[`{code}`]({ANCHOR})");
            rest = &after[end + 1..]; // past the closing backtick
        } else {
            out.push_str(&rest[start..]);
            return out;
        }
    }
    out.push_str(rest);
    out
}

/// The seven-doors page, compiled from the canonical method doc so it cannot drift:
/// front matter is prepended, and every backtick-wrapped lens code becomes a link.
fn doors_page() -> String {
    let mut body = include_str!("../../docs/method/anti-summarization.md").to_owned();

    // Routes on their own line: stack each route in the seven-doors table's last column
    // as a bulleted line. Done BEFORE linkify, on the raw backticked cell, so the comma
    // inside an English name ("Compute, Don't Assert") is never mistaken for a separator.
    body = body
        .lines()
        .map(|line| {
            let is_door_row = line
                .strip_prefix("| ")
                .and_then(|r| r.chars().next())
                .is_some_and(|c| c.is_ascii_digit())
                && line.matches('|').count() == 5;
            if !is_door_row {
                return line.to_owned();
            }
            let cells: Vec<&str> = line.trim_matches('|').split(" | ").collect();
            if cells.len() != 4 {
                return line.to_owned();
            }
            let routes = cells[3]
                .trim()
                .split(", ")
                .map(|r| format!("• {r}"))
                .collect::<Vec<_>>()
                .join("<br>");
            format!(
                "| {} | {} | {} | {routes} |",
                cells[0].trim(),
                cells[1].trim(),
                cells[2].trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // The canonical doc references lenses by their peira code (`RECTIFY-NAME`), the
    // identifier the CLI and packets print. On the site the code is never a label: the
    // link text is the English name, followed by the lens's own script in its native form
    // (देवनागरी, 漢字, Ελληνικά, עברית) — never a Latin romanization. The generator owns the
    // script so it is consistent across every lens, not ad-hoc per doc line.
    for lens in CATALOG {
        let (name, script) = display(lens.id);
        let needle = format!("`{}`", lens.id);
        let slug = lens.id.to_lowercase();
        let link = if script.is_empty() {
            format!("[{name}]({{{{ '/lenses/{slug}/' | relative_url }}}})")
        } else {
            format!(
                "[{name}]({{{{ '/lenses/{slug}/' | relative_url }}}}) <span class=\"script\">{script}</span>"
            )
        };
        body = body.replace(&needle, &link);
    }
    // Gate and lint codes are not lenses, so they own no page — link every `PEIR-…` code
    // to the enforcement table, where each gate and lint is documented.
    body = link_gate_codes(&body);
    body = body
        .replace(
            "](six-structures.md)",
            "]({{ '/method/six-structures/' | relative_url }})",
        )
        .replace(
            "](claim-grading.md)",
            "]({{ '/method/claim-grading/' | relative_url }})",
        );

    let mut out = String::new();
    out.push_str("---\nlayout: default\ntitle: The seven doors\nnav_order: 2\n---\n\n");
    out.push_str(
        "<!-- Generated from docs/method/anti-summarization.md by cli/examples/gen_site.rs. Do not edit by hand. -->\n\n",
    );
    out.push_str(&body);
    out.push('\n');
    out
}

fn write_file(path: &std::path::Path, body: &str) -> std::io::Result<()> {
    fs::write(path, body)
}

// ── The .claude/skills/peira Claude Code skill, generated from the catalogue ──────
//
// The doer-guidance prose is authored (it is not catalogue data); the lens table and
// the per-lens playbooks are compiled from CATALOG so they cannot drift. `{TABLE}` is
// the only interpolation point.

const SKILL_DIR: &str = ".claude/skills/peira";

const SKILL_PREAMBLE: &str = r#"---
name: peira
description: Examine a claim in a peira vault through the classical critical-thinking lenses — 立極, 正名, 體用, 白馬非馬, 四句, Toulmin, प्रमाण, and the causal ladder — and write examination artifacts that propose nodes and edges. Use when a claim needs cross-examining before it is relied on, when `peira gates` blocks and the fix is not obvious, or when turning folklore into a scoped proposition. Triggers - "examine this claim", "cross-examine", "is this over-claimed", "run the lenses", "why is this blocked", "make this court-safe".
---

<!-- Generated by cli/examples/gen_site.rs from peira_lens::CATALOG. The prose is authored;
     the lens table and lenses/*.md playbooks are compiled — edit the catalogue, not this file. -->

# peira — examine a claim

## What you may and may not do

You are the **doer**. The CLI is the **checker**. That division is not advice; it is
built into the data model, and you should not try to work around it.

**You may:**
- create and edit `question`, `hypothesis`, `claim`, `observation`, `term`,
  `criterion`, `protocol`, `run`, `examination` and `dissent` nodes
- add edges between them
- propose an evidence grade with `proposed=G2` on an edge
- write an `examination` node recording what you found

**You may not:**
- write `status:` or `confidence:` on any node. There is no such field. The parser
  refuses the document and names the key. Do not attempt it.
- write `grade=` without `by=`. An unattributed grade silently degrades to a
  proposal, and the lint pack reports it.
- declare that a claim is accepted, verified, or cleared. Only
  `peira status` says that, and it derives the answer.

If you catch yourself wanting to assert a verdict, that is the feeling the design
exists to produce. Write the evidence instead.

## Workflow

1. **Read the state first.** Never begin by proposing fixes.

   ```bash
   peira gates <vault> --node <id>
   peira status <vault> <id>
   ```

   Every blocking finding names its gate, what was actually found, and a remedy.
   Start from what the engine already told you.

2. **Pick the lenses that the blocks point to.** Each enforced gate belongs to a lens,
   and each lens has a playbook in `lenses/`:

{TABLE}

   `peira lens <ID>` prints the failure mode, the operation and a worked example for
   any lens; every lens in the catalogue has a playbook in `lenses/`, including the
   catalogued ones that own no gate.

3. **Do the examination.** Follow the playbook. It will tell you what to look for
   and what to write.

4. **Write an examination node** into `80-examinations/`:

   ```markdown
   ---
   id: 20260809T160000
   type: examination
   title: 體用 examination of c-overclaim
   lens: SUBSTANCE-FUNCTION
   examines: [c-overclaim]
   examined_by: claude
   ---

   What the lens looked for, what was found, and what is proposed — with the
   proposed nodes and edges written out so a human can accept or reject them.
   ```

5. **Re-run the checker.** Do not report a fix as done on your own authority:

   ```bash
   peira gates <vault> --node <id>
   ```

## The move that resolves most blocks

Most over-claims are not fixed by adding evidence. They are fixed by **restating
the claim within what the evidence already carries**.

The same observations, the same competing hypothesis — but against *"this entry proves
execution"* the alternative is a `contradicts` edge, and against *"the record
establishes catalogued presence, and may contribute to an execution inference alongside
independent evidence"* the very same hypothesis becomes a `limits` edge. Nothing about
the evidence changed. The claim stopped reaching past it.

Reach for that before you reach for more evidence.

## Honest failure

If a claim cannot be rescued, say so and write a `dissent` node preserving it and
its best argument. Rejection never deletes — that is the מחלוקת rule, and the
reasoning that rejected something is worth as much later as the conclusion.
"#;

/// The gate→lens→playbook table for the enforced lenses (the ones whose gates block).
fn skill_table() -> String {
    let mut out = String::from("   | Gate | Lens | Playbook |\n   |---|---|---|\n");
    for l in CATALOG.iter().filter(|l| l.phase == Phase::Enforced) {
        let (name, script) = display(l.id);
        let gates = if l.gates.is_empty() {
            "grounded extension".to_owned()
        } else {
            l.gates
                .iter()
                .map(|g| format!("`{}`", g.code))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let lens_label = if script.is_empty() {
            name.to_owned()
        } else {
            format!("{name} {script}")
        };
        let slug = l.id.to_lowercase();
        let _ = writeln!(
            out,
            "   | {gates} | {lens_label} | [{slug}.md](lenses/{slug}.md) |"
        );
    }
    out
}

/// One playbook, compiled from the lens's catalogue entry.
fn skill_playbook(lens: &Lens) -> String {
    let (name, script) = display(lens.id);
    let enforced = lens.phase == Phase::Enforced;
    let mut out = String::new();
    if script.is_empty() {
        let _ = writeln!(out, "# {name}\n");
    } else {
        let _ = writeln!(out, "# {name} {script}\n");
    }
    out.push_str("<!-- Generated from peira_lens::CATALOG by cli/examples/gen_site.rs. Edit the lens in lens/src/lib.rs and regenerate. -->\n\n");
    let _ = writeln!(
        out,
        "**peira code:** `{}` · **{}** · {}\n",
        lens.id,
        tradition_long(lens.tradition),
        if enforced {
            "**Enforced** — owns a gate"
        } else {
            "**Catalogued** — a reading, owns no gate"
        }
    );
    let _ = writeln!(out, "**Failure mode:** {}\n", lens.failure_mode);
    let _ = writeln!(out, "**What it does:** {}\n", lens.operation);
    if enforced && !lens.gates.is_empty() {
        let codes = lens
            .gates
            .iter()
            .map(|g| format!("`{}`", g.code))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "**Enforced gates:** {codes}\n");
    }
    let _ = writeln!(out, "**Worked example:** {}\n", lens.worked_example);
    if !lens.background.is_empty() {
        let _ = writeln!(out, "**In the tradition:** {}\n", lens.background);
    }
    out.push_str("**Sources:**\n");
    for s in lens.sources {
        let _ = writeln!(out, "{}", source_line(s));
    }
    let _ = writeln!(
        out,
        "\nRun `peira lens {}` for this entry in the tool.",
        lens.id
    );
    out
}

/// Old (romanized) lens slugs mapped to their new English slug. Each emits a redirect
/// stub so a bookmarked `/lenses/zhengming/` still lands on the renamed page. Historical
/// by nature — a one-time record of the id anglicisation, not derivable from the catalogue.
const REDIRECTS: &[(&str, &str)] = &[
    ("liji", "criterion"),
    ("zhengming", "rectify-name"),
    ("tiyong", "substance-function"),
    ("baima", "white-horse"),
    ("catuskoti", "four-corners"),
    ("pramana", "means-of-knowing"),
    ("pancavayava", "five-members"),
    ("trairupya", "three-marks"),
    ("anupalabdhi", "non-perception"),
    ("abhasa", "semblance"),
    ("gewu", "know-by-doing"),
    ("erdi", "two-truths"),
    ("machloket", "preserve-minority"),
    ("elenchus", "cross-examine"),
    ("aufhebung", "synthesis"),
];

fn redirect_stub(new_slug: &str) -> String {
    format!("---\nlayout: redirect\nredirect_to: /lenses/{new_slug}/\nsitemap: false\n---\n")
}

fn run() -> std::io::Result<usize> {
    let out_root = std::env::args()
        .nth(1)
        .map_or_else(|| PathBuf::from("docs"), PathBuf::from);
    let lenses_dir = out_root.join("lenses");
    fs::create_dir_all(&lenses_dir)?;

    let mut written = 0usize;
    for lens in CATALOG {
        let path = lenses_dir.join(format!("{}.md", lens.id.to_lowercase()));
        write_file(&path, &lens_page(lens))?;
        written += 1;
    }
    write_file(&lenses_dir.join("index.md"), &index_page())?;
    write_file(&out_root.join("doors.md"), &doors_page())?;
    for (old, new) in REDIRECTS {
        write_file(&lenses_dir.join(format!("{old}.md")), &redirect_stub(new))?;
    }

    // The Claude Code skill, compiled from the same catalogue. Its playbook filenames
    // are the (new) lens slugs, so the old-slug playbooks are cleared first.
    let skill = PathBuf::from(SKILL_DIR);
    let skill_lenses = skill.join("lenses");
    let _ = fs::remove_dir_all(&skill_lenses);
    fs::create_dir_all(&skill_lenses)?;
    write_file(
        &skill.join("SKILL.md"),
        &SKILL_PREAMBLE.replace("{TABLE}", skill_table().trim_end()),
    )?;
    for lens in CATALOG {
        write_file(
            &skill_lenses.join(format!("{}.md", lens.id.to_lowercase())),
            &skill_playbook(lens),
        )?;
    }
    Ok(written)
}

fn main() -> ExitCode {
    match run() {
        Ok(n) => {
            eprintln!("Generated {n} lens pages + index + doors");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("gen_site: {e}");
            ExitCode::FAILURE
        }
    }
}
