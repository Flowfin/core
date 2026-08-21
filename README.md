# core

Every Flowfin client needs the same things and none of them should write those twice: talking to a Jellyfin server, holding a session, caching what was fetched, decoding artwork, tracking playback position, and measuring whether the speed budget was met. Eleven clients written independently drift in what they cache, in when they give up on a slow server, and in what they call fast. The speed budget is written as numbers a build can miss, and a number nothing measures is a wish, so this is where those numbers are instrumented. What shared means technically is the first maintainer decision and the plan states the options with their costs rather than choosing one. The core draws nothing: a core that knows about widgets stops being shared the first time two platforms disagree about a list.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

See [SECURITY.md](SECURITY.md) for how to report a security problem, what
this repository treats as one, and what a reporter gets back.

## License

AGPL-3.0, copyright 2026 Nils Lehnen.

The full text is in [LICENSE](LICENSE).
