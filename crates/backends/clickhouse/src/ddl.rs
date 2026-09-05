use std::collections::BTreeMap;

use super::{
    DictionaryField, NamedCollectionEntry, RefreshSchedule, ResourceOperation, TableReference,
    TableTtl, TtlAction, TtlDestination, ViewRefresh, ViewSqlSecurity, WindowView, WorkloadSetting,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TableMetadata {
    pub engine_arguments: Vec<String>,
    pub engine_parameters: BTreeMap<String, String>,
    pub ttl_rules: Vec<TableTtl>,
    pub settings: BTreeMap<String, String>,
    pub column_ttls: BTreeMap<String, String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct DictionaryMetadata {
    pub fields: BTreeMap<String, DictionaryFieldMetadata>,
    pub range_min: Option<String>,
    pub range_max: Option<String>,
    pub settings: BTreeMap<String, String>,
    pub lifetime_min_seconds: Option<u64>,
    pub lifetime_max_seconds: Option<u64>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct DictionaryFieldMetadata {
    pub default_expression: Option<String>,
    pub expression: Option<String>,
    pub hierarchical: bool,
    pub injective: bool,
    pub object_id: bool,
}

pub(crate) fn table_metadata(engine: &str, engine_full: &str, definition: &str) -> TableMetadata {
    TableMetadata {
        engine_arguments: engine_arguments(engine, engine_full),
        engine_parameters: engine_parameters(engine, engine_full),
        ttl_rules: clause(engine_full, "TTL", &["SETTINGS"])
            .map(|value| {
                split_top_level(value, ',')
                    .into_iter()
                    .map(|rule| parse_ttl(&rule))
                    .collect()
            })
            .unwrap_or_default(),
        settings: clause(engine_full, "SETTINGS", &[])
            .map(parse_settings)
            .unwrap_or_default(),
        column_ttls: column_ttls(definition),
    }
}

pub(crate) fn view_refresh(database: &str, definition: &str) -> Option<ViewRefresh> {
    let refresh = top_level_keyword(definition, "REFRESH")? + "REFRESH".len();
    let remainder = definition[refresh..].trim_start();
    let end = [
        "APPEND",
        "TO",
        "ENGINE",
        "EMPTY",
        "DEFINER",
        "SQL SECURITY",
        "AS",
    ]
    .into_iter()
    .filter_map(|keyword| top_level_keyword(remainder, keyword))
    .min()
    .unwrap_or(remainder.len());
    let clause = remainder[..end].trim();
    let schedule = if starts_with_keyword(clause, "EVERY") {
        RefreshSchedule::Every {
            interval: segment_after(
                clause,
                "EVERY",
                &["OFFSET", "RANDOMIZE FOR", "DEPENDS ON", "SETTINGS"],
            ),
        }
    } else if starts_with_keyword(clause, "AFTER") {
        RefreshSchedule::After {
            interval: segment_after(
                clause,
                "AFTER",
                &["RANDOMIZE FOR", "DEPENDS ON", "SETTINGS"],
            ),
        }
    } else if starts_with_keyword(clause, "DEPENDS ON") {
        RefreshSchedule::DependenciesOnly
    } else {
        return None;
    };
    let offset = optional_segment(
        clause,
        "OFFSET",
        &["RANDOMIZE FOR", "DEPENDS ON", "SETTINGS"],
    );
    let randomize_for = optional_segment(clause, "RANDOMIZE FOR", &["DEPENDS ON", "SETTINGS"]);
    let dependencies = optional_segment(clause, "DEPENDS ON", &["SETTINGS"])
        .map(|value| {
            split_top_level(&value, ',')
                .into_iter()
                .filter_map(|name| table_reference(database, &name))
                .collect()
        })
        .unwrap_or_default();
    let settings = optional_segment(clause, "SETTINGS", &[])
        .map(|value| parse_settings(&value))
        .unwrap_or_default();
    Some(ViewRefresh {
        schedule,
        offset,
        randomize_for,
        dependencies,
        settings,
        append: top_level_keyword(definition, "APPEND").is_some_and(|position| {
            top_level_keyword(definition, "AS").is_none_or(|as_position| position < as_position)
        }),
    })
}

pub(crate) fn view_definer(definition: &str) -> Option<String> {
    let start = top_level_keyword(definition, "DEFINER")? + "DEFINER".len();
    let remainder = definition[start..].trim_start();
    let remainder = remainder
        .strip_prefix('=')
        .unwrap_or(remainder)
        .trim_start();
    let end = ["SQL SECURITY", "AS"]
        .into_iter()
        .filter_map(|keyword| top_level_keyword(remainder, keyword))
        .min()
        .unwrap_or(remainder.len());
    let value = remainder[..end].trim();
    (!value.is_empty()).then(|| unquote_identifier(value))
}

pub(crate) fn view_sql_security(definition: &str) -> Option<ViewSqlSecurity> {
    let start = top_level_keyword(definition, "SQL SECURITY")? + "SQL SECURITY".len();
    let value = definition[start..].split_whitespace().next()?;
    Some(match value.to_ascii_uppercase().as_str() {
        "DEFINER" => ViewSqlSecurity::Definer,
        "INVOKER" => ViewSqlSecurity::Invoker,
        "NONE" => ViewSqlSecurity::None,
        _ => ViewSqlSecurity::Unknown {
            raw: value.to_string(),
        },
    })
}

pub(crate) fn view_target(definition: &str) -> Option<String> {
    let position = top_level_keyword(definition, "TO")?;
    if top_level_keyword(definition, "AS").is_some_and(|as_position| position > as_position) {
        return None;
    }
    let remainder = definition[position + "TO".len()..].trim_start();
    let end = remainder
        .find(|character: char| character.is_whitespace() || character == '(')
        .unwrap_or(remainder.len());
    let target = remainder[..end].replace('`', "");
    (!target.is_empty()).then_some(target)
}

pub(crate) fn window_view(definition: &str) -> WindowView {
    WindowView {
        target: view_target(definition),
        inner_engine: optional_segment(
            definition,
            "INNER ENGINE",
            &["ENGINE", "WATERMARK", "ALLOWED_LATENESS", "POPULATE", "AS"],
        )
        .map(|value| value.strip_prefix('=').unwrap_or(&value).trim().to_string()),
        storage_engine: standalone_window_engine(definition),
        watermark: optional_segment(
            definition,
            "WATERMARK",
            &["ALLOWED_LATENESS", "POPULATE", "AS"],
        ),
        allowed_lateness: optional_segment(definition, "ALLOWED_LATENESS", &["POPULATE", "AS"]),
    }
}

fn standalone_window_engine(definition: &str) -> Option<String> {
    let position = scanner_positions(definition).find(|&position| {
        definition
            .get(position..position + "ENGINE".len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case("ENGINE"))
            && word_boundary(definition.as_bytes().get(position.wrapping_sub(1)).copied())
            && word_boundary(
                definition
                    .as_bytes()
                    .get(position + "ENGINE".len())
                    .copied(),
            )
            && !definition[..position]
                .trim_end()
                .to_ascii_uppercase()
                .ends_with("INNER")
    })?;
    let remainder = definition[position + "ENGINE".len()..].trim_start();
    let remainder = remainder
        .strip_prefix('=')
        .unwrap_or(remainder)
        .trim_start();
    let end = ["WATERMARK", "ALLOWED_LATENESS", "POPULATE", "AS"]
        .into_iter()
        .filter_map(|keyword| top_level_keyword(remainder, keyword))
        .min()
        .unwrap_or(remainder.len());
    let engine = remainder[..end].trim();
    (!engine.is_empty()).then(|| engine.to_string())
}

pub(crate) fn dictionary_metadata(definition: &str) -> DictionaryMetadata {
    let fields = dictionary_fields(definition);
    let (range_min, range_max) = dictionary_range(definition);
    let settings = parenthesized_clause(definition, "SETTINGS")
        .map(parse_settings)
        .unwrap_or_default();
    let (lifetime_min_seconds, lifetime_max_seconds) = dictionary_lifetime(definition);
    DictionaryMetadata {
        fields,
        range_min,
        range_max,
        settings,
        lifetime_min_seconds,
        lifetime_max_seconds,
    }
}

pub(crate) fn apply_dictionary_metadata(
    fields: &mut [DictionaryField],
    metadata: &DictionaryMetadata,
) {
    for field in fields {
        let Some(options) = metadata.fields.get(&field.name) else {
            continue;
        };
        field.default_expression = options.default_expression.clone();
        field.expression = options.expression.clone();
        field.hierarchical = options.hierarchical;
        field.injective = options.injective;
        field.object_id = options.object_id;
    }
}

fn dictionary_fields(definition: &str) -> BTreeMap<String, DictionaryFieldMetadata> {
    let Some(open) = top_level_character(definition, '(') else {
        return BTreeMap::new();
    };
    let Some(close) = matching_delimiter(definition, open, b'(', b')') else {
        return BTreeMap::new();
    };
    split_top_level(&definition[open + 1..close], ',')
        .into_iter()
        .filter_map(|field| {
            let (name, remainder) = leading_identifier(&field)?;
            let positions = [
                "DEFAULT",
                "EXPRESSION",
                "HIERARCHICAL",
                "INJECTIVE",
                "IS_OBJECT_ID",
            ]
            .into_iter()
            .filter_map(|keyword| {
                top_level_keyword(remainder, keyword).map(|position| (position, keyword))
            })
            .collect::<Vec<_>>();
            let mut metadata = DictionaryFieldMetadata {
                hierarchical: positions
                    .iter()
                    .any(|(_, keyword)| *keyword == "HIERARCHICAL"),
                injective: positions.iter().any(|(_, keyword)| *keyword == "INJECTIVE"),
                object_id: positions
                    .iter()
                    .any(|(_, keyword)| *keyword == "IS_OBJECT_ID"),
                ..DictionaryFieldMetadata::default()
            };
            metadata.default_expression =
                dictionary_field_expression(remainder, "DEFAULT", &positions);
            metadata.expression = dictionary_field_expression(remainder, "EXPRESSION", &positions);
            Some((name, metadata))
        })
        .collect()
}

fn dictionary_field_expression(
    value: &str,
    keyword: &str,
    positions: &[(usize, &str)],
) -> Option<String> {
    let start = positions
        .iter()
        .find(|(_, candidate)| *candidate == keyword)?
        .0
        + keyword.len();
    let end = positions
        .iter()
        .filter_map(|(position, _)| (*position > start).then_some(*position))
        .min()
        .unwrap_or(value.len());
    let expression = value[start..end].trim();
    (!expression.is_empty()).then(|| expression.to_string())
}

fn dictionary_range(definition: &str) -> (Option<String>, Option<String>) {
    let Some(range) = parenthesized_clause(definition, "RANGE") else {
        return (None, None);
    };
    let min = segment_after(range, "MIN", &["MAX"]);
    let max = optional_segment(range, "MAX", &[]);
    ((!min.is_empty()).then_some(min), max)
}

fn dictionary_lifetime(definition: &str) -> (Option<u64>, Option<u64>) {
    let Some(lifetime) = parenthesized_clause(definition, "LIFETIME") else {
        return (None, None);
    };
    if let Ok(seconds) = lifetime.parse::<u64>() {
        return (Some(seconds), Some(seconds));
    }
    let min = segment_after(lifetime, "MIN", &["MAX"]).parse().ok();
    let max = optional_segment(lifetime, "MAX", &[]).and_then(|value| value.parse().ok());
    (min, max)
}

fn parenthesized_clause<'a>(value: &'a str, keyword: &str) -> Option<&'a str> {
    let keyword_start = top_level_keyword(value, keyword)?;
    let remainder = &value[keyword_start + keyword.len()..];
    let relative_open = top_level_character(remainder, '(')?;
    if !remainder[..relative_open].trim().is_empty() {
        return None;
    }
    let close = matching_delimiter(remainder, relative_open, b'(', b')')?;
    Some(remainder[relative_open + 1..close].trim())
}

pub(crate) fn resource_operations(definition: &str) -> Vec<ResourceOperation> {
    let Some(open) = top_level_character(definition, '(') else {
        return Vec::new();
    };
    let Some(close) = matching_delimiter(definition, open, b'(', b')') else {
        return Vec::new();
    };
    split_top_level(&definition[open + 1..close], ',')
        .into_iter()
        .map(|operation| {
            let operation = operation.trim();
            if operation.eq_ignore_ascii_case("MASTER THREAD") {
                ResourceOperation::MasterThread
            } else if operation.eq_ignore_ascii_case("WORKER THREAD") {
                ResourceOperation::WorkerThread
            } else if operation.eq_ignore_ascii_case("QUERY") {
                ResourceOperation::Query
            } else if operation.eq_ignore_ascii_case("MEMORY RESERVATION") {
                ResourceOperation::MemoryReservation
            } else if operation.eq_ignore_ascii_case("READ ANY DISK") {
                ResourceOperation::ReadDisk { disk: None }
            } else if operation.eq_ignore_ascii_case("WRITE ANY DISK") {
                ResourceOperation::WriteDisk { disk: None }
            } else if let Some(disk) = case_insensitive_prefix(operation, "READ DISK") {
                ResourceOperation::ReadDisk {
                    disk: Some(unquote_identifier(disk)),
                }
            } else if let Some(disk) = case_insensitive_prefix(operation, "WRITE DISK") {
                ResourceOperation::WriteDisk {
                    disk: Some(unquote_identifier(disk)),
                }
            } else {
                ResourceOperation::Unknown {
                    raw: operation.to_string(),
                }
            }
        })
        .collect()
}

pub(crate) fn workload_settings(definition: &str) -> Vec<WorkloadSetting> {
    let Some(start) = top_level_keyword(definition, "SETTINGS") else {
        return Vec::new();
    };
    split_top_level(&definition[start + "SETTINGS".len()..], ',')
        .into_iter()
        .filter_map(|setting| {
            let resource_position = top_level_keyword(&setting, "FOR");
            let (assignment, resource) = resource_position.map_or_else(
                || (setting.as_str(), None),
                |position| {
                    (
                        setting[..position].trim(),
                        Some(unquote_identifier(&setting[position + "FOR".len()..])),
                    )
                },
            );
            let equals = top_level_character(assignment, '=')?;
            let name = assignment[..equals].trim();
            let value = assignment[equals + 1..].trim();
            (!name.is_empty() && !value.is_empty()).then(|| WorkloadSetting {
                name: name.to_string(),
                value: value.to_string(),
                resource,
            })
        })
        .collect()
}

pub(crate) fn named_collection_entries(
    keys: Vec<String>,
    definition: &str,
) -> Vec<NamedCollectionEntry> {
    let overrides = top_level_keyword(definition, "AS")
        .map(|position| &definition[position + "AS".len()..])
        .into_iter()
        .flat_map(|entries| split_top_level(entries, ','))
        .filter_map(|entry| {
            let (key, remainder) = leading_identifier(&entry)?;
            let overridable = if top_level_keyword(remainder, "NOT OVERRIDABLE").is_some() {
                Some(false)
            } else if top_level_keyword(remainder, "OVERRIDABLE").is_some() {
                Some(true)
            } else {
                None
            };
            Some((key, overridable))
        })
        .collect::<BTreeMap<_, _>>();

    keys.into_iter()
        .map(|key| NamedCollectionEntry {
            overridable: overrides.get(&key).copied().flatten(),
            key,
        })
        .collect()
}

fn case_insensitive_prefix<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .get(..prefix.len())
        .filter(|candidate| candidate.eq_ignore_ascii_case(prefix))
        .filter(|_| word_boundary(value.as_bytes().get(prefix.len()).copied()))
        .map(|_| value[prefix.len()..].trim())
        .filter(|value| !value.is_empty())
}

