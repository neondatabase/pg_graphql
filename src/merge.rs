use graphql_parser::query::{Field, Text, Value};
use indexmap::IndexMap;

use crate::parser_util::alias_or_name;

/// Merges duplicates in a vector of fields. The fields in the vector are added to a
/// map from field name to field. If a field with the same name already exists in the
/// map, the existing and new fields' children are combined into the existing field's
/// children. These children will be merged later when they are normalized.
///
/// The map is an `IndexMap` to ensure iteration order of the fields is preserved.
/// This prevents tests from being flaky due to field order changing between test runs.
pub fn merge<'a, 'b, T>(fields: Vec<Field<'a, T>>) -> Result<Vec<Field<'a, T>>, String>
where
    T: Text<'a> + Eq + AsRef<str>,
{
    let mut merged: IndexMap<String, Field<'a, T>> = IndexMap::new();

    for current_field in fields {
        let response_key = alias_or_name(&current_field);
        match merged.get_mut(&response_key) {
            Some(existing_field) => {
                if can_merge(&current_field, existing_field)? {
                    existing_field
                        .selection_set
                        .items
                        .extend(current_field.selection_set.items);
                }
            }
            None => {
                merged.insert(response_key, current_field);
            }
        }
    }

    let fields = merged.into_iter().map(|(_, field)| field).collect();

    Ok(fields)
}

fn can_merge<'a, T>(field_a: &Field<'a, T>, field_b: &Field<'a, T>) -> Result<bool, String>
where
    T: Text<'a> + Eq + AsRef<str>,
{
    if field_a.name != field_b.name {
        return Err(format!(
            "Fields `{}` and `{}` are different",
            field_a.name.as_ref(),
            field_b.name.as_ref(),
        ));
    }
    if !same_arguments(&field_a.arguments, &field_b.arguments) {
        return Err(format!(
            "Two fields named `{}` have different arguments",
            field_a.name.as_ref(),
        ));
    }

    Ok(true)
}

fn same_arguments<'a, 'b, T>(
    arguments_a: &[(T::Value, Value<'a, T>)],
    arguments_b: &[(T::Value, Value<'a, T>)],
) -> bool
where
    T: Text<'a> + Eq + AsRef<str>,
{
    if arguments_a.len() != arguments_b.len() {
        return false;
    }

    for (arg_a_name, arg_a_val) in arguments_a {
        for (arg_b_name, arg_b_val) in arguments_b {
            if arg_a_name == arg_b_name && arg_a_val != arg_b_val {
                return false;
            }
        }
    }

    true
}
