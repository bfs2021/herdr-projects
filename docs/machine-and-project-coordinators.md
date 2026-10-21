# Machine and project coordinators

**Three levels you can talk to, each its own conversation.**

| Level | Talk to it | Its row says | Named in the chat | Lives in Herdr |
|---|---|---|---|---|
| The [Coordinator](coordinator.md) | the chat, as always, or `Enter` on its row | `Main coordinator` | `Coordinator` | a `Coordinator` tab in the home space (`~`) |
| A machine coordinator | `Enter` or `Space` on its row | `Machine coordinator` | after the machine: `vm-str coordinator`, `Mac coordinator` | a tab of that name in that machine's home space |
| A project coordinator | `Enter` or `Space` on its row | `Project coordinator · herdr-projects` | after the repo: `herdr-projects coordinator`, `~ coordinator` for the home folder | a tab of that name in the repo's own space |

The Coordinator stays as it is. The others sit beside it: talking to one never touches the others.

## Their rows

**Each coordinator is a row of the list, drawn as a task agent's**: a mark in the marks' column for where it stands, the
same marks the chat's line has for a coordinator or an agent (working, idle, needs you, stopped, not started, `◎` on
[autopilot](autopilot.md)), its name, the last thing it said on the line under it, and on the right how long ago it said
it (on autopilot its timing: `2 of 5 added  last HSW-92  next in 40m`). No review tag, pull request or progress: a
coordinator has none.

- **Where it stands is where the grouping puts it** (`Alt+S`): the main one and the machine's at the top of their
  machine's part (under the machine's heading with every machine shown), a project's beside them, or grouped by repo at
  the top of its repo's group. Another machine's main Coordinator stands in that machine's part too.
- **Under All** only those that were ever talked to show, and the main one. **The filter Coordinators** (`Shift+Tab`
  from All, [the filter row](board.md#the-filter-row)) shows the board with only the coordinators' rows left, every one
  of the machine shown: also a machine's that never ran and a project's used lately, so that your first message starts
  it.
- **Headings are only headings**: a machine's or a repo's folds and unfolds, and has no coordinator of its own.

## Talking to one

- **`Enter` or `Space` on its row** opens it in the chat. The chat names it (`herdr-projects coordinator  idle`), and so
  does the box: `Send message to herdr-projects coordinator`. `Enter` sends, and it decides what to start, as the
  Coordinator does.
- **It follows the selection**, as a task's agent does: move to another coordinator's row and you talk to that one,
  move to a task and you talk to its agent. `Space` again goes back to the Coordinator.

Your first message starts it. Until then the chat says `not started: your first message starts it`.

## What each one is

- **An ordinary agent in a long-lived chat**, like the Coordinator: the same board commands, its own tab, and settings
  of its own (below). Everything you say is one conversation, kept: if its
  program exits or Herdr restarts, your next message starts it again in the same conversation.
- **A machine coordinator** is about that machine as a whole: its repos and folders, the tasks and agents on it. It runs
  in a folder of its own (`~/.herdr-projects/data/coordinator/<session>/machine/chat`), with its rules in `AGENTS.md`
  there, as the Coordinator's are.
- **A project coordinator** is about one repo. It runs in the repo's own folder, at its main level: your checkout. It
  reads it freely and changes nothing there unless you ask it to; work goes on the board as tasks of that repo. Its tab
  goes in the repo's space (the one Herdr labels after the repo); with none open, one is made there. Since the repo's
  `AGENTS.md` is yours, its rules are a file beside its state
  (`~/.herdr-projects/data/coordinator/<session>/projects/<repo>/rules.md`), passed when it starts, as a working agent's
  are.
  Your own chats in the repo's folder are never taken for its conversation: it keeps the one it has, and moves to
  another (after its own `/clear`) only when Herdr names that one for its pane.
- **Another machine's** coordinators run on that machine. Its board sends what they said with its tasks, and what you
  type goes there. A machine on an older version has none: the chat says to update it there.
- **Herdr's sidebar** names each as the board does, with where it stands and its last message.

## A row is its coordinator

A coordinator's row stands for it, as a task's row stands for the task:

- `Alt+P` shows it beside the list, `Alt+T` whole, in the [task panel](task-panel.md)'s form.
- `Ctrl+E` opens its settings page: its own rows first, under `This coordinator`, then for the machine's your own
  settings, for a project's your entry for that repo.
- `Ctrl+S`, with nothing typed, puts it on [autopilot](autopilot.md) with its saved goal and settings; `Ctrl+X` takes it
  off. While on, its mark is autopilot's `◎`, its timing is on the right and what it does is under its name.

**Each has its own settings**, `coordinator.*` and `autopilot.*`, looked up **this coordinator → you → built in**
([settings](settings.md)). What you set for yourself counts for every coordinator; one set for this coordinator alone
wins over it. `coordinator.instructions` add up: yours, then the coordinator's own.

```
herdr-projects set coordinator.effort high --machine-coordinator
herdr-projects set coordinator.agent codex --project-coordinator herdr-projects
herdr-projects show --project-coordinator herdr-projects
```

## From a shell

Every `coordinator` command takes `--machine` or `--project <repo>`:

```
herdr-projects coordinator --machine                    how it stands
herdr-projects coordinator --machine tell <text>        send it a message
herdr-projects coordinator --project herdr-projects tell <text> [--delegate]
herdr-projects coordinator --project <repo> restart | new | stop
```
