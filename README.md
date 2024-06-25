# TODO_CLI

A TODO task management CLI application

## Snippets

```rust
use time::OffsetDateTime;
use std::time::SystemTime;

// Get current time
let now = SystemTime::now();
let now = OffsetDateTime::from(now);
// Format the date as YYYY-MM-DD
let now = &format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.date());
```

## TODOS

* CRUD tasks
* Query operations
* Add categories support
* Add comments support