//! Shared utilities for rs-grid examples.
//!
//! Provides fake data generation and a reusable
//! `build_model` function so that each example only
//! needs to wire up its own UI / framework integration.

/// Deterministic fake data generation for examples and tests.
pub mod fake_data;

use rs_grid_core::{
    column::{CellEditor, CellFormat, ColumnDef, SelectOption},
    datasource::FnDataSource,
    model::GridModel,
};

/// Build a [`GridModel`] backed by deterministic fake
/// data with the given number of rows and columns.
pub fn build_model(row_count: u64, col_count: usize) -> GridModel {
    let base: Vec<ColumnDef> = vec![
        ColumnDef::new("name", "Name", 200.0),
        ColumnDef::new("email", "Email", 260.0),
        ColumnDef::new("role", "Role", 140.0),
        ColumnDef::new("dept", "Department", 160.0),
        {
            let mut c = ColumnDef::new("salary", "Salary", 120.0);
            c.format = Some(CellFormat::Currency {
                symbol: "$".into(),
                decimal_places: 2,
                thousands_sep: Some(','),
                symbol_after: false,
            });
            c
        },
        {
            let mut c = ColumnDef::new("active", "Active", 80.0);
            c.format = Some(CellFormat::Boolean {
                true_label: "\u{2713}".into(),
                false_label: "\u{2717}".into(),
            });
            c
        },
        {
            let mut c = ColumnDef::new("avatar", "Avatar", 60.0);
            c.format = Some(CellFormat::Image {
                base_url: Some(
                    "https://ui-avatars.com/api/?size=40&name=".into(),
                ),
                border_radius: 16.0,
                padding: 4.0,
            });
            c
        },
    ];

    let mut columns: Vec<ColumnDef> =
        base.into_iter().take(col_count.min(7)).collect();

    let extras_needed = col_count.saturating_sub(7);
    for col in fake_data::EXTRA_COLUMNS.iter().take(extras_needed) {
        let mut c = ColumnDef::new(col.key, col.label, col.width);
        c.format = match col.format_hint {
            fake_data::FormatHint::Text => None,
            fake_data::FormatHint::Integer => Some(CellFormat::Number {
                decimal_places: 0,
                thousands_sep: Some(' '),
                decimal_sep: '.',
            }),
            fake_data::FormatHint::Currency => Some(CellFormat::Currency {
                symbol: "$".into(),
                decimal_places: 0,
                thousands_sep: Some(','),
                symbol_after: false,
            }),
            fake_data::FormatHint::Percent => {
                Some(CellFormat::Percent { decimal_places: 0 })
            }
            fake_data::FormatHint::Boolean => Some(CellFormat::Boolean {
                true_label: "\u{2713}".into(),
                false_label: "\u{2717}".into(),
            }),
            fake_data::FormatHint::ImageText => Some(CellFormat::ImageText {
                base_url: String::new(),
                suffix: String::new(),
                image_size: 20.0,
                border_radius: 2.0,
                gap: 6.0,
            }),
        };
        columns.push(c);
    }

    // Dynamic columns beyond the 92 hand-crafted extras
    let dynamic_needed = col_count.saturating_sub(7 + fake_data::EXTRA_COUNT);
    for i in 0..dynamic_needed {
        let (key, label, width, hint) = fake_data::dynamic_col_def(i);
        let mut c = ColumnDef::new(&key, &label, width);
        c.format = match hint {
            fake_data::FormatHint::Integer => Some(CellFormat::Number {
                decimal_places: 0,
                thousands_sep: Some(' '),
                decimal_sep: '.',
            }),
            fake_data::FormatHint::Currency => Some(CellFormat::Currency {
                symbol: "$".into(),
                decimal_places: 0,
                thousands_sep: Some(','),
                symbol_after: false,
            }),
            fake_data::FormatHint::Percent => {
                Some(CellFormat::Percent { decimal_places: 0 })
            }
            fake_data::FormatHint::Boolean => Some(CellFormat::Boolean {
                true_label: "\u{2713}".into(),
                false_label: "\u{2717}".into(),
            }),
            _ => None,
        };
        columns.push(c);
    }

    // Wire up a Select editor for the country column.
    if let Some(col) = columns.iter_mut().find(|c| c.key == "country") {
        let options: Vec<SelectOption> = fake_data::COUNTRIES
            .iter()
            .map(|(code, name)| {
                let uri = rs_grid_icons::flag_data_uri(code).unwrap_or("");
                SelectOption {
                    value: format!("{uri} {name}"),
                    label: name.to_string(),
                    icon: rs_grid_icons::flag_data_uri(code)
                        .map(|s| s.to_string()),
                }
            })
            .collect();
        col.editor = Some(CellEditor::Select { options });
    }

    // Wire up a Select editor for the gender column.
    if let Some(col) = columns.iter_mut().find(|c| c.key == "gender") {
        let options: Vec<SelectOption> = fake_data::GENDERS
            .iter()
            .map(|&label| {
                let key = label.to_uppercase().replace(' ', "-");
                let uri = rs_grid_icons::gender_icon_uri(&key).unwrap_or("");
                SelectOption {
                    value: format!("{uri} {label}"),
                    label: label.to_string(),
                    icon: rs_grid_icons::gender_icon_uri(&key)
                        .map(|s| s.to_string()),
                }
            })
            .collect();
        col.editor = Some(CellEditor::Select { options });
    }

    let source =
        FnDataSource::new(row_count, move |row: u64, col_key: &str| {
            fake_data::fake_cell(row, col_key)
        });

    GridModel::with_data_source(columns, Box::new(source), 40.0, 60.0)
}
