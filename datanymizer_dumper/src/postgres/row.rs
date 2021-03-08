use crate::Table;
use anyhow::Result;
use datanymizer_engine::Engine;
use postgres::types::Type;
use std::char;

#[derive(Debug)]
pub struct PgRow<T>
where
    T: Table<Type>,
{
    table: T,
    source: String,
}

impl<T> PgRow<T>
where
    T: Table<Type>,
{
    pub fn from_string_row(source: String, parent_table: T) -> Self {
        Self {
            source,
            table: parent_table,
        }
    }

    /// Applies the transform engine to every column in the row
    /// Returns a new StringRecord for store in the dump
    pub fn transform(&self, engine: &Engine) -> Result<String> {
        let split_char: char = char::from_u32(0x0009).unwrap();
        let values: Vec<_> = self.source.split(split_char).collect();
        let transformed_values = engine
            .process_row(
                self.table.get_name(),
                self.table.get_column_indexes(),
                &values,
            )?
            .iter()
            .map(|v| {
                let s = v.replace("\t", "\\t");
                dbg!(&s);
                s
            })
            .collect::<Vec<_>>()
            .join("\t");

        Ok(transformed_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::{column::PgColumn, table::PgTable};
    use datanymizer_engine::Settings;

    mod transform {
        use super::*;

        #[test]
        fn sample() {
            let config = r#"
              source: {}
              tables:
                - name: table_name
                  rules:
                    first_name:
                      capitalize: ~
                    middle_name:
                      capitalize: ~
                    last_name:
                      capitalize: ~
            "#;
            let settings = Settings::from_yaml(config, String::new()).unwrap();

            let mut table = PgTable::new("table_name".to_string(), None);

            let col1 = PgColumn {
                position: 1,
                name: String::from("first_name"),
                data_type: String::new(),
                inner_type: Some(0),
            };
            let col2 = PgColumn {
                position: 2,
                name: String::from("middle_name"),
                data_type: String::new(),
                inner_type: Some(0),
            };
            let col3 = PgColumn {
                position: 3,
                name: String::from("last_name"),
                data_type: String::new(),
                inner_type: Some(0),
            };

            table.set_columns(vec![col1, col2, col3]);
            let row = PgRow::from_string_row("first\tmiddle\tlast".to_string(), table);

            assert_eq!(
                row.transform(&Engine::new(settings)).unwrap(),
                "First\tMiddle\tLast"
            );
        }

        #[test]
        fn tab_character() {
            let config = format!(
                r#"
                  source: {{}}
                  tables:
                    - name: table_name
                      rules:
                        text:
                          template:
                            format: {}
                "#,
                "text\twith\ttabs"
            );
            let settings = Settings::from_yaml(config.as_str(), String::new()).unwrap();

            let mut table = PgTable::new("table_name".to_string(), None);

            let col = PgColumn {
                position: 1,
                name: String::from("text"),
                data_type: String::new(),
                inner_type: Some(0),
            };

            table.set_columns(vec![col]);
            let row = PgRow::from_string_row("text".to_string(), table);
            let transformed_row = row.transform(&Engine::new(settings)).unwrap();
            dbg!(&transformed_row);

            assert_eq!(transformed_row.split("\t").collect::<Vec<_>>().len(), 1);
        }
    }
}
