# The fixture analyses

Five files, and none of them is an analysis of this tree. Each is a SARIF
document written by hand so that a rule in `.github/codeql/codeql.sh` is proven
against text it fully controls. A fixture that judged a real analysis would prove
the state of the tree on the day it ran rather than the rule.

They are JSON, so none of them carries a comment of its own, which is why this
file exists. Each is one change away from the one above it.

`one-finding.sarif` is the subject. One run, a tool whose extension declares two
rules, and one result naming a rule, a level, a file and a line. It is what the
gate has to refuse, and it is what proves that the reader reaches the rule
identifier a register entry would have to name.

`no-finding.sarif` is the one-change neighbour: the same run, the same two rules
loaded, and an empty result set. It has to pass. A neighbour rather than an empty
document, because a gate that passes an empty document proves that a reader of
nothing finds nothing.

`finding-without-a-rule-id.sarif` is the same result with its `ruleId` and its
`level` removed. It has to be read as a finding rather than dropped, because a
result naming no rule is still a finding and dropping it is the quietest way to
lose one.

`no-rules-loaded.sarif` is the same run with the extension's rule set emptied and
nothing found. That is a query set that never loaded, and it reports zero
findings, so it reads exactly like a clean analysis. The gate refuses it on the
count rather than on the results.

`no-run.sarif` carries no analysis run at all, which is the same failure one step
further out.
