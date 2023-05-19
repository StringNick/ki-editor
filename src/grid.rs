use crossterm::style::Color;
use ropey::{Rope, RopeSlice};
use tree_sitter::Point;

use crate::{
    rectangle::{Border, BorderDirection, Rectangle},
    screen::Dimension,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub rows: Vec<Vec<Cell>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cell {
    pub symbol: String,
    pub foreground_color: Color,
    pub background_color: Color,
}

impl Cell {
    fn from_char(c: char) -> Self {
        Cell {
            symbol: c.to_string(),
            foreground_color: Color::White,
            background_color: Color::White,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            symbol: " ".to_string(),
            foreground_color: Color::White,
            background_color: Color::White,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PositionedCell {
    pub cell: Cell,
    pub position: Point,
}

impl Grid {
    /// The `new_grid` need not be the same size as the old grid (`self`).
    pub fn diff(&self, new_grid: &Grid) -> Vec<PositionedCell> {
        let mut cells = vec![];
        for (row_index, new_row) in new_grid.rows.iter().enumerate() {
            for (column_index, new_cell) in new_row.iter().enumerate() {
                match self
                    .rows
                    .get(row_index)
                    .map(|old_row| old_row.get(column_index))
                    .flatten()
                {
                    Some(old_cell) if new_cell == old_cell => {
                        // Do nothing
                    }
                    // Otherwise
                    _ => cells.push(PositionedCell {
                        cell: new_cell.clone(),
                        position: Point::new(row_index as usize, column_index as usize),
                    }),
                }
            }
        }
        cells
    }

    pub fn new(dimension: Dimension) -> Grid {
        let mut cells: Vec<Vec<Cell>> = vec![];
        cells.resize_with(dimension.height.into(), || {
            let mut cells = vec![];
            cells.resize_with(dimension.width.into(), || Cell::default());
            cells
        });
        Grid { rows: cells }
    }

    pub fn to_position_cells(&self) -> Vec<PositionedCell> {
        let mut cells = vec![];
        for (row_index, row) in self.rows.iter().enumerate() {
            for (column_index, cell) in row.iter().enumerate() {
                cells.push(PositionedCell {
                    cell: cell.clone(),
                    position: Point::new(row_index as usize, column_index as usize),
                })
            }
        }

        cells
    }

    fn from_text(dimension: Dimension, text: &str) -> Grid {
        Grid::from_rope(dimension, &Rope::from_str(text))
    }

    fn from_rope(dimension: Dimension, rope: &Rope) -> Grid {
        let mut grid = Grid::new(dimension);

        rope.lines().enumerate().for_each(|(row_index, line)| {
            line.chars()
                .enumerate()
                .for_each(|(column_index, character)| {
                    grid.rows[row_index][column_index] = Cell {
                        symbol: character.to_string(),
                        ..Cell::default()
                    }
                })
        });

        grid
    }

    pub fn update(self, other: &Grid, rectangle: Rectangle) -> Grid {
        let mut grid = self;
        for (row_index, rows) in other.rows.iter().enumerate() {
            for (column_index, cell) in rows.iter().enumerate() {
                grid.rows[row_index + rectangle.origin.row]
                    [column_index + rectangle.origin.column] = cell.clone();
            }
        }
        grid
    }

    pub fn set_border(mut self, border: Border) -> Grid {
        let dimension = self.dimension();
        match border.direction {
            BorderDirection::Horizontal => {
                for i in 0..dimension.width.saturating_sub(border.start.column as u16) {
                    self.rows[border.start.row][border.start.column + i as usize] = Cell {
                        symbol: "─".to_string(),
                        foreground_color: Color::Black,
                        ..Cell::default()
                    };
                }
            }
            BorderDirection::Vertical => {
                for i in 0..dimension.height.saturating_sub(border.start.row as u16) {
                    self.rows[border.start.row + i as usize][border.start.column] = Cell {
                        symbol: "│".to_string(),
                        foreground_color: Color::Black,
                        ..Cell::default()
                    };
                }
            }
        }
        self
    }

    fn dimension(&self) -> Dimension {
        Dimension {
            height: self.rows.len() as u16,
            width: self.rows[0].len() as u16,
        }
    }
}

#[cfg(test)]
mod test_grid {
    use tree_sitter::Point;

    use pretty_assertions::assert_eq;

    use crate::{
        grid::{Cell, Grid, PositionedCell},
        screen::Dimension,
    };

    #[test]
    fn diff_same_size() {
        let dimension = Dimension {
            height: 2,
            width: 4,
        };
        let old = Grid::from_text(dimension, "a\nbc");
        let new = Grid::from_text(dimension, "bc");
        let actual = old.diff(&new);
        let expected = vec![
            PositionedCell {
                position: Point { row: 0, column: 0 },
                cell: Cell::from_char('b'),
            },
            PositionedCell {
                position: Point { row: 0, column: 1 },
                cell: Cell::from_char('c'),
            },
            PositionedCell {
                position: Point { row: 1, column: 0 },
                cell: Cell::from_char(' '),
            },
            PositionedCell {
                position: Point { row: 1, column: 1 },
                cell: Cell::from_char(' '),
            },
        ];
        assert_eq!(actual, expected);
    }
}