fn starts_with_keyword(value: &str, keyword: &str) -> bool {
    value
        .get(..keyword.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(keyword))
        && word_boundary(value.as_bytes().get(keyword.len()).copied())
}

fn optional_segment(value: &str, keyword: &str, following: &[&str]) -> Option<String> {
    top_level_keyword(value, keyword)
        .map(|position| segment_after(&value[position..], keyword, following))
        .filter(|value| !value.is_empty())
}

fn segment_after(value: &str, keyword: &str, following: &[&str]) -> String {
    let remainder = value[keyword.len()..].trim_start();
    let end = following
        .iter()
        .filter_map(|next| top_level_keyword(remainder, next))
        .min()
        .unwrap_or(remainder.len());
    remainder[..end].trim().to_string()
}

fn table_reference(default_database: &str, name: &str) -> Option<TableReference> {
    let parts = split_top_level(name.trim(), '.');
    match parts.as_slice() {
        [table] => Some(TableReference {
            database: default_database.to_string(),
            table: unquote_identifier(table),
        }),
        [database, table] => Some(TableReference {
            database: unquote_identifier(database),
            table: unquote_identifier(table),
        }),
        _ => None,
    }
}

fn unquote_identifier(value: &str) -> String {
    value
        .trim()
        .strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .map_or_else(
            || value.trim().to_string(),
            |value| value.replace("``", "`"),
        )
}

