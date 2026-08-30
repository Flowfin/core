# 0039. The page, the one item type, and what next up is not

Date: 2026-08-30

Status: accepted. Supersedes nothing. Superseded by nothing.

Issue: #39

## The decision

A library read answers with one page type carrying the items, the offset the page
begins at and the total the server stated, asked for by an offset and a count on
the two routes in 0010 that accept them and answered whole on the one that does
not; every item in every one of those reads is one item type, which the server
already makes true; the core always asks for the total and never turns it off,
because the field a server returns with counting off is the page's own length
rather than nothing; and next up is not decided here, because no capability in
0010 carries it.

## Where the readings were taken

The two server lines are 0010's, at the two commits that record names, so this
one is written against the same bytes:

    $ A=1fbd8739292cce610231be93daf43368733edf63    # the 10.11 line
    $ B=c3ed1407ca698b0905de99da87b67415e6a62dbd    # the 12.0 line
    $ for r in $A $B; do
    >   git -C jellyfin show $r:SharedVersion.cs | grep AssemblyVersion | head -1
    > done
    [assembly: AssemblyVersion("10.11.11")]
    [assembly: AssemblyVersion("12.0.0")]

Every reading below is at both, and where the two agree the output is pasted once
with that said rather than twice.

## The page is an offset and a count, because that is what the server has

The two routes 0010 names for a list of items take exactly those two:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/ItemsController.cs \
    >     | grep -c 'FromQuery] int? startIndex'
    > done
    4
    4

Four apiece, which is the two reads this record is about and the two obsolete
`Users/{userId}` spellings of them that 0010 already excludes. There is no
cursor, no continuation token and no link header to follow, so a cursor in the
core's own interface would be a value the core invented over an offset it still
had to send, and the first thing it would hide is the case below.

The answer carries the offset back, and it is the same three fields on both
lines:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:MediaBrowser.Model/Querying/QueryResult.cs \
    >     | grep -n 'public .* {' | head -3
    > done
    47:    public IReadOnlyList<T> Items { get; set; }
    53:    public int TotalRecordCount { get; set; }
    59:    public int StartIndex { get; set; }

So a page is those three and the core adds nothing to them. What it does add is a
name for the fourth thing a caller needs, which is whether there is another page,
and that is derived from the three rather than asked for: the offset plus the
number of items returned, against the total. Derived rather than stored, because a
stored flag is a fourth field that can disagree with the three it came from.

## The total is a number that can lie, and the core never asks it to

Both routes take a flag that turns counting off, and it defaults to on:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/ItemsController.cs \
    >     | grep -c 'FromQuery] bool enableTotalRecordCount = true'
    > done
    4
    4

What that flag does is the part worth having in front of anybody writing the
paging loop. With counting off the server builds the result with no total at all:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Emby.Server.Implementations/Library/LibraryManager.cs \
    >     | grep -A8 'if (query.EnableTotalRecordCount)' | head -9
    > done
                if (query.EnableTotalRecordCount)
                {
                    return _itemRepository.GetItems(query);
                }

                return new QueryResult<BaseItem>(
                    query.StartIndex,
                    null,
                    _itemRepository.GetItemList(query));

and the constructor that receives that `null` fills it in from the page:

    $ git -C jellyfin show $A:MediaBrowser.Model/Querying/QueryResult.cs | sed -n '36,41p'
        public QueryResult(int? startIndex, int? totalRecordCount, IReadOnlyList<T> items)
        {
            StartIndex = startIndex ?? 0;
            TotalRecordCount = totalRecordCount ?? items.Count;
            Items = items;
        }

Identical on the 12.0 line. So `TotalRecordCount` with counting off is the number
of items in the page, and it arrives in the same field, with the same type, as a
real total. A client paging until the offset plus the page length reaches the
total stops after the first page and shows a library with one screenful in it,
and nothing anywhere reports an error.

