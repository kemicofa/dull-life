use bevy::ecs::component::Component;

const MIN_ROWS_ERROR: &str = "Grid row must have a minimum length of 2";
const MIN_COLS_ERROR: &str = "Grid column must have a minimum length of 2";
const COLS_LEN_CONSISTENCY_ERROR: &str = "All grid columns must be of the same length";

const MIN_ROWS: usize = 2;
const MIN_COLS: usize = 2;

type Grid = Vec<Vec<u8>>;

#[derive(Component)]
pub struct DullWorld {
    rows: usize,
    cols: usize,
    grid: Grid,
}

impl DullWorld {
    pub fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn get_grid(&self) -> &Grid {
        &self.grid
    }

    pub fn from_config(grid: Grid) -> Result<Self, String> {
        let rows = grid.len();

        if rows < MIN_ROWS {
            return Err(MIN_ROWS_ERROR.into());
        }

        let cols = grid[0].len();

        if cols < MIN_COLS {
            return Err(MIN_COLS_ERROR.into());
        }

        let all_cols_match = grid
            .iter()
            .all(|col| col.len() == cols && col.iter().all(|&cell| cell == 0 || cell == 1));

        if !all_cols_match {
            return Err(COLS_LEN_CONSISTENCY_ERROR.into());
        }

        Ok(Self { rows, cols, grid })
    }

    fn get_living_cells_count(&self, row: usize, col: usize) -> u8 {
        // (-1, -1) -> (-1, 1),
        // (0, -1) + (0, 1)
        // (1, -1) -> (1, 1)

        let row_plus_one = (row + 1) % self.rows;
        let row_minus_one = (row + self.rows - 1) % self.rows;
        let col_plus_one = (col + 1) % self.cols;
        let col_minus_one = (col + self.cols - 1) % self.cols;

        self.grid[row_minus_one][col_minus_one]
            + self.grid[row_minus_one][col]
            + self.grid[row_minus_one][col_plus_one]
            + self.grid[row][col_minus_one]
            + self.grid[row][col_plus_one]
            + self.grid[row_plus_one][col_minus_one]
            + self.grid[row_plus_one][col]
            + self.grid[row_plus_one][col_plus_one]
    }

    pub fn step(&mut self) {
        let mut next_generation = vec![vec![0; self.cols]; self.rows];

        for (row_index, row) in self.grid.iter().enumerate() {
            for (col_index, cell) in row.iter().enumerate() {
                let living_cells_count = self.get_living_cells_count(row_index, col_index);

                match cell {
                    0 => match living_cells_count {
                        // Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
                        3 => {
                            next_generation[row_index][col_index] = 1;
                        }
                        _ => {
                            next_generation[row_index][col_index] = 0;
                        }
                    },
                    1 => match living_cells_count {
                        // Any live cell with fewer than two live neighbours dies, as if caused by underpopulation.
                        0..=1 => {
                            next_generation[row_index][col_index] = 0;
                        }
                        // Any live cell with two or three live neighbours lives on to the next generation.
                        2..=3 => {
                            next_generation[row_index][col_index] = 1;
                        }
                        // Any live cell with more than three live neighbours dies, as if by overpopulation.
                        _ => {
                            next_generation[row_index][col_index] = 0;
                        }
                    },
                    _ => panic!("A cell had a different value than 0 or 1"),
                }
            }
        }

        self.grid = next_generation;
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn it_should_not_create_world_if_rows_too_small() {
        let result = DullWorld::from_config(vec![vec![]]);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), MIN_ROWS_ERROR);
    }

    #[test]
    fn it_should_not_create_world_if_cols_too_small() {
        let result = DullWorld::from_config(vec![vec![0], vec![0]]);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), MIN_COLS_ERROR);
    }

    #[test]
    fn it_should_not_create_world_if_cols_len_are_inconsitent() {
        let result = DullWorld::from_config(vec![vec![0, 0], vec![0, 0, 0]]);

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), COLS_LEN_CONSISTENCY_ERROR);
    }

    #[test]
    fn it_should_create_world() {
        let result = DullWorld::from_config(vec![vec![1; 3]; 2]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().grid, vec![vec![1, 1, 1], vec![1, 1, 1]]);
    }

    /// Any live cell with fewer than two live neighbours dies, as if caused by underpopulation.
    #[test]
    fn it_should_die_if_less_than_two_live_neighbours() {
        let config = vec![vec![0, 0, 0], vec![0, 1, 0], vec![0, 0, 0]];
        let result = DullWorld::from_config(config);

        assert!(result.is_ok());
        let mut world = result.unwrap();
        world.step();

        assert_eq!(
            world.grid,
            vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0],]
        );
    }

    #[test]
    fn it_should_not_die_if_two_or_three_live_neighbours() {
        let config = vec![vec![0, 0, 0], vec![0, 1, 0], vec![0, 0, 0]];
        let result = DullWorld::from_config(config);

        assert!(result.is_ok());
        let mut world = result.unwrap();
        world.step();

        assert_eq!(
            world.grid,
            vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0],]
        );
    }

    #[test]
    fn it_should_die_if_more_than_three_live_neighbours() {
        let config = vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 1, 1, 1, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0],
        ];
        let result = DullWorld::from_config(config);

        assert!(result.is_ok());
        let mut world = result.unwrap();
        world.step();

        assert_eq!(
            world.grid,
            vec![
                vec![0, 0, 0, 0, 0],
                vec![0, 1, 1, 1, 0],
                vec![0, 1, 0, 1, 0],
                vec![0, 1, 1, 1, 0],
                vec![0, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn it_should_revice_if_exactly_three_live_neighbours() {
        let config = vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0],
        ];
        let result = DullWorld::from_config(config);

        assert!(result.is_ok());
        let mut world = result.unwrap();
        world.step();

        assert_eq!(
            world.grid,
            vec![
                vec![0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0],
                vec![0, 1, 1, 1, 0],
                vec![0, 0, 0, 0, 0],
                vec![0, 0, 0, 0, 0],
            ]
        );
    }
}
