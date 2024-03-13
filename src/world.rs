use bevy::{ecs::component::Component, utils::HashMap};

const MIN_ROWS_ERROR: &str = "Grid row must have a minimum length of 2";
const MIN_COLS_ERROR: &str = "Grid column must have a minimum length of 2";
const COLS_LEN_CONSISTENCY_ERROR: &str = "All grid columns must be of the same length";

const MIN_ROWS: usize = 2;
const MIN_COLS: usize = 2;

const ROW_PRIME: usize = 22283;
const COL_PRIME: usize = 19709;

type Grid = Vec<Vec<u8>>;
type CellPosition = (usize, usize);
type LivingCellsCount = u8;

type Hash = usize;
type LiveCellMap = HashMap<Hash, CellPosition>;
type DeadCellMap = HashMap<Hash, (LivingCellsCount, CellPosition)>;

#[derive(Component)]
pub struct DullWorld {
    rows: usize,
    cols: usize,
    living_cells: LiveCellMap,
}

fn build_living_cell_key(row_index: usize, col_index: usize) -> usize {
    row_index * ROW_PRIME + col_index * COL_PRIME
}

fn build_map_from_grid(grid: &Grid) -> LiveCellMap {
    let mut living_cells: LiveCellMap = HashMap::new();

    for (row_index, row) in grid.iter().enumerate() {
        for (col_index, col) in row.iter().enumerate() {
            if *col == 1 {
                living_cells.insert(
                    build_living_cell_key(row_index, col_index),
                    (row_index, col_index),
                );
            }
        }
    }

    living_cells
}

impl DullWorld {
    pub fn dimensions(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn get_living_cells(&self) -> Vec<CellPosition> {
        self.living_cells
            .iter()
            .map(|(_, position)| *position)
            .collect()
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

        let living_cells = build_map_from_grid(&grid);

        Ok(Self {
            rows,
            cols,
            living_cells,
        })
    }

    fn process_neighbors(
        &self,
        row: usize,
        col: usize,
        dead_cells_neighboring_living_cells: &mut DeadCellMap,
    ) -> LivingCellsCount {
        let row_plus_one = (row + 1) % self.rows;
        let row_minus_one = (row + self.rows - 1) % self.rows;
        let col_plus_one = (col + 1) % self.cols;
        let col_minus_one = (col + self.cols - 1) % self.cols;

        let neighbor_positions = [
            (row_minus_one, col_minus_one),
            (row_minus_one, col),
            (row_minus_one, col_plus_one),
            (row, col_minus_one),
            (row, col_plus_one),
            (row_plus_one, col_minus_one),
            (row_plus_one, col),
            (row_plus_one, col_plus_one),
        ];

        neighbor_positions
            .iter()
            .fold(0, |living_neighbors_count, (row_index, col_index)| {
                let next_key = build_living_cell_key(*row_index, *col_index);
                if self.living_cells.contains_key(&next_key) {
                    return living_neighbors_count + 1;
                }

                if let Some(entry) = dead_cells_neighboring_living_cells.get_mut(&next_key) {
                    *entry = (entry.0 + 1, (*row_index, *col_index));
                } else {
                    dead_cells_neighboring_living_cells
                        .insert(next_key, (1, (*row_index, *col_index)));
                }

                living_neighbors_count
            })
    }

    pub fn step(&mut self) {
        let mut dead_cells_neighboring_living_cells: DeadCellMap = HashMap::new();
        let mut next_generation: LiveCellMap = HashMap::new();

        for (key, (row_index, col_index)) in self.living_cells.iter() {
            let living_neighbors_count = self.process_neighbors(
                *row_index,
                *col_index,
                &mut dead_cells_neighboring_living_cells,
            );

            if living_neighbors_count != 2 && living_neighbors_count != 3 {
                continue;
            }
            next_generation.insert(*key, (*row_index, *col_index));
        }

        for (key, (living_neighbors_count, (row_index, col_index))) in
            dead_cells_neighboring_living_cells.iter()
        {
            if *living_neighbors_count != 3 {
                continue;
            }
            next_generation.insert(*key, (*row_index, *col_index));
        }

        self.living_cells = next_generation;
    }

    pub fn is_live(&self, row_index: usize, col_index: usize) -> bool {
        self.living_cells
            .contains_key(&build_living_cell_key(row_index, col_index))
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
        let mut living_cells = result.unwrap().get_living_cells();
        living_cells.sort();

        assert_eq!(
            living_cells,
            [(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2)]
        );
    }

    /// Any live cell with fewer than two live neighbours dies, as if caused by underpopulation.
    #[test]
    fn it_should_die_if_less_than_two_live_neighbours() {
        let config = vec![vec![0, 0, 0], vec![0, 1, 0], vec![0, 0, 0]];
        let result = DullWorld::from_config(config);

        assert!(result.is_ok());
        let mut world = result.unwrap();
        world.step();

        assert_eq!(world.get_living_cells(), []);
    }

    #[test]
    fn it_should_not_die_if_two_or_three_live_neighbours() {
        let config = vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 1, 1, 1, 0],
            vec![0, 0, 0, 0, 0],
        ];
        let result = DullWorld::from_config(config);

        assert!(result.is_ok());
        let mut world = result.unwrap();
        world.step();

        let mut living_cells = world.get_living_cells();
        living_cells.sort();

        assert_eq!(living_cells, [(0, 2), (1, 2), (2, 2)]);
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

        let mut living_cells = world.get_living_cells();
        living_cells.sort();

        assert_eq!(
            living_cells,
            [
                (1, 1),
                (1, 2),
                (1, 3),
                (2, 1),
                (2, 3),
                (3, 1),
                (3, 2),
                (3, 3)
            ]
        );
    }

    #[test]
    fn it_should_revive_if_exactly_three_live_neighbours() {
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

        let mut living_cells = world.get_living_cells();
        living_cells.sort();

        assert_eq!(living_cells, [(2, 1), (2, 2), (2, 3)]);
    }
}
