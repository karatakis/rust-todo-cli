# TODO_CLI

A TODO task management CLI application

## Demo

![Demo video gif](./docs/first.gif)

## Docs

```
A CLI application to manage TODO tasks

Usage: todo-cli [OPTIONS] <COMMAND>

Commands:
  task          All operations for tasks
  undo          Undo last operation
  redo          Redo last undo operation
  actions       List last actions
  housekeeping  1) Delete archived, 2) Delete actions log, 3) Archive all completed tasks
  category      All operations for task categories
  help          Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>  Specifies the storage file [default: data.db]
  -h, --help         Print help
  -V, --version      Print version
```

### Task

```
All operations for tasks

Usage: todo-cli task <COMMAND>

Commands:
  add     Add a new task
  delete  Delete an existing task
  update  Update an existing task
  list    List all the tasks based on query filters
  read    Read an existing task
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Category

```
All operations for task categories

Usage: todo-cli category <COMMAND>

Commands:
  list          List all categories
  add           Add a category to a task
  rename        Rename a category from a task
  remove        Remove a category from a task
  batch-rename  Batch rename a task
  batch-delete  Batch delete a task
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Testing

* `cargo tarpaulin --out Html` get test coverage report
* `./scripts/batch_insert.sh` batch insert many tasks for debug purposes

## TODOs

* CRUD tasks [DONE]
* * Create [DONE]
* * Update [DONE]
* * Delete [DONE]
* * Read [DONE]
* * List [DONE]
* Undo [DONE]
* Redo [DONE]
* Query operations [DONE]
* * Status [DONE]
* * Fuzzy Search [DONE]
* * Sorting [DONE]
* * By Created Date [DONE]
* * By Updated Date [DONE]
* * By Deadline Date [DONE]
* Add Categories Support [DONE]
* * Create [DONE]
* * Update [DONE]
* * Delete [DONE]
* * List [DONE]
* * Attach to Task [DONE]
* * Detach from to Task [DONE]
* Housekeeping Command [DONE]
* * Clear logs [DONE]
* * Clear archived [DONE]
* Archive completed command [DONE]
* Testing [DONE]
