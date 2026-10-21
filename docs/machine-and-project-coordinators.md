# Machine and project coordinators

**Three levels you can talk to, each its own conversation.**

| Level | Talk to it | Named | Lives in Herdr |
|---|---|---|---|
| The [Coordinator](coordinator.md) | the chat, as always | `Coordinator` | a `Coordinator` tab in the home space (`~`) |
| A machine coordinator | `Space` on its machine's row | after the machine: `vm-str coordinator`, `Mac coordinator` | a tab of that name in that machine's home space |
| A project coordinator | `Space` on its project's row (its repo's heading) | after the repo: `herdr-projects coordinator`, `~ coordinator` for the home folder | a tab of that name in the repo's own space |

The Coordinator stays as it is. The others sit beside it: talking to one never touches the others.

## A machine's row and a project's row are its coordinator

**A machine's row** (its heading, with every machine shown or grouped by machine) **and a project's row** (a repo's
heading, grouped by repo) stand for their coordinator, and are drawn as a task agent's row:

- **its coordinator's mark** in the marks' column, the same marks the chat's line has for a coordinator or an agent:
  working, idle, needs you, stopped, not started, and autopilot's `◎` while that is on;
- then its caret and its name, with **no rule** to the edge of the list;
- on the right how long ago its coordinator last said something (on [autopilot](autopilot.md) its timing:
  `2 of 5 added  last HSW-92  next in 40m`), and **under its name the last thing it said**. One that never said
  anything has no second line. No review tag, pull request or progress: a coordinator has none.
- `Enter` folds it and unfolds it, as any heading; `Space` talks to its coordinator.

**The filter Coordinators** (`Tab` from All, [the filter row](board.md#the-filter-row)) leaves only the Coordinator's
row, each machine's row and the rows of its projects that have a project coordinator, for the machine shown; its count
is how many there are. `Enter` on a machine's row there folds its projects' rows away.

## Talking to one

- **`Space` on its row** opens it in the chat. The chat names it (`herdr-projects coordinator  idle`), and so does the
  box: `Send message to herdr-projects coordinator`. `Enter` sends, and it decides what to start, as the Coordinator
  does.
- **A machine's row** shows with every machine (`Alt+M`, the filter row's first item) or grouped by machine (`Alt+S`);
  **a project's row** grouped by repo (`Alt+S`), and under Coordinators. The home folder's tasks have one too.
- **A repo with no tasks in view keeps its row**, folded (`0 tasks`), when it was used in the last 14 days (a task,
  archived ones too) or has a project coordinator, so `Space` still reaches it. Older repos, and repos whose folder is
  gone, have none.
- **It follows the selection**, as a task's agent does: move to another machine's or project's row and you talk to that
  one's coordinator, move to a task and you talk to its agent. `Space` again goes back to the Coordinator.
- A status group's heading (`to do`, `done`) has none. `Space` there says so.

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

## Its row's keys

As a task's row stands for the task:

- `Alt+P` shows its coordinator beside the list, `Alt+T` whole, in the [task panel](task-panel.md)'s form.
- `Ctrl+E` opens its settings page: its own rows first, under `This coordinator`, then on this machine's row your own
  settings, on a project's your entry for that repo.
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