fn engine_arguments(engine: &str, engine_full: &str) -> Vec<String> {
    raw_engine_arguments(engine, engine_full)
        .into_iter()
        .filter(|argument| top_level_character(argument, '=').is_none())
        .collect()
}

fn engine_parameters(engine: &str, engine_full: &str) -> BTreeMap<String, String> {
    raw_engine_arguments(engine, engine_full)
        .into_iter()
        .filter_map(|argument| {
            let equals = top_level_character(&argument, '=')?;
            let name = argument[..equals].trim();
            let value = argument[equals + 1..].trim();
            (!name.is_empty() && !value.is_empty()).then(|| (name.to_string(), value.to_string()))
        })
        .collect()
}

fn raw_engine_arguments(engine: &str, engine_full: &str) -> Vec<String> {
    let remainder = engine_full
        .get(engine.len()..)
        .filter(|_| engine_full[..engine.len()].eq_ignore_ascii_case(engine))
        .unwrap_or("")
        .trim_start();
    if !remainder.starts_with('(') {
        return Vec::new();
    }
    matching_delimiter(remainder, 0, b'(', b')')
        .map(|end| split_top_level(&remainder[1..end], ','))
        .unwrap_or_default()
}

fn parse_ttl(raw: &str) -> TableTtl {
    let raw = raw.trim().to_string();
    let action_start = [
        "DELETE",
        "WHERE",
        "TO DISK",
        "TO VOLUME",
        "RECOMPRESS",
        "GROUP BY",
    ]
    .into_iter()
    .filter_map(|keyword| top_level_keyword(&raw, keyword).map(|position| (position, keyword)))
    .min_by_key(|(position, _)| *position);
    let Some((position, keyword)) = action_start else {
        return TableTtl {
            expression: raw.clone(),
            action: TtlAction::Delete { predicate: None },
            raw,
        };
    };
    let expression = raw[..position].trim().to_string();
    let action_text = raw[position..].trim();
    let action = match keyword {
        "DELETE" => TtlAction::Delete {
            predicate: top_level_keyword(action_text, "WHERE")
                .map(|where_position| {
                    action_text[where_position + "WHERE".len()..]
                        .trim()
                        .to_string()
                })
                .filter(|value| !value.is_empty()),
        },
        // ClickHouse 26.6 normalizes `DELETE WHERE predicate` to just
        // `WHERE predicate` in `system.tables.engine_full`.
        "WHERE" => TtlAction::Delete {
            predicate: Some(action_text["WHERE".len()..].trim().to_string()),
        },
        "TO DISK" => TtlAction::Move {
            destination: TtlDestination::Disk,
            target: action_text["TO DISK".len()..].trim().to_string(),
        },
        "TO VOLUME" => TtlAction::Move {
            destination: TtlDestination::Volume,
            target: action_text["TO VOLUME".len()..].trim().to_string(),
        },
        "RECOMPRESS" => TtlAction::Recompress {
            codec: action_text["RECOMPRESS".len()..].trim().to_string(),
        },
        "GROUP BY" => {
            let remainder = action_text["GROUP BY".len()..].trim();
            let set = top_level_keyword(remainder, "SET");
            TtlAction::GroupBy {
                keys: set
                    .map_or(remainder, |position| &remainder[..position])
                    .trim()
                    .to_string(),
                assignments: set
                    .map(|position| split_top_level(&remainder[position + "SET".len()..], ','))
                    .unwrap_or_default(),
            }
        }
        _ => TtlAction::Unknown {
            raw: action_text.to_string(),
        },
    };
    TableTtl {
        expression,
        action,
        raw,
    }
}

