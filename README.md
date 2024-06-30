# TODO_CLI

A TODO task management CLI application

## Snippets

```rust
pub fn task_exists(&self, id: i64) -> Result<bool> {
    let sql = Query::select()
        .expr(Func::sum(Expr::col((TaskIden::Table, TaskIden::Id))))
        .from(TaskIden::Table)
        .and_where(Expr::col(TaskIden::Id).eq(id))
        .to_string(SqliteQueryBuilder);

    let count: usize = self.conn.query_row(&sql, (), |row| Ok(row.get(0)?))?;

    if count > 0 {
        Ok(true)
    } else {
        Ok(false)
    }
}
```

## TODOs

* CRUD tasks [PENDING]
* * Create [DONE]
* * Update [DONE]
* * Delete [DONE]
* * Read [DONE]
* * List [PENDING]
* Undo [DONE]
* Redo [DONE]
* Query operations [PENDING]
* * Status [PENDING]
* * Fuzzy Search [PENDING]
* * Sorting [PENDING]
* * By Created Date [PENDING]
* * By Updated Date [PENDING]
* * By Deadline Date [PENDING]
* Add Categories Support [Done]
* * Create [Done]
* * Update [Done]
* * Delete [Done]
* * List [Done]
* * Attach to Task [Done]
* * Detach from to Task [Done]
* Housekeeping Command [PLANED]
* * Clear logs [PLANED]
* * Clear archived [PLANED]
* Archive completed command [PLANED]