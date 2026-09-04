# Code of conduct

## Who this covers, and where

Everybody who takes part here: an issue, a pull request, a review, a commit
message, and a private message that started from one of those. It covers me too.
I hold this repository and I am inside these rules rather than above them, which
is the sentence that decides whether the rest of the document is worth reading.

## What is expected

Argue with the work. Everything else in this repository is built on that
already - a claim carries the command that produced it, a number carries its
run, and a disagreement is settled by executing something rather than by who
said it. `CONTRIBUTING.md` states that as a rule about evidence. Here it is the
same rule about people: the thing under discussion is a file, a measurement or a
decision, never the person who wrote it.

Keep measured and assumed apart in the sentence. Being wrong in public is the
ordinary cost of working this way, and a correction is a repair rather than a
verdict on whoever needed it. Somebody who writes "I did not measure this" has
done the harder thing, not the weaker one.

Take a refusal with its reason. A change sent back here comes back with what was
wrong, and disagreeing with that reason is fair. Reopening it unchanged in a
different place is not.

## What is not acceptable

- An attack on a person rather than on their work, including one dressed as a
  question about their competence.
- Demeaning somebody for who they are, or for a group they belong to.
- Harassment, in the open or in messages that started here and continued
  somewhere else.
- Publishing somebody's private details - a legal name, an address, an employer,
  anything they did not put here themselves - without their consent.
- Unwanted attention of a sexual kind, and sexual material in any part of this
  repository or its tracker.
- Sustained disruption: the same settled argument reopened without new evidence,
  a thread derailed on purpose, a review answered with volume instead of
  substance.
- Threatening any of the above, whether or not it is carried out.

That list holds what has to be named, and it is a floor rather than a boundary.
Behaviour nobody wrote down is judged by the section above it, not permitted by
its absence from this one. Some of this repository's own gate rules are built the
same way, in `.github/invariants/rules`, where a register of names stands in for
a test of purpose and every run prints that bound beside its verdict. The same
disclosure is owed here.

## Reporting

Two routes, in this order.

**A private report on this repository.** The form under Security is the only
private channel this repository has:

    https://github.com/Flowfin/core/security/advisories/new

It is labelled for a vulnerability, because that is what it was built for. A
conduct report sent through it is not misfiled: it reaches the same person, it
is not public, and no part of it is published by submitting it. That the form
answers at all is a setting on the repository rather than a promise in a file,
so it is read rather than asserted:

    gh api repos/Flowfin/core/private-vulnerability-reporting
    {"enabled":true}

Run 2026-09-04. That reading needs administrative access to this repository, so
it is not one a reader of this board can run; what a reader can do is open the
address above and see whether the form is there.

`SECURITY.md` names the same form for a vulnerability and is where a security
report belongs; the two documents point at one door on purpose, so there is no
second route to keep in step with this one. That file carries its own reading of
the same setting, taken earlier, answering `false`, and saying in its own words
that the destination was shut on the day it was read. The reading above is the
later of the two and was taken against the same repository.

**A message through GitHub to the account that holds this repository**, which is
https://github.com/iderex. Use this where the first route does not fit - where
the report is about the form itself, or where you would rather it did not sit in
a security queue.

Do not open an issue about a conduct problem. An issue here is public from the
moment it is submitted, and it names the person it is about to everybody, before
anybody has read it.

No mailbox is published, here or anywhere else on this board. An address in a
document outlives whoever was reading it and stays in the history after it is
changed, and an address nobody watches is worse than none because the document
promises a reply. Both routes above are accounts on a service that already
authenticates the sender, and they need nothing kept alive to keep working.

## What happens after a report arrives

I read it, and I answer it. If something is missing I ask for it, and if I decide
it is not a problem you get the reason rather than silence.

There is no deadline and there will not be one, for the reason `SECURITY.md`
gives about the other route: a window this project cannot hold is worse than
none, because a reporter left past it cannot tell a busy week from a report that
never arrived.

What I can do is limited and worth stating exactly rather than implying more: ask
somebody to stop, edit or delete content on this repository, close or lock a
thread, block an account from this repository, and report an account to GitHub.
Nothing here reaches anybody outside this repository, and none of it is a
sanction anywhere else.

Your name does not appear in whatever follows unless you ask for it to. Where
something has to be said in public, it is said about the behaviour and the
outcome.

## Where this document is weak

**One person holds it.** A report about my own behaviour comes to me, and there
is no second reader, no appeal and no independent body. That is the honest state
of a repository with one holder rather than an arrangement I am recommending, and
it is the reason the route below exists rather than being buried. GitHub's own
abuse route is not mine and does not go through me:

    https://github.com/contact/report-abuse

**Nothing enforces this.** No check on this repository reads this file, and none
could: every gate leg here judges bytes in the tree, and how somebody spoke to
somebody else is not a byte in the tree. What stands behind this document is that
I do what it says, and the way to find out that I did not is to report it. A
document that looked enforced would be worse than this sentence.

**It is not the Contributor Covenant**, and the difference is deliberate. That
document's enforcement section describes a body of people, a graduated ladder of
consequences and a review of appeals. None of those exists here, and publishing a
ladder nobody climbs would describe an apparatus this repository does not have -
which is the failure the paragraph above is written against.