fn parse_settings(value: &str) -> BTreeMap<String, String> {
    split_top_level(value, ',')
        .into_iter()
        .filter_map(|setting| {
            let equals = top_level_character(&setting, '=')?;
            let name = setting[..equals].trim();
            let value = setting[equals + 1..].trim();
            (!name.is_empty()).then(|| (name.to_string(), value.to_string()))
        })
        .collect()
}

fn column_ttls(definition: &str) -> BTreeMap<String, String> {
    let Some(open) = top_level_character(definition, '(') else {
        return BTreeMap::new();
    };
    let Some(close) = matching_delimiter(definition, open, b'(', b')') else {
        return BTreeMap::new();
    };
    split_top_level(&definition[open + 1..close], ',')
        .into_iter()
        .filter_map(|column| {
            let (name, remainder) = leading_identifier(&column)?;
            let ttl = top_level_keyword(remainder, "TTL")?;
            let value = remainder[ttl + "TTL".len()..].trim();
            let end = ["COMMENT", "CODEC", "STATISTICS"]
                .into_iter()
                .filter_map(|keyword| top_level_keyword(value, keyword))
                .min()
                .unwrap_or(value.len());
            let expression = value[..end].trim();
            (!expression.is_empty()).then(|| (name, expression.to_string()))
        })
        .collect()
}

