use std::collections::HashMap;
use std::string::String;
use sea_orm::{ColumnTrait, Order, QueryFilter, QueryOrder, QuerySelect, Select};
use sea_orm::{DbErr, EntityTrait};
use serde::Deserialize;
use crate::entities::entity::Column;

#[derive(Deserialize)]
pub struct Sort {
    #[serde(rename = "colId")]
    pub col_id: String,
    pub sort: String,
}

#[derive(Deserialize)]
pub struct FieldFilter {
    #[serde(rename = "filterType")]
    pub filter_type: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub filter: String,
}

#[derive(Deserialize)]
pub struct FilterQuery {
    pub start: u64,
    pub end: u64,
    pub filter: HashMap<String, FieldFilter>,
    pub sort: Vec<Sort>,
    #[serde(rename = "globalSearch")]
    pub global_search: String,
}

impl FilterQuery {
    pub async fn apply_filters<E: EntityTrait>(
        &self,
        qs: Select<E>,
        global_searchable_fields: &Vec<Column>,
        column_map: &HashMap<&str, Column>,
    ) -> Result<Select<E>, DbErr> {
        let mut qs = qs;

        // Filter by each field
        for (name, filter) in &self.filter {
            let column= column_map.get(name.as_str()).unwrap();

            if filter.filter_type != "text" {
                return Err(DbErr::Custom("Unsupported filter type".to_string()));
            }

            match filter.kind.as_str() {
                "equals" => qs = qs.filter(column.eq(filter.filter.clone())),
                "notEquals" => qs = qs.filter(column.ne(filter.filter.clone())),
                "contains" => qs = qs.filter(column.contains(filter.filter.clone())),
                "notContains" => qs = qs.filter(column.contains(filter.filter.clone()).not()),
                "startsWith" => qs = qs.filter(column.starts_with(filter.filter.clone())),
                "endsWith" => qs = qs.filter(column.ends_with(filter.filter.clone())),
                "blank" => qs = qs.filter(column.is_null()),
                "notBlank" => qs = qs.filter(column.is_not_null()),
                _ => return Err(DbErr::Custom(format!(
                    "Opérateur non supporté: {}.",
                    filter.kind
                ))),
            }
        }

        // Global filter
        if !self.global_search.is_empty() {
            // This seems to be safe from injection as the builder replaces spaces with underscores
            for field in global_searchable_fields {
                qs = qs.filter(field.contains(&self.global_search));
            }
        }

        // Sorting
        if self.sort.len() > 0 {
            let sort = &self.sort.first().unwrap();
            let column= column_map.get(sort.col_id.as_str()).unwrap();

            qs = qs.order_by(*column, if sort.sort.to_lowercase() == "asc" { Order::Asc } else { Order::Desc });
        }

        // Paging
        qs = qs.offset(self.start).limit(self.end - self.start);

        Ok(qs)
    }
}