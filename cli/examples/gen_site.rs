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
        "LIJI" => ("Set the Pole", "立極"),
        "ZHENGMING" => ("Rectification of Names", "正名"),
        "TIYONG" => ("Substance and Function", "體用"),
        "BAIMA" => ("The White Horse Is Not a Horse", "白馬非馬"),
        "CATUSKOTI" => ("The Four Corners", "四句"),
        "TOULMIN" => ("Name the Warrant", ""),
        "PRAMANA" => ("The Means of Knowing", "प्रमाण"),
        "RUNG" => ("Earn the Rung", ""),
        "ELENCHUS" => ("Socratic Cross-Examination", "ἔλεγχος"),
        "ACH" => ("Analysis of Competing Hypotheses", ""),
        "PANCAVAYAVA" => ("The Five-Membered Argument", "पञ्चावयव"),
        "STEELMAN" => ("Steelman First", ""),
        "DOUBLECRUX" => ("Double Crux", ""),
        "MACHLOKET" => ("Preserve the Minority", "מחלוקת"),
        "AUFHEBUNG" => ("Synthesis That Preserves", ""),
        "THESEUS" => ("Ship of Theseus — Amend or Supersede", ""),
        "CHESTERTON" => ("Chesterton's Fence", ""),
        "PREMORTEM" => ("Premortem / Inversion", ""),
        "TRAIRUPYA" => ("The Three Marks of a Valid Reason", "因三相"),
        "ANUPALABDHI" => ("Non-Perception as a Reason", "不可得因"),
        "ABHASA" => ("The Semblances of Proof", "似因・似宗"),
        "ERDI" => ("The Two Truths", "二諦"),
        "DUNG" => ("Grounded Extension — Compute, Don't Assert", ""),
        "GEWU" => (
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
        Tradition::Indian => "Indian — Nyāya and the pramāṇa epistemology",
        Tradition::Buddhist => "Buddhist — Madhyamaka and Dignāgan logic",
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

/// The seven-doors page, compiled from the canonical method doc so it cannot drift:
/// front matter is prepended, and every backtick-wrapped lens code becomes a link.
fn doors_page() -> String {
    let mut body = include_str!("../../docs/method/anti-summarization.md").to_owned();
    // The canonical doc references lenses by their peira code (`ZHENGMING`), because that
    // is the identifier the CLI and packets print. On the site the code is never a label:
    // link text is the English name, and the original script that already follows in the
    // doc stays as etymology. Romanization does not appear.
    for lens in CATALOG {
        let needle = format!("`{}`", lens.id);
        let link = format!(
            "[{}]({{{{ '/lenses/{}/' | relative_url }}}})",
            display(lens.id).0,
            lens.id.to_lowercase()
        );
        body = body.replace(&needle, &link);
    }
    body = body
        .replace("](README.md)", "]({{ '/method/' | relative_url }})")
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
