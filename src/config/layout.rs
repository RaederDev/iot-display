use crate::config::{Column, Row, Widget};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    AddRow,
    RemoveRow(usize),
    SwapRows(usize, usize),
    AddColumn(usize),
    RemoveColumn(usize, usize),
    SwapColumns(usize, usize, usize),
    AddWidget(usize, usize, Widget),
    RemoveWidget(usize, usize, usize),
    SwapWidgets(usize, usize, usize, usize),
    MoveWidget {
        row: usize,
        column: usize,
        widget: usize,
        target_row: usize,
        target_column: usize,
    },
}

pub fn apply(rows: &mut Vec<Row>, action: Action) {
    match action {
        Action::AddRow => rows.push(Row::default()),
        Action::RemoveRow(row) => {
            if rows.len() > 1 {
                rows.remove(row);
            }
        }
        Action::SwapRows(a, b) => rows.swap(a, b),
        Action::AddColumn(row) => rows[row].columns.push(Column::default()),
        Action::RemoveColumn(row, column) => {
            if rows[row].columns.len() > 1 {
                rows[row].columns.remove(column);
            }
        }
        Action::SwapColumns(row, a, b) => rows[row].columns.swap(a, b),
        Action::AddWidget(row, column, widget) => rows[row].columns[column].widgets.push(widget),
        Action::RemoveWidget(row, column, widget) => {
            rows[row].columns[column].widgets.remove(widget);
        }
        Action::SwapWidgets(row, column, a, b) => rows[row].columns[column].widgets.swap(a, b),
        Action::MoveWidget {
            row,
            column,
            widget,
            target_row,
            target_column,
        } => {
            let widget = rows[row].columns[column].widgets.remove(widget);
            rows[target_row].columns[target_column].widgets.push(widget);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_widgets_and_reordering_preserves_content() {
        let mut rows = crate::config::Config::default().layout;
        apply(
            &mut rows,
            Action::MoveWidget {
                row: 1,
                column: 0,
                widget: 0,
                target_row: 1,
                target_column: 1,
            },
        );
        assert_eq!(rows[1].columns[0].widgets, vec![Widget::PiHole]);
        assert_eq!(
            rows[1].columns[1].widgets,
            vec![Widget::Departures, Widget::Weather]
        );
        apply(&mut rows, Action::SwapWidgets(1, 1, 0, 1));
        assert_eq!(rows[1].columns[1].widgets[0], Widget::Weather);
        apply(&mut rows, Action::RemoveRow(0));
        apply(&mut rows, Action::RemoveRow(0));
        assert_eq!(rows.len(), 1);
    }
}
