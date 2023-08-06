# pnch
Track your time from the CLI. 

<img src="./assets/logo.png" width="250" alt="pnch" align="right">

- [Overview](#overview)
- [Installation](#installation)
- [How to use it](#how-to-use-it)
    - [Managing entries](#managing-entries)
    - [Listing and exporting entries](#listing-and-exporting-entries)
- [Integrations](#integrations)
    - [Tempo](#tempo)
- [License](#license)

## Overview
Categorize and add a description to what you did and later export your timesheet to different formats.

If it doesn't do something which you feel it should or it's not clear how to, please file an issue.

```
$ pnch in
you are now pnched in.

$ pnch out "pnch/Release on github!"
you are now pnched out.

$ pnch ls
You were punched in for 8 hours 22 minutes
┌------------┬-------┬----------------┬-------┬-------┬--------------------------------------------┐
│ Date       │ Id    │ Tag            │ In    │ Out   │ Description                                │
├------------┼-------┼----------------┼-------┼-------┼--------------------------------------------┤
│ 2023-07-30 │ 4     │ [pnch]         │ 18:31 │ 20:29 │ Added tags                                 │
├------------┼-------┼----------------┼-------┼-------┼--------------------------------------------┤
│ 2023-08-04 │ 5     │ [pnch]         │ 08:30 │ 09:41 │ Added description                          │
│            │ 6     │ [pnch]         │ 09:41 │ 11:03 │ Better error messages                      │
│            │ 7     │ [pnch]         │ 11:03 │ 12:08 │ Resolve a bug                              │
│            │ 8     │ [pnch]         │ 12:59 │ 14:15 │ Added editing                              │
├------------┼-------┼----------------┼-------┼-------┼--------------------------------------------┤
│ 2023-08-06 │ 9     │ [pnch]         │ 10:03 │ 10:33 │ Release on github!                         │
└------------┴-------┴----------------┴-------┴-------┴--------------------------------------------┘
```

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

Finally, add the built binary to your paths.

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
description of the issue. The tag is something to identify a group of entries. It could be a 
project, a subproject, a specific task, or like in this example, a particuliar issue.

It is also possible to add the tag and description while punching in
```
pnch in "ISSUE-124/The source of the bug was found"
```

or could be edited later:
```
pnch edit "ISSUE-124/The source of the bug was NOT found"
```

If you forgot to pnch in, it is also possible to specify the time while punching in or out:
```
pnch in --time 8:02
```

or by editing it later on:
```
pnch edit --in 8:01
```

### Listing and exporting entries
To export or list your timesheet, use the command
```
pnch ls
```
By default, only the last two weeks are printed. To print more use the different period filters:
```
pnch ls --since 2023-01-01 --from 2022-12-01 --to 2022-12-05 --last "4 weeks"
```
> Note that an entry only needs to be true for only one flag to be returned by the `ls` commands.

To update the default period, use
```
pnch config ls-default-period "5 weeks"
```

You can also filter by tags
```
pnch ls --tag "ISSUE-123"
```

You can either list in a pretty format or export in a csv format.

## Integrations
### Tempo
Coming soon

## License
MIT - Enjoy!
