# Parley


## Goals

* Should be easy to deploy and easy to understand and use.
* Supports different and varied document formats but should start with typst.
* Robust command line and easy to use frontend.
* No ties with Github, I don't want to lean on their review format and tooling I want to create my own.
* User experience is paramount. It should be easy to review a document and then respond to that document review.
  Document versioning, comments which anchor to versioning, rich text, etc etc. The discussion should feel easy to participate in
  and resolve.
* Historical features. You should be able to look up writing by authors, modified times, accepted time, etc etc etc. I should
  be able to go back and see different versions of the same document.
* Rich search


## The problem

Most companies don't have a clear process or design canvas for RFDs. They usually try to fit it into Google docs or notion
or something like that, but that creates its own set of problems since none of those tools were specifically made for
RFDs themselves.

I want a tool that feels like it was designed for the workflow people use when they want to present, collaborate on, and
move forward with ideas.

## Parts

* A main API and web server which will render/serve the documents and allow people to comment on them.
* A CLI which will allow users to search, display, update, update states, and more. A central control interface for
  the entire application.



## Ideas

* We can probably use ssh for auth? This would give a good basis for registration without creating an API key
  for the user. Although, if we want people to use the API we do need some token based provisioning. We should tie
  in web authentication to the CLI and have that process easy. Unsure if we should just sans the ssh and then just
  distribute a key to keep things consistent all the way around, not a split of ssh auth and token auth.
* You comment in the web ui but edit in your local editor. I think this is the way it needs to be but it would be nice to
show comments next to the editing that you're doing so you can refer to what the person said in one pane. BUt might be
too hard
* CLI subcommands: init, edit, update, view,
* We'll do full text snapshots as that is fairly cheap. Delta storage can be complex. Possibly we'll drop all non-cricial
 versions after a while though? If a document has 50 versions and only 3 of them have comments pinned to them then realistically
  we can cut out most versions and just leave the versions with the comments, the first one, and the latest one. A form of
  compaction that can save on space over the long term. Deleting is not great though, would much rather store deltas? unsure.
* We'll support many document formats but starting with typst is the first. Need html renderers for each new format.
* We need a good anchoring strategy:
## The Anchoring Engine: Block-Relative Context

To support hyper-focused, long-form technical review comments without suffering from line-number drift bugs when documents are updated, Parley splits spatial layout movement from textual mutation.

### The Mechanism
1. **The Block Hierarchy:** Typst source files compile into distinct, deeply structured HTML elements. The Parley server injects a persistent, content-derived hash attribute into every structural node block (e.g., paragraphs, lists, code containers): `<p data-block-hash="sha256-a1b2...">`.
2. **Granular Hooks:** When a user highlights text inside the web interface to leave a comment, Parley captures coordinates relative *only* to the boundaries of that independent block, alongside immediate text context strings:
   ```json
   {
     "thread_id": "uuid-v7",
     "block_hash": "sha256-a1b2c3...",
     "start_offset": 42,
     "length": 6,
     "context": {
       "exact": "SQLite",
       "prefix": "We choose ",
       "suffix": " for simplicity."
     }
    }


Collision Resolution

    Text Shift (Pristine): If an author inserts 100 new lines above this comment, the target paragraph remains internally unchanged. The block_hash matches perfectly during the next compilation compile phase. The comment effortlessly floats down the page, perfectly anchored to its native text.

    Text Change (Fuzzy Match): If an author edits the paragraph itself, the block hash breaks. The engine triggers a fuzzy context search inside that region looking for the prefix + exact + suffix bounds to re-anchor the comment.

    Text Obliteration (The Time-Machine): If the text block is deleted or heavily altered, the thread is gracefully flagged as is_outdated = true. It collapses neatly under its nearest parent heading. Users can click the archived thread to open a split-screen visual snapshot showing the exact historical document state at the millisecond that comment was submitted.


### Schema

Table	Purpose
users	Tracks team member IDs, usernames, and base workspace details.
ssh_keys	Maps public cryptographic SSH components to specific registered user accounts.
rfds	Root metadata table tracking titles, current statuses, and identifiers.
rfd_revisions	Houses the compressed, full-text source document snapshots along with compiled HTML records for every single save event.
threads	Tracks discussion metadata, block-relative hashes, structural offsets, and context mapping coordinates.
messages	Flat message records contained within a thread, supporting raw Markdown bodies.
