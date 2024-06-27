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

## TODOS

* CRUD tasks [UNDONE]
* * Create [DONE]
* * Update [DONE]
* * Delete [DONE]
* * Read [DONE]
* * List [UNDONE]
* Undo [UNDONE]
* Redo [UNDONE]
* Query operations [UNDONE]
* * Status [UNDONE]
* * Fuzzy search [UNDONE]
* * Sorting [UNDONE]
* * By Created Date [UNDONE]
* * By Updated Date [UNDONE]
* * By Deadline Date [UNDONE]
* Add categories support [UNDONE]
* Add comments support [UNDONE]