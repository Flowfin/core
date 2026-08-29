# Recorded fixtures, and how one is made

This directory holds responses recorded from a real server. It is empty of
recordings today, and the procedure below exists before the first one rather than
after it, which is the whole argument of #109: a rule added once fixtures have
landed does nothing about the ones already in the history, and the repair is a
history rewrite rather than a commit.

Read this before recording. `.github/fixture-scrub/fixture-scrub.sh` refuses a
recording that did not follow it, and
`docs/decisions/0109-what-a-recorded-fixture-may-carry.md` is where what follows
was decided and is the thing to argue with.

## What may be in a file here

Every value a recording carries is either a value nobody could have recorded from
a person, or one of the synthetic values below. Nothing else.

| what | what a recording carries |
| --- | --- |
| the session token | nothing. There is no synthetic token, and the header carrying one is removed rather than rewritten. |
| a password | nothing, for the same reason. |
| the server address | `https://server.invalid`, or `http://127.0.0.1` with the port the fake server answered on. |
| the device identifier | `synthetic-device-0` |
| the device name | `a synthetic device` |
| a server-supplied identifier | `00000000-0000-4000-8000-00000000000N`, with or without the hyphens, and `N` a single hexadecimal digit. |

`server.invalid` is reserved by RFC 6761: no resolver answers for it, so a
recording that leaks into a run cannot reach a machine belonging to anybody.

The synthetic values are stable on purpose. An identifier that changes on every
recording turns each re-recording into a diff nobody can read, and a real value
that slips through arrives in exactly that noise.

## The procedure

1. Record against a server you own, into a working file OUTSIDE this repository.
   Nothing arrives in the tree unscrubbed, not for a minute and not on a branch,
   because a commit that is amended away is still a commit that existed.

2. Replace, in this order. The token and the password first, because they are the
   two that are removed rather than rewritten and a substitution pass that
   rewrites them has already decided they may stay. Then the address, the device
   identity, and every identifier, each with the value from the table above.
   Where two identifiers are different in the recording they are different
   synthetic identifiers, or the recording stops being able to show the thing it
   was recorded for.

3. Read the file. The check refuses what it can name, and a title, an account
   name and a viewing history are none of those: 0068 closes its list with a
   question a contributor answers rather than with a set of shapes, so the last
   reading is yours. The question is 0068's own: could two people running the same
   build against the same server hold different values here?

4. Run the check before committing:

       bash .github/fixture-scrub/fixture-scrub.sh check

5. Commit the scrubbed file only.

## What the first recording has to land with, and this directory does not carry yet

`.gitattributes` normalises every tracked path to a line feed, and the one
exception is `tests/fixtures/**`, where a fixture exists to prove a byte. A
response recorded off a wire is the same case and is not covered by that
exception: the head of an HTTP answer ends its lines with a carriage return and a
line feed, so the tree-wide rule rewrites a recording into something no server
sent, silently, on the way into the index.

The rule this directory needs is `tests/recorded/** -text` beside the one already
there, with `tests/recorded/README.md text=auto eol=lf` after it so this document
stays normalised. It is not in `.gitattributes` today, and the reason is the rule
this repository holds about guards rather than an oversight: nothing here has the
bytes for it to bite on, and a `-text` line over an empty directory is a guard
that cannot fail, which is what `.gitattributes` says in its own header it exists
against. So the line lands in the same change as the first recording, together
with an assertion over that recording's bytes in the shape
`tests/fixture_bytes.rs` already holds for the other directory.

Read that before recording rather than after. A recording committed under the
tree-wide rule has already lost the bytes, and the repair is a re-recording.

## What this procedure does not do

It does not make a scrubbed recording honest against the server it came from.
That is #104, and it is the other direction: this says what may not be in a file,
and that says whether what is in one still matches a real server.

It does not reach a recording that was committed and then deleted. Nothing here
reads the history, which is the reason the order in step 1 is absolute rather
than a preference.
