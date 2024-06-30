use sea_query::Iden;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Iden)]
pub enum TaskCategoryIden {
    #[iden = "task_categories"]
    Table,
    TaskId,
    Category,
}
