# Toulmin — Name the Warrant

**Gate:** `ELEN-WARRANT-MISSING`
**Failure mode:** the unstated warrant — grounds and claim are given, and the rule
connecting them never is.

Toulmin's observation is that arguments fail at the *warrant* far more often than at the
data, and that the warrant is the part nobody writes down. Writing it down is usually
enough to see the problem.

## What fires this

A claim has no `warrant:` — or has one that is blank. The gate distinguishes those two,
because a blank field is a different defect from a missing one: somebody started.

## What to look for

State the inference as a bare syllogism and read the middle line aloud:

- **Grounds:** the hive holds a record for this path.
- **Warrant:** *…therefore the program ran.*
- **Claim:** the program ran.

The warrant, written out, is `a catalogue record implies execution` — visibly false the
moment it is a sentence rather than an assumption.

Compare a sound one:

- **Grounds:** the hive holds a record for this path.
- **Warrant:** a catalogue entry evidences that Windows recorded the path and file
  identity; it licenses a statement about recording, and nothing beyond it.
- **Claim:** the record establishes catalogued presence.

## What to write

```yaml
warrant: >-
  A catalogue entry evidences that Windows recorded the path and file identity. It
  licenses a statement about recording, and nothing beyond it.
```

## Watch for

A warrant that restates the claim ("the warrant is that this proves execution") satisfies
the field and defeats the purpose. The test: could someone accept your grounds, accept
your warrant, and still reject your claim? If yes, the warrant is not yet doing its job.

Backing, qualifier and rebuttal are the rest of the Toulmin apparatus. The gate requires
only the warrant, because it is the one that is always missing.
