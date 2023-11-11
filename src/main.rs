use colored::Colorize;
use ordered_float::OrderedFloat;
use std::fmt::Display;

fn main() {
    let mut board = Board::new(
        2,
        RateConfig {
            pieces: PieceRates {
                pawn: 1.0,
                queen: 3.0,
            },
            position: PositionRates {
                // pawn: 0.5,
                // queen: 1.5,
                pawn: 0.0,
                queen: 0.0,
            },
            kills: KillRates {
                pawn: 10.0,
                queen: 30.0,
            },
            win: 1000.0,
            max_depth: 5,
        },
    );

    println!("{}", board);

    while board.winner().is_none() && board.turn < 100 {
        let move_ = board.find_best_move();
        println!("Player {} played {}", board.current_player(), move_);
        board.push(move_);

        println!("{}", board);
    }

    if let Some(winner) = board.winner() {
        println!("Player {} won!", winner);
    } else {
        println!("{}", "Draw".underline().bold());
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

impl Color {
    fn dir(&self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    fn colored(&self) -> colored::Color {
        match self {
            Color::White => colored::Color::White,
            Color::Black => colored::Color::BrightRed,
        }
    }

    fn other(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "{}", "White".color(self.colored())),
            Color::Black => write!(f, "{}", "Black".color(self.colored())),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Piece {
    Pawn,
    Queen,
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Piece::Pawn => write!(f, "{}", "Pawn"),
            Piece::Queen => write!(f, "{}", "Queen"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct PlayersPiece {
    color: Color,
    piece: Piece,
}

impl PlayersPiece {
    fn new(color: Color, piece: Piece) -> PlayersPiece {
        PlayersPiece { color, piece }
    }
}

impl Display for PlayersPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use {Color::*, Piece::*};

        #[cfg(feature = "simple_pieces")]
        match (self.piece, self.color) {
            (Pawn, White) => write!(f, "{}", "P".white()),
            (Pawn, Black) => write!(f, "{}", "P".red()),
            (Queen, White) => write!(f, "{}", "Q".white()),
            (Queen, Black) => write!(f, "{}", "Q".red()),
        }

        #[cfg(all(feature = "reversed_pieces", not(feature = "simple_pieces")))]
        match (self.piece, self.color) {
            (Pawn, White) => write!(f, "{}", "🨣".white()),
            (Pawn, Red) => write!(f, "{}", "♙".red()),
            (Queen, White) => write!(f, "{}", "🨟".white()),
            (Queen, Red) => write!(f, "{}", "♕".red()),
        }

        #[cfg(all(not(feature = "reversed_pieces"), not(feature = "simple_pieces")))]
        match (self.piece, self.color) {
            (Pawn, White) => write!(f, "{}", "♙".white()),
            (Pawn, Red) => write!(f, "{}", "♙".red()),
            (Queen, White) => write!(f, "{}", "♕".white()),
            (Queen, Red) => write!(f, "{}", "♕".red()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct PosUncolorPiece {
    piece: Piece,
    row: u8,
    col: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Move {
    // NOT TRUE, since we need to know if the piece upgrades to queen
    // we don't need to save the piece, since we can just get it from the board
    // using the `from` position
    from: (u8, u8),
    to: (u8, u8),

    piece: Piece,

    // but we need to save what this move killed, so that we can undo it
    kill: Option<PosUncolorPiece>,

    // we save color so that Queen can move multiple times in one turn
    // so one move is **NOT** synonymous with one turn
    color: Color,
}

impl Move {
    fn continues(&self) -> bool {
        self.kill.is_some() && !self.is_upgrade()
    }

    fn is_upgrade(&self) -> bool {
        self.piece != Piece::Queen && (self.to.0 == if self.color == Color::White { 7 } else { 0 })
    }

    fn future_piece(&self) -> Piece {
        if self.is_upgrade() {
            Piece::Queen
        } else {
            self.piece
        }
    }

    fn filter_killer_moves(mut moves: Vec<Move>) -> Vec<Move> {
        moves.retain(|m| m.kill.is_some());
        moves
    }

    fn contains_killer_move(moves: &[Move]) -> bool {
        moves.iter().any(|m| m.kill.is_some())
    }

    fn filter_piece_moves(piece: Piece, mut moves: Vec<Move>) -> Vec<Move> {
        moves.retain(|m| m.piece == piece);
        moves
    }

    fn contains_piece_move(piece: Piece, moves: &[Move]) -> bool {
        moves.iter().any(|m| m.piece == piece)
    }
}

fn format_pos(pos: (u8, u8)) -> String {
    format!("{}{}", ((pos.0 + 'A' as u8) as char).to_string(), pos.1 + 1)
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{} -> {}", format_pos(self.from), format_pos(self.to));
        write!(f, "{}", pos)?;

        if let Some(PosUncolorPiece { piece, row, col }) = self.kill {
            write!(f, " # {} {}", format_pos((row, col)), piece)?;
        }

        if self.is_upgrade() {
            write!(f, " @@")?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct PieceRates {
    pawn: f32,
    queen: f32,
}

impl Eq for PieceRates {}

impl PieceRates {
    fn rate(&self, piece: Piece) -> f32 {
        match piece {
            Piece::Pawn => self.pawn,
            Piece::Queen => self.queen,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct PositionRates {
    /// The closer to the opponent's side, the higher the rate
    /// First line is `pawn`, second is 2×`pawn` and so on
    pawn: f32,

    /// The closer to the center of the board, the higher the rate
    /// Edge is `queen`, inner edge is 2×`queen` and so on
    /// Other way of doing things would be euclidean distance × `queen`
    queen: f32,
}

impl Eq for PositionRates {}

impl PositionRates {
    fn rate(&self, row: u8, col: u8, color: Color, piece: Piece) -> f32 {
        match piece {
            Piece::Pawn => {
                ((match color {
                    Color::White => row + 1,
                    Color::Black => 8 - row,
                }) as f32)
                    * self.pawn
            }
            Piece::Queen => {
                let centered = |v: i8| 5 - ((v * 2 - 7).abs() + 1) / 2;
                let row = centered(row as i8);
                let col = centered(col as i8);

                (row as f32 + col as f32) * self.queen
            }
        }
    }
}

#[cfg(test)]
mod pos_rate_tests {
    use super::*;

    #[test]
    fn test_rate() {
        let rates = PositionRates {
            pawn: 1.0,
            queen: 1.0,
        };

        assert_eq!(rates.rate(0, 0, Color::White, Piece::Pawn), 1.0);
        assert_eq!(rates.rate(0, 0, Color::Black, Piece::Pawn), 8.0);
        assert_eq!(rates.rate(7, 0, Color::White, Piece::Pawn), 8.0);
        assert_eq!(rates.rate(7, 0, Color::Black, Piece::Pawn), 1.0);

        assert_eq!(rates.rate(0, 0, Color::White, Piece::Queen), 1.0 + 1.0);
        assert_eq!(rates.rate(3, 3, Color::White, Piece::Queen), 4.0 + 4.0);
        assert_eq!(rates.rate(0, 3, Color::White, Piece::Queen), 1.0 + 4.0);
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct KillRates {
    pawn: f32,
    queen: f32,
}

impl Eq for KillRates {}

impl KillRates {
    fn rate(&self, piece: Piece) -> f32 {
        match piece {
            Piece::Pawn => self.pawn,
            Piece::Queen => self.queen,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct RateConfig {
    pieces: PieceRates,
    position: PositionRates,
    kills: KillRates,
    win: f32,
    max_depth: usize,
}

impl Eq for RateConfig {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Board {
    board: [[Option<PlayersPiece>; 8]; 8],
    moves: Vec<Move>,
    turn: usize,
    show_moves_for: Option<(u8, u8)>,
    rating: RateConfig,
}

impl Board {
    fn new(lines: u8, rates: RateConfig) -> Board {
        let mut board = Board::empty(rates);

        for i in 0..8 {
            for j in 0..8 {
                // starting at A1 (0, 0) <=> 2|0 + 0
                if (i + j) % 2 == 0 {
                    if i < lines {
                        *board.get_mut(i, j) = Some(PlayersPiece::new(Color::White, Piece::Pawn));
                    } else if i >= 8 - lines {
                        *board.get_mut(i, j) = Some(PlayersPiece::new(Color::Black, Piece::Pawn));
                    }
                }
            }
        }

        board
    }

    fn empty(rates: RateConfig) -> Board {
        Board {
            board: [[None; 8]; 8],
            moves: Vec::new(),
            turn: 0,
            show_moves_for: None,
            rating: rates,
        }
    }

    fn occupied_by(&self, row: u8, col: u8) -> Option<Color> {
        self.board[row as usize][col as usize].map(|p| p.color)
    }

    fn last_player(&self) -> Option<Color> {
        self.last_move().map(|m| m.color)
    }

    fn last_move(&self) -> Option<Move> {
        self.moves.last().map(|m| *m)
    }

    fn current_player(&self) -> Color {
        let Some(move_) = self.last_move() else {
            return Color::White;
        };

        let Move { to, color, .. } = move_;

        if move_.continues() && !self.find_moves(to.0, to.1, Some(true)).unwrap().is_empty() {
            color
        } else {
            color.other()
        }
    }

    fn get_ref(&self, row: u8, col: u8) -> &Option<PlayersPiece> {
        &self.board[row as usize][col as usize]
    }

    fn get_mut(&mut self, row: u8, col: u8) -> &mut Option<PlayersPiece> {
        &mut self.board[row as usize][col as usize]
    }

    fn in_bounds(&self, row: i8, col: i8) -> bool {
        row >= 0 && row < 8 && col >= 0 && col < 8
    }

    fn is_free(&self, row: i8, col: i8) -> bool {
        self.in_bounds(row, col) && self.get_ref(row as u8, col as u8).is_none()
    }

    fn find_moves(&self, row: u8, col: u8, kills: Option<bool>) -> Option<Vec<Move>> {
        if self.board[row as usize][col as usize].is_none() {
            return None;
        }

        let (rowu, colu) = (row, col);
        let (rowi, coli) = (row as i8, col as i8);

        let PlayersPiece { piece, color } = self.get_ref(row, col).unwrap();

        match piece {
            Piece::Pawn => {
                let mut moves = vec![];

                for col_offset in [-1, 1] {
                    let (row, col) = (rowi + color.dir(), coli + col_offset);
                    if self.is_free(row, col) {
                        if !kills.unwrap_or(false) {
                            moves.push(Move {
                                from: (rowu, colu),
                                to: (row as u8, col as u8),
                                piece,
                                kill: None,
                                color,
                            })
                        }
                    } else if self.in_bounds(row, col)
                        && self.get_ref(row as u8, col as u8).unwrap().color != color
                        && self.is_free(row + color.dir(), col + col_offset)
                    {
                        if kills.unwrap_or(true) {
                            let killed = self.get_ref(row as u8, col as u8).unwrap().piece;
                            moves.push(Move {
                                from: (rowu, colu),
                                to: ((row + color.dir()) as u8, (col + col_offset) as u8),
                                piece,
                                kill: Some(PosUncolorPiece {
                                    piece: killed,
                                    row: row as u8,
                                    col: col as u8,
                                }),
                                color,
                            })
                        }
                    }
                }

                Some(moves)
            }
            Piece::Queen => {
                let mut moves = vec![];

                for row_offset in [-1, 1] {
                    for col_offset in [-1, 1] {
                        let (mut row, mut col) = (rowi + row_offset, coli + col_offset);
                        while self.is_free(row, col) {
                            if !kills.unwrap_or(false) {
                                moves.push(Move {
                                    from: (rowu, colu),
                                    to: (row as u8, col as u8),
                                    piece,
                                    kill: None,
                                    color,
                                });
                            }
                            row += row_offset;
                            col += col_offset;
                        }

                        if self.in_bounds(row, col)
                            && self.get_ref(row as u8, col as u8).unwrap().color != color
                            && self.is_free(row + row_offset, col + col_offset)
                        {
                            let killed = self.get_ref(row as u8, col as u8).unwrap().piece;

                            if !kills.unwrap_or(true) {
                                continue;
                            }

                            let (mut free_row, mut free_col) = (row + row_offset, col + col_offset);
                            while self.is_free(free_row, free_col) {
                                moves.push(Move {
                                    from: (rowu, colu),
                                    to: (free_row as u8, free_col as u8),
                                    piece,
                                    kill: Some(PosUncolorPiece {
                                        piece: killed,
                                        row: row as u8,
                                        col: col as u8,
                                    }),
                                    color,
                                });

                                free_row += row_offset;
                                free_col += col_offset;
                            }
                        }
                    }
                }

                Some(moves)
            }
        }
    }

    fn find_all_current_moves(&self) -> Vec<Move> {
        let moves: Vec<_> = self
            .all_current_pieces()
            .flat_map(|p| self.find_moves(p.0, p.1, None).unwrap())
            .collect();

        if !Move::contains_killer_move(&moves) {
            return moves;
        }

        let moves = Move::filter_killer_moves(moves);

        if !Move::contains_piece_move(Piece::Queen, &moves) {
            return moves;
        }

        Move::filter_piece_moves(Piece::Queen, moves)
    }

    fn is_valid_move(&self, move_: Move) -> bool {
        let Some(piece) = self.get_ref(move_.from.0, move_.from.1) else {
            return false;
        };

        if piece.color != self.current_player() {
            return false;
        }

        self.find_all_current_moves().contains(&move_)
    }

    fn all_players_pieces(&self, player: Color) -> impl Iterator<Item = (u8, u8, Piece)> + '_ {
        (0..8)
            .map(move |r| {
                (0..8).map(move |c| match *self.get_ref(r, c) {
                    Some(PlayersPiece { color, piece }) if color == player => Some((r, c, piece)),
                    _ => None,
                })
            })
            .flatten()
            .flatten()
    }

    fn all_current_pieces(&self) -> impl Iterator<Item = (u8, u8, Piece)> + '_ {
        self.all_players_pieces(self.current_player())
    }

    fn winner(&self) -> Option<Color> {
        if self.all_players_pieces(Color::White).count() == 0 {
            return Some(Color::Black);
        }

        if self.all_players_pieces(Color::Black).count() == 0 {
            return Some(Color::White);
        }

        self.find_all_current_moves()
            .is_empty()
            .then(|| self.current_player().other())
    }

    fn push(&mut self, move_: Move) -> Option<Color> {
        if !self.is_valid_move(move_) {
            panic!("Invalid move");
        }

        self.push_unsafe(move_);

        self.winner()
    }

    fn push_unsafe(&mut self, move_: Move) {
        let Move {
            from,
            to,
            piece,
            kill,
            color,
        } = move_;
        *self.get_mut(from.0, from.1) = None;

        if let Some(kill) = kill {
            *self.get_mut(kill.row, kill.col) = None;
        }

        let piece = match move_.is_upgrade() {
            true => Piece::Queen,
            false => piece,
        };
        *self.get_mut(to.0, to.1) = Some(PlayersPiece::new(color, piece));

        self.moves.push(move_);

        if self.current_player() != color {
            self.turn += 1;
        }
    }

    fn pop(&mut self) -> Move {
        if self.current_player() != self.moves.last().expect("No moves to pop").color {
            self.turn -= 1;
        }

        let move_ = self.moves.pop().expect("No moves to pop");

        let Move {
            from,
            to,
            piece,
            kill,
            color,
        } = move_;

        *self.get_mut(from.0, from.1) = Some(PlayersPiece::new(color, piece));

        if let Some(PosUncolorPiece { row, col, piece }) = kill {
            *self.get_mut(row, col) = Some(PlayersPiece::new(color.other(), piece));
        }

        *self.get_mut(to.0, to.1) = None;

        move_
    }

    fn with_move<T>(&mut self, move_: Move, f: impl FnOnce(&mut Self) -> T) -> T {
        let moves_before = self.moves.len();
        self.push(move_);
        let ret = f(self);
        self.pop();
        assert_eq!(moves_before, self.moves.len());
        ret
    }

    fn with_move_unsafe<T>(&mut self, move_: Move, f: impl FnOnce(&mut Self) -> T) -> T {
        let moves_before = self.moves.len();
        self.push_unsafe(move_);
        let ret = f(self);
        self.pop();
        assert_eq!(moves_before, self.moves.len());
        ret
    }

    fn rate(&mut self, player: Color) -> f32 {
        fn rate_inner(board: &mut Board, player: Color, depth: usize) -> f32 {
            let RateConfig { win, max_depth, .. } = board.rating;

            if let Some(winner) = board.winner() {
                return if winner == player { win } else { -win };
            }

            if depth < max_depth {
                // calculating max rate of player
                let moves = board.find_all_current_moves();
                moves
                    .into_iter()
                    .map(|move_| {
                        let continuation = board.current_player()
                            == board.last_player().expect("`max_depth` must be > 0");
                        board.with_move_unsafe(move_, |board| {
                            -rate_inner(board, player, if continuation { depth } else { depth + 1 })
                        }) * if continuation { 1.0 } else { -1.0 }
                    })
                    .max_by(|a, b| a.partial_cmp(b).expect("Nan"))
                    .expect("No moves")
            } else {
                board.rate_current_board()
            }
        }

        rate_inner(self, player, 0)
    }

    fn rate_current_board(&self) -> f32 {
        fn rate_player(board: &Board, player: Color) -> f32 {
            let RateConfig {
                pieces,
                position,
                kills,
                ..
            } = board.rating;

            let pos = board
                .all_players_pieces(player)
                .into_iter()
                .map(|(r, c, p)| position.rate(r, c, player, p))
                .sum::<f32>();
            let piece = board
                .all_players_pieces(player)
                .into_iter()
                .map(|(_, _, p)| pieces.rate(p))
                .sum::<f32>();
            let kill = board
                .all_players_pieces(player)
                .into_iter()
                .map(|(r, c, _)| {
                    board
                        .find_moves(r, c, Some(true))
                        .map(|moves| {
                            moves
                                .into_iter()
                                .map(|m| kills.rate(m.kill.unwrap().piece))
                                .sum()
                        })
                        .unwrap_or(0.0)
                })
                .sum::<f32>();

            pos + piece + kill
        }
        let current_player = self.current_player();
        rate_player(self, current_player) - rate_player(self, current_player.other())
    }

    fn find_best_move(&mut self) -> Move {
        let moves = self.find_all_current_moves();
        moves
            .into_iter()
            .max_by_key(|m| {
                OrderedFloat(self.with_move_unsafe(*m, |b| -b.rate(b.current_player())))
            })
            .unwrap()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //   1 2 3 4 5 6 7 8
        // A P . P . P . P .
        // B . P . P . P . P
        // C . . . . . . . .
        // D . . . . . . . .
        // E . . . . . . . .
        // F . . . . . . . .
        // G P . P . P . P .
        // H . P . P . P . P

        write!(
            f,
            "#{} - Player {} is on the move\n",
            self.turn + 1,
            self.current_player(),
        )?;

        write!(
            f,
            "Rating for {} - {}\n",
            self.current_player(),
            self.rate_current_board()
        )?;

        // for move_ in self.find_all_current_moves() {
        //     write!(f, "{} {}\n", "-".color(move_.color.colored()), move_)?;
        // }

        let moves = self
            .show_moves_for
            .map(|(r, c)| self.find_moves(r, c, None))
            .flatten();

        write!(f, "# ")?;
        write!(f, "{}\n", "1 2 3 4 5 6 7 8".underline().bold())?;

        for row in 0..8 {
            write!(f, "{}|", ((row + 'A' as u8) as char).to_string().bold())?;
            for col in 0..8 {
                if let Some(piece) = self.get_ref(row, col) {
                    let piece = piece.to_string();
                    if let Some(move_pos) = self.show_moves_for {
                        if move_pos == (row as u8, col as u8) {
                            write!(f, "{} ", piece.underline().italic())?;
                            continue;
                        }
                    }

                    write!(f, "{} ", piece.to_string())?;
                    continue;
                } else if let Some(moves) = &moves {
                    if let Some(move_) = moves.iter().find(|m| m.to == (row as u8, col as u8)) {
                        let moving = self.get_ref(move_.from.0, move_.from.1).unwrap();
                        let piece = PlayersPiece::new(moving.color, move_.future_piece());
                        write!(f, "{} ", piece.to_string().dimmed())?;
                        continue;
                    }
                }

                match self.show_moves_for {
                    Some((r, c)) if r == row as u8 && c == col as u8 => {
                        write!(f, "{} ", ".".underline())?
                    }
                    _ => write!(f, ". ")?,
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
