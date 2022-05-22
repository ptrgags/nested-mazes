use crate::grid::Grid;
use crate::direction::Direction;
use crate::grid_coords::GRID_SIZE;
use crate::dfs::DFSMaze;

const HALF_GRID_SIZE: usize = GRID_SIZE / 2;

pub struct Tile {
    level: usize,
    x: usize,
    y: usize,
    pub grid: Grid,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            level: 0,
            x: 0,
            y: 0,
            grid: Grid::new()
        }
    }

    pub fn make_root(maze_gen: &mut DFSMaze) -> Self {
        let mut root = Self::new();
        root.grid.mark_boundaries();
        root.grid.mark_exit(Direction::Down, 3);
        root.grid.mark_exit(Direction::Up, 5);
        maze_gen.maze_fill(&mut root.grid);
        maze_gen.clear();

        root
    }

    pub fn subdivide(&self, maze_gen: &mut DFSMaze) -> [Self; 4] {
        let bottom = self.grid.get_horizontal_seam(0, Direction::Down);
        let h_middle = self.grid.get_horizontal_seam(HALF_GRID_SIZE, Direction::Down);
        let top = self.grid.get_horizontal_seam(GRID_SIZE - 1, Direction::Up);
        
        let left = self.grid.get_vertical_seam(0, Direction::Left);
        let v_middle = self.grid.get_vertical_seam(HALF_GRID_SIZE, Direction::Left);
        let right = self.grid.get_vertical_seam(GRID_SIZE - 1, Direction::Right);
        
        // In Morton order:
        // Southwest
        let mut sw = Self::new();
        sw.level = self.level + 1;
        sw.x = self.x << 1;
        sw.y = self.y << 1;
        sw.grid.set_boundary(
            // Right
            &v_middle[0..4],
            // Up
            &h_middle[0..4],
            // Left
            &left[0..4],
            // Down
            &bottom[0..4]
        );
        self.grid.propagate_interior(&mut sw.grid, 0..4, 0..4);

        // Southeast
        let mut se = Self::new();
        se.level = self.level + 1;
        se.x = self.x << 1 | 1;
        se.y = self.y << 1;
        se.grid.set_boundary(
            // Right
            &right[0..4],
            // Up
            &h_middle[4..8],
            // Left
            &v_middle[0..4],
            // Down
            &bottom[4..8]
        );
        self.grid.propagate_interior(&mut se.grid, 4..8, 0..4);

        // Northwest
        let mut nw = Self::new();
        nw.level = self.level + 1;
        nw.x = self.x << 1;
        nw.y = self.y << 1 | 1;
        nw.grid.set_boundary(
            // Right
            &v_middle[4..8],
            // Up
            &top[0..4],
            // Left
            &left[4..8],
            // Down
            &h_middle[0..4]
        );
        self.grid.propagate_interior(&mut nw.grid, 0..4, 4..8);

        // Northeast
        let mut ne = Self::new();
        ne.level = self.level + 1;
        ne.x = self.x << 1;
        ne.y = self.y << 1 | 1;
        ne.grid.set_boundary(
            // Right
            &right[4..8],
            // Up
            &top[4..8],
            // Left
            &v_middle[4..8],
            // Down
            &h_middle[4..8],
        );
        self.grid.propagate_interior(&mut ne.grid, 4..8, 4..8);

        let mut result = [sw, se, nw, ne];

        for i in 0..4 {
            maze_gen.maze_fill(&mut result[i].grid);
            maze_gen.clear();
        }

        result
    }
}