The core therefore never sends that flag as false, and the reason is written here
rather than left as a default nobody chose. It is not a performance option this
core declines to use; it is a field that becomes wrong rather than absent, which
is the one shape 0004's vocabulary has no way to express and 0101 says to expect
from a server.

What the core does with the total it is given is state it and nothing more. It is
the server's number, it is not revalidated, and a client that compares it against
what it has drawn is comparing two things the server said at two moments.

## The view list is not paged, and the core does not pretend it is

The third read in 0010's `library-query` capability takes neither parameter:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/UserViewsController.cs \
    >     | sed -n '/HttpGet("UserViews")/,/^    {/p' | grep 'FromQuery'
    > done
        [FromQuery] Guid? userId,
        [FromQuery] bool? includeExternalContent,
        [FromQuery, ModelBinder(typeof(CommaDelimitedCollectionModelBinder))] CollectionType?[] presetViews,
        [FromQuery] bool includeHidden = false)

Identical on both lines. There is no offset and no count, so the whole set comes
back in one answer.

The core answers that read with the same page type, carrying an offset of zero and
a total equal to the number of items, and says in the type's own documentation
that this read has one page and always will. That is the honest shape rather than
a second answer type: a caller written against a page works for all three reads,
and a caller that asks this one for a second page is told there is none, which is
true. What is refused is the other direction, a paging request the core would turn
into nothing on the wire, because a caller asking for the second hundred views and
receiving the first hundred silently is the same failure as the total above.

## One item type across every read

This is the condition #39 states last and it is the one the server already meets.
Every list read answers with the same generic over the same item, and the detail
read answers with that item on its own:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/UserViewsController.cs \
    >     | grep -c 'public QueryResult<BaseItemDto> GetUserViews'
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/ItemsController.cs \
    >     | grep -c 'ActionResult<QueryResult<BaseItemDto>>'
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/UserLibraryController.cs \
    >     | grep -c 'ActionResult<BaseItemDto>> GetItem\|ActionResult<BaseItemDto> GetItem'
    > done
    2
    4
    2
    2
    4
    2

Every count is a read this record is about plus the obsolete `Users/{userId}`
spelling of it, which is why the middle one is four: it holds two reads.

So the core does not have to make this true; it has to avoid breaking it. What
would break it is a type per read, which is what a codebase acquires when each
read is written by whoever needed it: a view, a list item and a detail item that
are three types because three call sites wanted three different subsets. The core
carries one, every read produces it, and the fields a given read did not populate
are absent on the value rather than absent from the type.

Which fields those are is not decided here. It depends on what the core asks for
in the `fields` parameter, that parameter is a list the server documents per read,
and choosing it is a request-shaping decision that belongs with the code rather
than with this record. What is decided is that the difference shows up as an
absent field on one type and never as a second type.

## Resume is a read this board has; next up is not

Resume is in 0010's table as `resume-list`, it answers with the same item type,
and it takes the same two paging parameters:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/ItemsController.cs \
    >     | sed -n '/HttpGet("UserItems\/Resume")/,/^    {/p' \
    >     | grep -c 'FromQuery] int? startIndex\|FromQuery] int? limit'
    > done
    2
    2

so it is a library read like the other two and nothing here is special about it.

