mod initial;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(initial::Migration)]
    }
}

pub use sea_orm_migration::migrator::MigratorTrait;
