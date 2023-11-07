use colored::Colorize;
use std::fmt::Display;

fn main() {
    let mut board = Board::new(2);

    println!("{}", board);

    while let None = board.winner() {
        let move_ = board.find_all_moves()[0];
        println!("Player {} played {}", board.current_player(), move_);
        board.push(move_);

        println!("{}", board);
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

    fn filter_killer_moves(moves: &[Move]) -> Vec<Move> {
        moves
            .iter()
            .filter(|m| m.kill.is_some())
            .map(|m| *m)
            .collect()
    }

    fn filter_piece_moves(piece: Piece, moves: &[Move]) -> Vec<Move> {
        moves
            .iter()
            .filter(|m| m.piece == piece)
            .map(|m| *m)
            .collect()
    }
}

fn format_pos(pos: (u8, u8)) -> String {
    format!("{}{}", ((pos.0 + 'A' as u8) as char).to_string(), pos.1 + 1)
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{} -> {}", format_pos(self.from), format_pos(self.to));
        if let Some(PosUncolorPiece { piece, row, col }) = self.kill {
            write!(f, "{} # {} {}", pos, format_pos((row, col)), piece)
        } else {
            write!(f, "{}", pos)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Board {
    board: [[Option<PlayersPiece>; 8]; 8],
    moves: Vec<Move>,
    show_moves_for: Option<(u8, u8)>,
}

impl Board {
    fn new(lines: u8) -> Board {
        let mut board = Board::empty();

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

    fn empty() -> Board {
        Board {
            board: [[None; 8]; 8],
            moves: Vec::new(),
            show_moves_for: None,
        }
    }

    fn occupied_by(&self, row: u8, col: u8) -> Option<Color> {
        self.board[row as usize][col as usize].map(|p| p.color)
    }

    fn last_player(&self) -> Option<Color> {
        self.moves.last().map(|m| m.color)
    }

    fn current_player(&self) -> Color {
        let Some(move_) = self.moves.last() else {
            return Color::White;
        };

        let Move { to, color, .. } = move_;

        if move_.continues() && !self.find_moves(to.0, to.1, Some(true)).unwrap().is_empty() {
            *color
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

        if piece == Piece::Pawn {
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

            return Some(moves);
        }

        None
    }

    fn find_all_moves(&self) -> Vec<Move> {
        let moves: Vec<_> = self
            .all_current_pieces()
            .iter()
            .flat_map(|p| self.find_moves(p.0, p.1, None).unwrap())
            .collect();

        if Move::filter_killer_moves(&moves).is_empty() {
            return moves;
        }

        let moves = Move::filter_killer_moves(&moves);

        if Move::filter_piece_moves(Piece::Queen, &moves).is_empty() {
            return moves;
        }

        Move::filter_piece_moves(Piece::Queen, &moves)
    }

    fn is_valid_move(&self, move_: Move) -> bool {
        let Some(piece) = self.get_ref(move_.from.0, move_.from.1) else {
            return false;
        };

        if piece.color != self.current_player() {
            return false;
        }

        self.find_all_moves().contains(&move_)
    }

    fn all_players_pieces(&self, player: Color) -> Vec<(u8, u8, Piece)> {
        (0..8)
            .map(move |r| {
                (0..8).map(move |c| match *self.get_ref(r, c) {
                    Some(PlayersPiece { color, piece }) if color == player => Some((r, c, piece)),
                    _ => None,
                })
            })
            .flatten()
            .flatten()
            .collect()
    }

    fn all_current_pieces(&self) -> Vec<(u8, u8, Piece)> {
        self.all_players_pieces(self.current_player())
    }

    fn winner(&self) -> Option<Color> {
        if self.all_players_pieces(Color::White).is_empty() {
            return Some(Color::Black);
        }

        if self.all_players_pieces(Color::Black).is_empty() {
            return Some(Color::White);
        }

        self.find_all_moves()
            .is_empty()
            .then(|| self.current_player().other())
    }

    fn push(&mut self, move_: Move) -> Option<Color> {
        if !self.is_valid_move(move_) {
            panic!("Invalid move");
        }

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

        self.winner()
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

        write!(f, "Player {} is on the move\n", self.current_player(),)?;

        for piece in self.all_players_pieces(self.current_player()) {
            for move_ in self.find_moves(piece.0, piece.1, None).unwrap() {
                write!(f, "{} {}\n", "-".color(move_.color.colored()), move_)?;
            }
        }

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