Next up is different, and it is the one thing #39's body asks for that this record
refuses rather than decides. The route exists on both lines and answers with the
same type:

    $ for r in $A $B; do
    >   git -C jellyfin show $r:Jellyfin.Api/Controllers/TvShowsController.cs \
    >     | grep -n 'HttpGet("NextUp")' -A2 | grep 'HttpGet\|ActionResult'
    > done
    76:    [HttpGet("NextUp")]
    78-    public ActionResult<QueryResult<BaseItemDto>> GetNextUp(
    75:    [HttpGet("NextUp")]
    77-    public ActionResult<QueryResult<BaseItemDto>> GetNextUp(

and it is in none of 0010's sixteen capabilities:

    $ git grep -c 'NextUp' -- docs/decisions/0010-the-server-surface-and-what-an-absence-does.md
    exit status 1

0010 is the authority for which paths the core may reach, 0069 is the authority
for which hosts, and #70 is the test that fails when the core reaches something
nobody configured. A path added here rather than there would be a surface growing
in the record that describes the thing rather than in the record that fixes it,
which 0010 names in its own words as how a surface grows by accident.

So next up is one of two things and neither is this record's: either 0010 is
superseded by one whose table carries `GET /Shows/NextUp`, with the argument for
it made there, or #39's scope loses next up and says so. This record states which
of #39's conditions that reaches: none of them. The three conditions are a test
per call against a recorded fixture, paging proven across a boundary, and one item
type across the calls, and each is about the calls that exist rather than about
the count of them.

## Why this is written down before the code

Nothing in this tree makes a request, and #27 is the transport. That is what makes
this record cost one file: an offset, a total that can be a page length, and a
read that is not paged are three shapes a caller has to hold, and every one of
them is cheaper to fix now than after five call sites and eleven clients have each
decided for themselves.

The specific failure is the total. A paging loop written against `TotalRecordCount`
is correct on every server that leaves counting on, which is every server anybody
will test against, because the flag is a thing the caller sends. It becomes wrong
only where somebody adds the flag for a measured reason, at which point the loop
is somewhere else and the symptom is a library that stops after one screenful.
That is a defect that ships, and the reading above is one command.

The second failure is quieter. `GET /UserViews` returning everything means a paging
parameter on the core's own view read would go nowhere, and the caller cannot tell
a request that was not sent from one that was answered in full. Both readings are
the same class as the trap 0055 records for a refused image and #51 for an absent
one: two different things arriving in one shape.

## Alternatives, and what each cost

A cursor in the core's own interface, opaque to the client and holding the offset
inside. It survives a server that changes its paging, and it stops a client from
constructing an offset the server has not seen. It costs the thing a cursor is
for: there is no cursor on this interface, so the core would be encoding an offset
it still sends, and a client that wants item four hundred has to walk to it. The
core's callers are eleven clients drawing a grid somebody scrolls, which is the
one case an offset serves better.

A separate answer type for the view list, since it is not paged. Honest about the
server, and it costs the condition #39 states last: a client written against one
read stops working for another, which is exactly the drift a shared core exists to
remove. A page with one page in it says the same thing and says it in the type the
other reads use.

Making the total optional on the core's page, so that a server with counting off
is expressed rather than papered over. It is the more truthful shape and it would
be right if the core ever sent that flag. It costs every caller a case that cannot
arise, because the flag is the core's to send and this record fixes it as never
sent; where that changes, this record is what has to move first.

Asking for the count separately, once, and paging without a total. Two requests
where the server offers one, on the first screen, which is the 1.2 seconds in #62.

Taking next up now, on the ground that the route exists on both lines and returns
the type this record already fixes. It would close #39's body as written and it
costs the thing 0010 is for: the enumerated surface would gain a path in a record
that is not the enumeration, and the next path would arrive the same way.

## What would reverse this

A supported server line offers a cursor or a continuation token on the two paged
reads, so that the offset is no longer what the interface has. The comparison is
against the route parameters at the two commits a superseding 0010 names, not
against a judgement about what is modern.

`GET /UserViews` gains an offset and a count on a supported line. The one-page
answer then hides a real second page, which is the failure this record is written
against, pointed the other way.

The core is measured to need the counting flag off, on a real library, with the
number that says so. The rule that the core always asks for the total is then
paying more than it is worth, and what replaces it is a page whose total is
optional rather than a page whose total may be a page length.

0010 is superseded by a record whose capability table carries a next-up path. This
record's last section is then answered from outside it, and what is owed is a
statement of what next up is in terms of the page and the item, which is the half
this record deliberately does not write.

Two of the three reads stop answering with the same item type on a supported line.
The condition #39 states last is then not the server's to meet and the core owes a
mapping, which is a different record from this one.
