use std::error::Error;
use std::fmt;

pub const MIN_BOARD_SIZE: u8 = 5;
pub const MAX_BOARD_SIZE: u8 = 31;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Player {
    Black = 1,
    White = 2,
}

impl Player {
    pub const fn other(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }

    pub const fn cell(self) -> Cell {
        match self {
            Self::Black => Cell::Black,
            Self::White => Cell::White,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Cell {
    #[default]
    Empty = 0,
    Black = 1,
    White = 2,
}

impl Cell {
    pub const fn player(self) -> Option<Player> {
        match self {
            Self::Empty => None,
            Self::Black => Some(Player::Black),
            Self::White => Some(Player::White),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Layout,
    Action,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Won(Player),
    Draw,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    Place { point: Point, color: Player },
    Pass,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GameConfig {
    pub size: u8,
}

impl GameConfig {
    pub fn new(size: u8) -> Result<Self, GameError> {
        if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) || size.is_multiple_of(2) {
            return Err(GameError::InvalidSize(size));
        }
        Ok(Self { size })
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { size: 15 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameError {
    InvalidSize(u8),
}

impl fmt::Display for GameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize(size) => write!(
                formatter,
                "棋盘尺寸必须是 {MIN_BOARD_SIZE} 到 {MAX_BOARD_SIZE} 之间的奇数，收到 {size}"
            ),
        }
    }
}

impl Error for GameError {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionError {
    GameFinished,
    PassNotAllowed,
    PointOutOfBounds(Point),
    PointOccupied(Point),
    WrongLayoutColor,
    OutsidePlayerRegion(Point),
}

impl fmt::Display for ActionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameFinished => formatter.write_str("对局已经结束"),
            Self::PassNotAllowed => formatter.write_str("行动阶段必须先在空格落子"),
            Self::PointOutOfBounds(point) => {
                write!(formatter, "坐标 ({}, {}) 超出棋盘", point.x + 1, point.y + 1)
            }
            Self::PointOccupied(point) => {
                write!(formatter, "坐标 ({}, {}) 已有棋子", point.x + 1, point.y + 1)
            }
            Self::WrongLayoutColor => formatter.write_str("布局阶段只能放置当前玩家自己的颜色"),
            Self::OutsidePlayerRegion(point) => {
                write!(formatter, "坐标 ({}, {}) 不在当前玩家的布局区域", point.x + 1, point.y + 1)
            }
        }
    }
}

impl Error for ActionError {}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct EvolutionStats {
    pub born_black: u16,
    pub born_white: u16,
    pub died_black: u16,
    pub died_white: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TurnResult {
    pub action: Action,
    pub phase_before: GamePhase,
    pub phase_after: GamePhase,
    pub generation: u32,
    pub evolution: Option<EvolutionStats>,
    pub status: GameStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    config: GameConfig,
    board: Vec<Cell>,
    phase: GamePhase,
    current_player: Player,
    consecutive_passes: u8,
    generation: u32,
    status: GameStatus,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(GameConfig::default())
    }
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let point_count = usize::from(config.size) * usize::from(config.size);
        Self {
            config,
            board: vec![Cell::Empty; point_count],
            phase: GamePhase::Layout,
            current_player: Player::Black,
            consecutive_passes: 0,
            generation: 0,
            status: GameStatus::Playing,
        }
    }

    pub const fn size(&self) -> u8 {
        self.config.size
    }

    pub fn board(&self) -> &[Cell] {
        &self.board
    }

    pub const fn phase(&self) -> GamePhase {
        self.phase
    }

    pub const fn current_player(&self) -> Player {
        self.current_player
    }

    pub const fn consecutive_passes(&self) -> u8 {
        self.consecutive_passes
    }

    pub const fn generation(&self) -> u32 {
        self.generation
    }

    pub const fn status(&self) -> GameStatus {
        self.status
    }

    pub fn cell(&self, point: Point) -> Option<Cell> {
        self.index(point).map(|index| self.board[index])
    }

    pub fn counts(&self) -> (u16, u16) {
        let mut black = 0;
        let mut white = 0;
        for cell in &self.board {
            match cell {
                Cell::Black => black += 1,
                Cell::White => white += 1,
                Cell::Empty => {}
            }
        }
        (black, white)
    }

    pub fn can_place(&self, point: Point, color: Player) -> bool {
        self.validate_place(point, color).is_ok()
    }

    pub fn apply(&mut self, action: Action) -> Result<TurnResult, ActionError> {
        if self.status != GameStatus::Playing {
            return Err(ActionError::GameFinished);
        }

        let phase_before = self.phase;
        match action {
            Action::Place { point, color } => {
                self.validate_place(point, color)?;
                let index = self.index(point).expect("validated point must be on the board");
                self.board[index] = color.cell();
                self.consecutive_passes = 0;
            }
            Action::Pass => {
                if self.phase != GamePhase::Layout {
                    return Err(ActionError::PassNotAllowed);
                }
                self.consecutive_passes = self.consecutive_passes.saturating_add(1);
            }
        }

        let evolution = match phase_before {
            GamePhase::Layout => {
                self.finish_layout_turn();
                None
            }
            GamePhase::Action => {
                let stats = self.evolve();
                self.generation += 1;
                self.update_status();
                if self.status == GameStatus::Playing {
                    self.current_player = self.current_player.other();
                }
                Some(stats)
            }
        };

        Ok(TurnResult {
            action,
            phase_before,
            phase_after: self.phase,
            generation: self.generation,
            evolution,
            status: self.status,
        })
    }

    fn validate_place(&self, point: Point, color: Player) -> Result<(), ActionError> {
        if self.status != GameStatus::Playing {
            return Err(ActionError::GameFinished);
        }
        let Some(index) = self.index(point) else {
            return Err(ActionError::PointOutOfBounds(point));
        };
        if self.board[index] != Cell::Empty {
            return Err(ActionError::PointOccupied(point));
        }
        if self.phase == GamePhase::Layout {
            if color != self.current_player {
                return Err(ActionError::WrongLayoutColor);
            }
            let middle = self.config.size / 2;
            let inside_region = match self.current_player {
                Player::Black => point.y <= middle,
                Player::White => point.y >= middle,
            };
            if !inside_region {
                return Err(ActionError::OutsidePlayerRegion(point));
            }
        }
        Ok(())
    }

    fn finish_layout_turn(&mut self) {
        if self.consecutive_passes >= 2 {
            self.phase = GamePhase::Action;
            self.current_player = Player::Black;
            self.consecutive_passes = 0;
            self.update_status();
        } else {
            self.current_player = self.current_player.other();
        }
    }

    fn evolve(&mut self) -> EvolutionStats {
        let previous = self.board.clone();
        let mut next = vec![Cell::Empty; previous.len()];
        let mut stats = EvolutionStats::default();

        for (index, cell) in previous.iter().copied().enumerate() {
            let point = self.point(index);
            let (black_neighbors, white_neighbors) = self.neighbor_counts(&previous, point);
            let next_cell = match cell {
                Cell::Empty => match black_neighbors - white_neighbors {
                    3 => Cell::Black,
                    -3 => Cell::White,
                    _ => Cell::Empty,
                },
                Cell::Black if matches!(black_neighbors - white_neighbors, 2 | 3) => Cell::Black,
                Cell::White if matches!(white_neighbors - black_neighbors, 2 | 3) => Cell::White,
                Cell::Black | Cell::White => Cell::Empty,
            };
            next[index] = next_cell;
            Self::record_change(&mut stats, cell, next_cell);
        }

        self.board = next;
        stats
    }

    fn neighbor_counts(&self, board: &[Cell], point: Point) -> (i8, i8) {
        let mut black = 0;
        let mut white = 0;
        for delta_y in -1i16..=1 {
            for delta_x in -1i16..=1 {
                if delta_x == 0 && delta_y == 0 {
                    continue;
                }
                let x = i16::from(point.x) + delta_x;
                let y = i16::from(point.y) + delta_y;
                if x < 0
                    || y < 0
                    || x >= i16::from(self.config.size)
                    || y >= i16::from(self.config.size)
                {
                    continue;
                }
                let index = y as usize * usize::from(self.config.size) + x as usize;
                match board[index] {
                    Cell::Black => black += 1,
                    Cell::White => white += 1,
                    Cell::Empty => {}
                }
            }
        }
        (black, white)
    }

    fn record_change(stats: &mut EvolutionStats, before: Cell, after: Cell) {
        if before == after {
            return;
        }
        match before {
            Cell::Black => stats.died_black += 1,
            Cell::White => stats.died_white += 1,
            Cell::Empty => {}
        }
        match after {
            Cell::Black => stats.born_black += 1,
            Cell::White => stats.born_white += 1,
            Cell::Empty => {}
        }
    }

    fn update_status(&mut self) {
        let (black, white) = self.counts();
        self.status = match (black, white) {
            (0, 0) => GameStatus::Draw,
            (0, _) => GameStatus::Won(Player::White),
            (_, 0) => GameStatus::Won(Player::Black),
            _ => GameStatus::Playing,
        };
    }

    fn index(&self, point: Point) -> Option<usize> {
        (point.x < self.config.size && point.y < self.config.size)
            .then(|| usize::from(point.y) * usize::from(self.config.size) + usize::from(point.x))
    }

    fn point(&self, index: usize) -> Point {
        let size = usize::from(self.config.size);
        Point::new((index % size) as u8, (index / size) as u8)
    }
}
