use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Take {
    pub x: u8,
    pub y: u8,
}
impl Take {
    pub fn x(&self) -> usize {
        self.x as usize
    }
    pub fn y(&self) -> usize {
        self.y as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GameSetting {
    board_height: u8,
    board_width: u8,
    goal: u8,
    piece_limit: u8,
}
impl GameSetting {
    pub fn try_new(
        board_height: usize,
        board_width: usize,
        goal: usize,
        piece_limit: usize,
    ) -> Result<GameSetting, String> {
        if board_height == 0 {
            return Err("Board height should be greater than 0".to_string());
        }
        if board_height > 11 {
            return Err("Board height should be less than or equal to 11".to_string());
        }
        if board_width == 0 {
            return Err("Board width should be greater than 0".to_string());
        }
        if board_width > 11 {
            return Err("Board width should be less than or equal to 11".to_string());
        }
        if goal == 0 {
            return Err("Goal should be greater than 0".to_string());
        }
        if goal > board_height {
            return Err("Goal should be less than or equal to board height".to_string());
        }
        if goal > board_width {
            return Err("Goal should be less than or equal to board width".to_string());
        }
        if piece_limit == 0 {
            return Err("Piece limit should be greater than 0".to_string());
        }
        if piece_limit > 127 {
            return Err("Piece limit should be less than or equal to 127".to_string());
        }
        Ok(GameSetting {
            board_height: board_height as u8,
            board_width: board_width as u8,
            goal: goal as u8,
            piece_limit: piece_limit as u8,
        })
    }
    pub fn try_new_normal_limited(
        board_size: usize,
        piece_limit: usize,
    ) -> Result<GameSetting, String> {
        GameSetting::try_new(board_size, board_size, board_size, piece_limit)
    }
    pub fn try_new_normal(board_size: usize) -> Result<GameSetting, String> {
        GameSetting::try_new(board_size, board_size, board_size, board_size * board_size)
    }
    pub fn board_height(&self) -> usize {
        self.board_height as usize
    }
    pub fn board_width(&self) -> usize {
        self.board_width as usize
    }
    pub fn goal(&self) -> usize {
        self.goal as usize
    }
    pub fn piece_limit(&self) -> usize {
        self.piece_limit as usize
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BoardState {
    takes: Vec<Take>,
}
impl BoardState {
    pub fn rotate(&self, setting: &GameSetting) -> BoardState {
        if setting.board_height() != setting.board_width() {
            panic!("Board should be square");
        }
        let mut new_takes = vec![];
        for take in self.takes.iter() {
            new_takes.push(Take {
                x: setting.board_height() as u8 - take.y - 1,
                y: take.x,
            });
        }
        BoardState { takes: new_takes }
    }

    pub fn mirror_x(&self, setting: &GameSetting) -> BoardState {
        let mut new_takes = vec![];
        for take in self.takes.iter() {
            new_takes.push(Take {
                x: setting.board_width() as u8 - take.x - 1,
                y: take.y,
            });
        }
        BoardState { takes: new_takes }
    }

    pub fn mirror_y(&self, setting: &GameSetting) -> BoardState {
        let mut new_takes = vec![];
        for take in self.takes.iter() {
            new_takes.push(Take {
                x: take.x,
                y: setting.board_height() as u8 - take.y - 1,
            });
        }
        BoardState { takes: new_takes }
    }

    pub fn normalized(&self, setting: &GameSetting) -> BoardState {
        let isotopes = if setting.board_height() == setting.board_width() {
            let mut v = vec![self.clone()];
            for _ in 0..3 {
                v.push(v.last().unwrap().rotate(setting));
            }
            v.push(v.last().unwrap().mirror_x(setting));
            for _ in 0..3 {
                v.push(v.last().unwrap().rotate(setting));
            }
            v
        } else {
            [self.clone(), self.mirror_x(setting)]
                .into_iter()
                .flat_map(|state| [state.clone(), state.mirror_y(setting)].into_iter())
                .collect::<Vec<_>>()
        };
        isotopes.into_iter().min().unwrap()
    }

    pub fn is_normalized(&self, setting: &GameSetting) -> bool {
        self == &self.normalized(setting)
    }
}
impl Eq for BoardState {}
impl PartialOrd for BoardState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for BoardState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.takes.len() != other.takes.len() {
            return self.takes.len().cmp(&other.takes.len());
        }
        for (a, b) in self.takes.iter().zip(other.takes.iter()) {
            if a.x != b.x {
                return a.x.cmp(&b.x);
            }
            if a.y != b.y {
                return a.y.cmp(&b.y);
            }
        }
        std::cmp::Ordering::Equal
    }
}
use std::hash::{Hash, Hasher};
impl Hash for BoardState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.takes.len().hash(state);
        for take in self.takes.iter() {
            take.x.hash(state);
            take.y.hash(state);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellView {
    None,
    First(usize),
    Second(usize),
}
impl CellView {
    pub fn is_first(&self) -> bool {
        matches!(self, CellView::First(_))
    }
    pub fn is_second(&self) -> bool {
        matches!(self, CellView::Second(_))
    }
    pub fn is_none(&self) -> bool {
        matches!(self, CellView::None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameResult {
    FirstWin,
    SecondWin,
    Continue,
}
impl GameResult {
    pub fn is_win(&self) -> bool {
        matches!(self, GameResult::FirstWin | GameResult::SecondWin)
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    setting: GameSetting,
    state: BoardState,
    steps_taken: usize,
}
impl Game {
    pub fn add_take(&mut self, take: Take) {
        assert!(!self.is_finished(), "Game already finished");
        self.state.takes = self
            .state
            .takes
            .iter()
            .skip(
                if self.state.takes.len() >= (self.setting.piece_limit * 2).into() {
                    1
                } else {
                    0
                },
            )
            .chain(std::iter::once(&take))
            .cloned()
            .collect();
        self.steps_taken += 1;
    }

    pub fn is_next_first(&self) -> bool {
        (self.steps_taken % 2) == 0
    }

    pub fn result(&self) -> GameResult {
        if self.is_win(true) {
            return GameResult::FirstWin;
        }
        if self.is_win(false) {
            return GameResult::SecondWin;
        }
        GameResult::Continue
    }

    fn is_win(&self, first: bool) -> bool {
        let f = |c: &CellView| {
            matches!(
                (first, c),
                (true, CellView::First(_)) | (false, CellView::Second(_))
            )
        };
        let cells = self
            .to_cells()
            .into_iter()
            .map(|row| row.into_iter().map(|cell| f(&cell)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        for y_base in 0..=self.setting.board_height() - self.setting.goal() {
            for x_base in 0..=self.setting.board_width() - self.setting.goal() {
                if (0..self.setting.goal()).all(|i| cells[y_base + i][x_base + i]) {
                    return true;
                }
                if (0..self.setting.goal())
                    .all(|i| cells[y_base + i][x_base + self.setting.goal() - i - 1])
                {
                    return true;
                }
            }
        }
        for y_base in 0..=self.setting.board_height() - self.setting.goal() {
            for x in 0..self.setting.board_width() {
                if (0..self.setting.goal()).all(|i| cells[y_base + i][x]) {
                    return true;
                }
            }
        }
        for x_base in 0..=self.setting.board_width() - self.setting.goal() {
            for row in cells.iter() {
                if (0..self.setting.goal()).all(|i| row[x_base + i]) {
                    return true;
                }
            }
        }
        false
    }

    pub fn new(setting: GameSetting) -> Game {
        Game {
            setting,
            state: BoardState::default(),
            steps_taken: 0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.result().is_win() || self.valid_take_count() == 0
    }

    pub fn valid_take_count(&self) -> usize {
        let mut count = 0;
        for y in 0..self.setting.board_height() {
            for x in 0..self.setting.board_width() {
                if self
                    .validate_take(Take {
                        x: x as u8,
                        y: y as u8,
                    })
                    .is_ok()
                {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn validate_take(&self, take: Take) -> Result<(), String> {
        if take.x() >= self.setting.board_width() || take.y() >= self.setting.board_height() {
            return Err("Out of board".to_string());
        }
        let untakable = self.state.takes.iter().skip(
            if self.state.takes.len() >= (self.setting.piece_limit * 2).into() {
                1
            } else {
                0
            },
        );
        for t in untakable {
            if t.x == take.x && t.y == take.y {
                return Err("Already taken".to_string());
            }
        }
        Ok(())
    }

    pub fn to_cells(&self) -> Vec<Vec<CellView>> {
        let mut board =
            vec![vec![CellView::None; self.setting.board_width()]; self.setting.board_height()];
        for (i, take) in self.state.takes.iter().enumerate() {
            let rest = ((self.setting.piece_limit() * 2 - self.state.takes.len()) + i) / 2;
            let cell = match ((self.state.takes.len() - i + 1) % 2) ^ (self.steps_taken % 2) {
                0 => CellView::Second(rest),
                _ => CellView::First(rest),
            };
            board[take.y as usize][take.x as usize] = cell;
        }
        board
    }

    pub fn state(&self) -> &BoardState {
        &self.state
    }
    pub fn steps_taken(&self) -> usize {
        self.steps_taken
    }
    pub fn replace_state(&mut self, state: BoardState, steps_taken: usize) {
        self.state = state;
        self.steps_taken = steps_taken;
        self.verify_full().expect("Invalid state");
    }

    pub fn verify_full(&self) -> Result<(), String> {
        // takes len should be up to piece_count * 2
        if self.state.takes.len() > (self.setting.piece_limit * 2).into() {
            return Err(format!(
                "Takes len should be up to piece_count * 2 ({}), but {}",
                self.setting.piece_limit * 2,
                self.state.takes.len()
            ));
        }
        // should not take the same cell
        {
            let mut set = std::collections::HashSet::new();
            for take in self.state.takes.iter() {
                if !set.insert(take) {
                    return Err(format!("Should not take the same cell {:?}", take));
                }
            }
        }
        // should not take out of board
        for take in self.state.takes.iter() {
            if take.x() >= self.setting.board_width() || take.y() >= self.setting.board_height() {
                return Err(format!("Should not take out of board {:?}", take));
            }
        }
        // both should not win at the same time
        if self.is_win(true) && self.is_win(false) {
            return Err("Both should not win at the same time".to_string());
        }
        Ok(())
    }

    pub fn normalize(&self) -> Game {
        let mut new_game = self.clone();
        new_game.state = self.state.normalized(&self.setting);
        new_game
    }

    pub fn is_normalized(&self) -> bool {
        self.state.is_normalized(&self.setting)
    }

    pub fn is_last_take(&self, take: Take) -> bool {
        self.state.takes.last() == Some(&take)
    }
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "TicTacToeGame {}x{} (height x width) with {} pieces",
            self.setting.board_height(),
            self.setting.board_width(),
            self.setting.piece_limit
        )?;
        let cells = self.to_cells();
        for row in cells.iter() {
            for _ in 0..self.setting.board_width() {
                write!(f, "------")?;
            }
            writeln!(f, "-")?;
            for cell in row.iter() {
                write!(
                    f,
                    "| {} ",
                    match cell {
                        CellView::None => " ".to_string(),
                        CellView::First(n) => format!("o{: <2}", n),
                        CellView::Second(n) => format!("x{: <2}", n),
                    }
                )?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "----------------")?;
        writeln!(
            f,
            "0 is about to disappear, o is the first player, x is the second player"
        )?;
        writeln!(f, "{} steps taken", self.steps_taken)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum GameAnalysis {
    Winning(usize),
    Losing(usize),
    #[default]
    Neutral,
}
impl GameAnalysis {
    pub fn is_winning(&self) -> bool {
        matches!(self, GameAnalysis::Winning(_))
    }
    pub fn is_losing(&self) -> bool {
        matches!(self, GameAnalysis::Losing(_))
    }
    pub fn is_neutral(&self) -> bool {
        matches!(self, GameAnalysis::Neutral)
    }
    pub fn max() -> GameAnalysis {
        GameAnalysis::Losing(0)
    }
    pub fn min() -> GameAnalysis {
        GameAnalysis::Winning(0)
    }
}
impl fmt::Display for GameAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameAnalysis::Winning(n) => write!(f, "Winning in {} steps", n),
            GameAnalysis::Losing(n) => write!(f, "Losing in {} steps", n),
            GameAnalysis::Neutral => write!(f, "Neutral"),
        }
    }
}
impl Eq for GameAnalysis {}
impl PartialOrd for GameAnalysis {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for GameAnalysis {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (GameAnalysis::Winning(a), GameAnalysis::Winning(b)) => a.cmp(b),
            (GameAnalysis::Losing(a), GameAnalysis::Losing(b)) => b.cmp(a),
            (GameAnalysis::Winning(_), _) => std::cmp::Ordering::Less,
            (GameAnalysis::Losing(_), _) => std::cmp::Ordering::Greater,
            (_, GameAnalysis::Winning(_)) => std::cmp::Ordering::Greater,
            (_, GameAnalysis::Losing(_)) => std::cmp::Ordering::Less,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
pub fn analyze(setting: GameSetting, start: BoardState, max_cnt: usize) -> AnalysisDictionary {
    let mut game = Game::new(setting);
    game.replace_state(start, 0);
    game = game.normalize();

    let mut visited = HashSet::new();
    let mut valid_take_count = HashMap::<BoardState, usize>::new();
    let mut edges = HashMap::<BoardState, Vec<BoardState>>::new();
    let mut edges_rev = HashMap::<BoardState, Vec<BoardState>>::new();
    //let mut stack = vec![];
    let mut search = VecDeque::new();
    visited.insert(game.state.clone());
    //stack.push(game.state.clone());
    search.push_back(game.state.clone());

    let mut queue = VecDeque::new();

    let mut cnt = 0;
    while let Some(state) = search.pop_front() {
        cnt += 1;
        if cnt % 10000 == 0 {
            println!("cnt: {}", cnt);
        }
        if cnt > max_cnt {
            break;
        }
        game.replace_state(state.clone(), 1);
        if game.result() == GameResult::FirstWin {
            queue.push_back(state.clone());
            valid_take_count.insert(state.clone(), 0);
            continue;
        }
        valid_take_count.insert(state.clone(), game.valid_take_count());
        for y in 0..game.setting.board_height() {
            for x in 0..game.setting.board_width() {
                let take = Take {
                    x: x as u8,
                    y: y as u8,
                };
                if game.validate_take(take).is_ok() {
                    let mut new_game = game.clone();
                    new_game.add_take(take);
                    let new_game = new_game.normalize();
                    let new_state = new_game.state;
                    if visited.insert(new_state.clone()) {
                        search.push_back(new_state.clone());
                    }
                    edges
                        .entry(state.clone())
                        .or_default()
                        .push(new_state.clone());
                    edges_rev
                        .entry(new_state.clone())
                        .or_default()
                        .push(state.clone());
                }
            }
        }
    }

    let mut done = HashMap::<BoardState, GameAnalysis>::new();

    let mut cnt = 0;
    while let Some(state) = queue.pop_front() {
        cnt += 1;
        if cnt % 10000 == 0 {
            println!("cnt: {}", cnt);
        }
        let valid_take_count = valid_take_count.get(&state).copied().unwrap();
        let mut all_done = valid_take_count == edges.get(&state).map(|v| v.len()).unwrap_or(0);
        let mut winning = false;
        let mut min_to_win = usize::MAX;
        let mut max_to_lose = 0;
        if let Some(next_states) = edges.get(&state) {
            for next_state in next_states {
                match done.get(next_state) {
                    Some(GameAnalysis::Winning(to_win)) => {
                        max_to_lose = max_to_lose.max(to_win + 1);
                    }
                    Some(GameAnalysis::Losing(to_lose)) => {
                        winning = true;
                        min_to_win = min_to_win.min(to_lose + 1);
                    }
                    _ => {
                        all_done = false;
                    }
                }
            }
        }
        let updated = if winning {
            let to_update = match done.get(&state) {
                Some(GameAnalysis::Winning(to_win)) => *to_win > min_to_win,
                _ => true,
            };
            if to_update {
                done.insert(state.clone(), GameAnalysis::Winning(min_to_win));
            }
            to_update
        } else if all_done {
            let to_update = match done.get(&state) {
                Some(GameAnalysis::Losing(to_lose)) => *to_lose < max_to_lose,
                _ => true,
            };
            if to_update {
                done.insert(state.clone(), GameAnalysis::Losing(max_to_lose));
            }
            to_update
        } else {
            false
        };
        if updated {
            if let Some(prev_states) = edges_rev.get(&state) {
                for prev_state in prev_states {
                    queue.push_back(prev_state.clone());
                }
            }
        }
    }

    AnalysisDictionary {
        setting,
        analysis: done,
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisDictionary {
    setting: GameSetting,
    analysis: HashMap<BoardState, GameAnalysis>,
}

impl AnalysisDictionary {
    pub fn setting(&self) -> &GameSetting {
        &self.setting
    }
    pub fn analysis(&self) -> &HashMap<BoardState, GameAnalysis> {
        &self.analysis
    }
    pub fn merge(&self, other: AnalysisDictionary) -> AnalysisDictionary {
        let mut analysis = self.analysis.clone();
        for (k, v) in other.analysis {
            // TODO: take better
            analysis.entry(k).or_insert(v);
        }
        AnalysisDictionary {
            setting: self.setting,
            analysis,
        }
    }
}