fn leading_identifier(value: &str) -> Option<(String, &str)> {
    let value = value.trim_start();
    if let Some(quoted) = value.strip_prefix('`') {
        let mut index = 0;
        while index < quoted.len() {
            let byte = quoted.as_bytes()[index];
            if byte == b'`' {
                if quoted.as_bytes().get(index + 1) == Some(&b'`') {
                    index += 2;
                    continue;
                }
                return Some((quoted[..index].replace("``", "`"), &quoted[index + 1..]));
            }
            index += 1;
        }
        return None;
    }
    let end = value.find(char::is_whitespace).unwrap_or(value.len());
    let name = &value[..end];
    (!matches!(
        name.to_ascii_uppercase().as_str(),
        "INDEX" | "CONSTRAINT" | "PROJECTION" | "PRIMARY"
    ))
    .then(|| (name.to_string(), &value[end..]))
}

fn clause<'a>(value: &'a str, keyword: &str, following: &[&str]) -> Option<&'a str> {
    let start = top_level_keyword(value, keyword)? + keyword.len();
    let remainder = &value[start..];
    let end = following
        .iter()
        .filter_map(|next| top_level_keyword(remainder, next))
        .min()
        .unwrap_or(remainder.len());
    Some(remainder[..end].trim())
}

fn split_top_level(value: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0;
    for position in top_level_positions(value, delimiter) {
        let part = value[start..position].trim();
        if !part.is_empty() {
            parts.push(part.to_string());
        }
        start = position + delimiter.len_utf8();
    }
    let part = value[start..].trim();
    if !part.is_empty() {
        parts.push(part.to_string());
    }
    parts
}

