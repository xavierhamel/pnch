# pnch
Track your time working on projects directly from the CLI. Categorize and add a description to what
you did and later export your timesheet to different formats.

## Installation
Clone the repo
```
git clone https://github.com/xavierhamel/pnch.git
```

Build with [cargo](https://github.com/rust-lang/cargo)
```
cd pnch
cargo build --release
```

Finally, add the built binary to your paths!

## How to use it
### Managing entries
When you start a new task, simply punch in. This will start a new entry saving the current time.
```
pnch in
```

When you are done with your task, punch out and specify a tag and a description.
```
pnch out "ISSUE-123/The issue was fixed"
```
The tag is the value specified before the forward slash (`/`) and the description is everything
after. In the example above, "ISSUE-123" would be the tag and "The issue was fixed" would be the
description of the issue.

It is also possible to add the tag and description while punching in
```
pnch in "ISSUE-124/The source of the bug was found"
```

or could be edited later:
```
pnch edit "ISSUE-124/The source of the bug was NOT found"
```

For now, when an entry is closed (with `pnch out`) there is no way to edit it afterwards.

If you forgot to pnch in, it is also possible to specify the time while punching in or out:
```
pnch in --time 8:02
```

### Listing and exporting entries
To export or list your timesheet, use the command
```
pnch ls
```

You can either list in a pretty format or export in a csv format.

## Coming in the future
 - [ ] Prettier
    - [ ] Better error messages.
    - [ ] Entries grouped when listing in pretty format.
    - [ ] Minimal colors for terminal supporting it.
- [ ] Edit past entries
- [ ] Computing the elapsed time
    - [ ] Total time
    - [ ] On specific tag
    - [ ] On specific day/period
- [ ] Improving the install guide and the documentation

---
Enjoy!