fn top_level_keyword(value: &str, keyword: &str) -> Option<usize> {
    scanner_positions(value).find(|&position| {
        value
            .get(position..position + keyword.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(keyword))
            && word_boundary(value.as_bytes().get(position.wrapping_sub(1)).copied())
            && word_boundary(value.as_bytes().get(position + keyword.len()).copied())
    })
}

fn top_level_character(value: &str, character: char) -> Option<usize> {
    top_level_positions(value, character).next()
}

fn top_level_positions(value: &str, character: char) -> impl Iterator<Item = usize> + '_ {
    scanner_positions(value).filter(move |&position| {
        value[position..]
            .chars()
            .next()
            .is_some_and(|value| value == character)
    })
}

fn scanner_positions(value: &str) -> impl Iterator<Item = usize> + use<> {
    let bytes = value.as_bytes();
    let mut positions = Vec::new();
    let mut depth = 0_u64;
    let mut quote = None::<u8>;
    let mut escaped = false;
    let mut position = 0;
    while position < bytes.len() {
        let byte = bytes[position];
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                position += 1;
                continue;
            }
            if byte == b'\\' && active_quote == b'\'' {
                escaped = true;
                position += 1;
                continue;
            }
            if byte == active_quote {
                if bytes.get(position + 1) == Some(&active_quote) {
                    position += 2;
                    continue;
                }
                quote = None;
            }
            position += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' | b'`' => quote = Some(byte),
            b'(' | b'[' | b'{' => {
                if depth == 0 {
                    positions.push(position);
                }
                depth += 1;
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    positions.push(position);
                }
            }
            _ if depth == 0 => positions.push(position),
            _ => {}
        }
        position += value[position..].chars().next().map_or(1, char::len_utf8);
    }
    positions.into_iter()
}

fn matching_delimiter(value: &str, open: usize, opening: u8, closing: u8) -> Option<usize> {
    let bytes = value.as_bytes();
    if bytes.get(open) != Some(&opening) {
        return None;
    }
    let mut depth = 0_u64;
    let mut quote = None;
    let mut escaped = false;
    for (offset, byte) in bytes[open..].iter().copied().enumerate() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' && active_quote == b'\'' {
                escaped = true;
            } else if byte == active_quote {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
        } else if byte == opening {
            depth += 1;
        } else if byte == closing {
            depth -= 1;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

fn word_boundary(byte: Option<u8>) -> bool {
    byte.is_none_or(|byte| !byte.is_ascii_alphanumeric() && byte != b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_engine_arguments_ttl_actions_settings_and_column_ttl() {
        let engine = "ReplacingMergeTree(version) TTL occurred_at + toIntervalDay(7) DELETE WHERE deleted = 1, occurred_at + toIntervalDay(30) TO VOLUME 'cold', occurred_at + toIntervalDay(60) RECOMPRESS CODEC(ZSTD(9)), occurred_at + toIntervalDay(90) GROUP BY tenant_id SET payload = any(payload) SETTINGS index_granularity = 4096, auto_statistics_types = 'minmax, uniq'";
        let definition = "CREATE TABLE analytics.events (`tenant_id` UInt64, `expires_at` DateTime TTL expires_at + toIntervalDay(1), `payload` String) ENGINE = ReplacingMergeTree(version)";
        let parsed = table_metadata("ReplacingMergeTree", engine, definition);
        assert_eq!(parsed.engine_arguments, ["version"]);
        assert!(parsed.engine_parameters.is_empty());
        assert_eq!(parsed.ttl_rules.len(), 4);
        assert!(matches!(
            parsed.ttl_rules[0].action,
            TtlAction::Delete { predicate: Some(ref value) } if value == "deleted = 1"
        ));
        assert!(matches!(
            parsed.ttl_rules[1].action,
            TtlAction::Move { destination: TtlDestination::Volume, ref target } if target == "'cold'"
        ));
        assert_eq!(parsed.settings["auto_statistics_types"], "'minmax, uniq'");
        assert_eq!(
            parsed.column_ttls["expires_at"],
            "expires_at + toIntervalDay(1)"
        );
    }

    #[test]
    fn parses_clickhouse_26_6_normalized_conditional_delete_and_quoted_settings() {
        let parsed = table_metadata(
            "MergeTree",
            "MergeTree ORDER BY id TTL ts + toIntervalDay(7) WHERE deleted = 1 SETTINGS index_granularity = 4096, custom = 'it''s, retained'",
            "CREATE TABLE audit.ttl_delete (`id` UInt64, `expires` DateTime TTL expires + toIntervalDay(1)) ENGINE = MergeTree ORDER BY id",
        );
        assert!(matches!(
            parsed.ttl_rules[0].action,
            TtlAction::Delete { predicate: Some(ref value) } if value == "deleted = 1"
        ));
        assert_eq!(parsed.ttl_rules[0].expression, "ts + toIntervalDay(7)");
        assert_eq!(parsed.settings["custom"], "'it''s, retained'");
    }

    #[test]
    fn parses_clickhouse_26_6_refreshable_materialized_view_contract() {
        let definition = "CREATE MATERIALIZED VIEW analytics.refresh_base REFRESH EVERY 1 HOUR OFFSET 5 MINUTE RANDOMIZE FOR 1 MINUTE DEPENDS ON analytics.upstream, `other`.`upstream` SETTINGS refresh_retries = 5 APPEND TO analytics.target (`id` UInt64) DEFINER = default SQL SECURITY DEFINER AS SELECT id FROM analytics.events";
        let refresh = view_refresh("analytics", definition).expect("refresh clause should parse");
        assert!(matches!(
            refresh.schedule,
            RefreshSchedule::Every { ref interval } if interval == "1 HOUR"
        ));
        assert_eq!(refresh.offset.as_deref(), Some("5 MINUTE"));
        assert_eq!(refresh.randomize_for.as_deref(), Some("1 MINUTE"));
        assert_eq!(refresh.dependencies.len(), 2);
        assert_eq!(refresh.dependencies[1].database, "other");
        assert_eq!(refresh.settings["refresh_retries"], "5");
        assert!(refresh.append);
    }

    #[test]
    fn parses_dependency_only_refresh_schedule() {
        let refresh = view_refresh(
            "analytics",
            "CREATE MATERIALIZED VIEW analytics.dependent REFRESH DEPENDS ON upstream TO analytics.target AS SELECT 1",
        )
        .expect("dependency-only refresh clause should parse");
        assert_eq!(refresh.schedule, RefreshSchedule::DependenciesOnly);
        assert_eq!(refresh.dependencies[0].database, "analytics");
        assert!(!refresh.append);
    }

    #[test]
    fn parses_resource_operations_and_scoped_workload_settings() {
        assert_eq!(
            resource_operations(
                "CREATE RESOURCE scheduler (MASTER THREAD, WORKER THREAD, QUERY, READ DISK `s3`, WRITE ANY DISK)"
            ),
            vec![
                ResourceOperation::MasterThread,
                ResourceOperation::WorkerThread,
                ResourceOperation::Query,
                ResourceOperation::ReadDisk {
                    disk: Some("s3".to_string())
                },
                ResourceOperation::WriteDisk { disk: None },
            ]
        );
        assert_eq!(
            workload_settings("CREATE WORKLOAD analytics IN all SETTINGS max_concurrent_threads = 8 FOR cpu, weight = 3"),
            vec![
                WorkloadSetting {
                    name: "max_concurrent_threads".to_string(),
                    value: "8".to_string(),
                    resource: Some("cpu".to_string()),
                },
                WorkloadSetting {
                    name: "weight".to_string(),
                    value: "3".to_string(),
                    resource: None,
                },
            ]
        );
    }

    #[test]
    fn separates_named_engine_parameters_from_positional_arguments() {
        let parsed = table_metadata(
            "S3",
            "S3('s3://bucket/archive.parquet', 'Parquet', storage_class_name = 'INTELLIGENT_TIERING')",
            "",
        );
        assert_eq!(
            parsed.engine_arguments,
            ["'s3://bucket/archive.parquet'", "'Parquet'"]
        );
        assert_eq!(
            parsed.engine_parameters["storage_class_name"],
            "'INTELLIGENT_TIERING'"
        );
    }

    #[test]
    fn parses_dictionary_field_range_lifetime_and_setting_contracts() {
        let parsed = dictionary_metadata(
            "CREATE DICTIONARY audit.tree (`id` UInt64 IS_OBJECT_ID, `parent_id` UInt64 DEFAULT 0 HIERARCHICAL, `name` String DEFAULT 'unknown' INJECTIVE, `derived` String EXPRESSION lowerUTF8(name)) PRIMARY KEY id SOURCE(CLICKHOUSE(TABLE 'src')) LIFETIME(MIN 30 MAX 60) LAYOUT(RANGE_HASHED()) RANGE(MIN valid_from MAX valid_to) SETTINGS(max_threads_for_updates = 4)",
        );

        assert!(parsed.fields["id"].object_id);
        assert_eq!(
            parsed.fields["parent_id"].default_expression.as_deref(),
            Some("0")
        );
        assert!(parsed.fields["parent_id"].hierarchical);
        assert!(parsed.fields["name"].injective);
        assert_eq!(
            parsed.fields["derived"].expression.as_deref(),
            Some("lowerUTF8(name)")
        );
        assert_eq!(parsed.range_min.as_deref(), Some("valid_from"));
        assert_eq!(parsed.range_max.as_deref(), Some("valid_to"));
        assert_eq!(parsed.settings["max_threads_for_updates"], "4");
        assert_eq!(parsed.lifetime_min_seconds, Some(30));
        assert_eq!(parsed.lifetime_max_seconds, Some(60));
    }

    #[test]
    fn parses_view_definer_and_sql_security() {
        let definition = "CREATE VIEW audit.secure (`id` UInt64) DEFINER = default SQL SECURITY INVOKER AS SELECT id FROM audit.src";
        assert_eq!(view_definer(definition).as_deref(), Some("default"));
        assert_eq!(
            view_sql_security(definition),
            Some(ViewSqlSecurity::Invoker)
        );
    }

    #[test]
    fn parses_window_view_execution_contract() {
        let definition = "CREATE WINDOW VIEW analytics.windowed TO analytics.results (`total` UInt64) INNER ENGINE = AggregatingMergeTree ORDER BY total ENGINE = MergeTree ORDER BY total WATERMARK ASCENDING ALLOWED_LATENESS toIntervalSecond('2') AS SELECT count() AS total FROM analytics.events GROUP BY tumble(ts, toIntervalSecond('5'))";
        let window = window_view(definition);
        assert_eq!(window.target.as_deref(), Some("analytics.results"));
        assert_eq!(
            window.inner_engine.as_deref(),
            Some("AggregatingMergeTree ORDER BY total")
        );
        assert_eq!(
            window.storage_engine.as_deref(),
            Some("MergeTree ORDER BY total")
        );
        assert_eq!(window.watermark.as_deref(), Some("ASCENDING"));
        assert_eq!(
            window.allowed_lateness.as_deref(),
            Some("toIntervalSecond('2')")
        );
    }

    #[test]
    fn parses_named_collection_override_flags_without_values() {
        let entries = named_collection_entries(
            vec!["host".to_string(), "password".to_string()],
            "CREATE NAMED COLLECTION analytics_remote AS host = '[HIDDEN]' OVERRIDABLE, password = '[HIDDEN]' NOT OVERRIDABLE",
        );
        assert_eq!(entries[0].key, "host");
        assert_eq!(entries[0].overridable, Some(true));
        assert_eq!(entries[1].key, "password");
        assert_eq!(entries[1].overridable, Some(false));
    }
}
