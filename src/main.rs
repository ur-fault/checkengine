use colored::Colorize;
use std::fmt::Display;

fn main() {
    let mut board = Board::new(2);
    board.show_moves_for = Some((6, 2));
    // dbg!(&board);
    println!("{}", board);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color {
    White,
    Red,
}

impl Color {
    fn dir(&self) -> i8 {
        match self {
            Color::White => 1,
            Color::Red => -1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Piece {
    Pawn,
    Queen,
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
            (Pawn, Red) => write!(f, "{}", "P".red()),
            (Queen, White) => write!(f, "{}", "Q".white()),
            (Queen, Red) => write!(f, "{}", "Q".red()),
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

// #[derive(Clone, Debug, PartialEq, Eq)]
// struct Move {
//     from: (u8, u8),
//     to: (u8, u8),

//     // TODO: implement custom Vec-like thingy, that uses global stack to save them
//     // since we know that moves are always added to the end of the vector, we can
//     // just pop them off when we need to undo the move
//     kill: Vec<PosUncolorPiece>,
// }

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
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Board {
    board: [[Option<PlayersPiece>; 8]; 8],
    on_move: Color,
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
                        *board.get_mut(i, j) = Some(PlayersPiece::new(Color::Red, Piece::Pawn));
                    }
                }
            }
        }

        board
    }

    fn empty() -> Board {
        Board {
            board: [[None; 8]; 8],
            on_move: Color::White,
            moves: Vec::new(),
            show_moves_for: None,
        }
    }

    fn occupied_by(&self, row: u8, col: u8) -> Option<Color> {
        self.board[row as usize][col as usize].map(|p| p.color)
    }

    fn get_all_moves(&self, row: u8, col: u8) -> Option<Vec<Move>> {
        if self.board[row as usize][col as usize].is_none() {
            return None;
        }

        let (rowu, colu) = (row, col);
        let (rowi, coli) = (row as i8, col as i8);

        let PlayersPiece { piece, color } = self.get_ref(row, col).unwrap();

        if piece == Piece::Pawn {
            let mut moves = vec![];

            for col in [-1, 1] {
                let (row, col) = (rowi + color.dir(), coli + col);
                if self.is_free(row, col) {
                    moves.push(Move {
                        from: (rowu, colu),
                        to: (row as u8, col as u8),
                        piece,
                        kill: None,
                        color,
                    })
                }
            }

            return Some(moves);
        }

        None
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

        write!(f, "# ")?;
        write!(f, "{}\n", "1 2 3 4 5 6 7 8".underline().bold())?;

        let moves = self
            .show_moves_for
            .map(|(r, c)| self.get_all_moves(r, c).unwrap());

        for row in 0..8 {
            write!(f, "{}|", ((row + 'A' as u8) as char).to_string().bold())?;
            for col in 0..8 {
                if let Some(piece) = self.get_ref(row, col) {
                    let piece = piece.to_string();
                    if let Some(move_pos) = self.show_moves_for {
                        if move_pos == (row as u8, col as u8) {
                            write!(f, "{} ", piece.underline())?;
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

                write!(f, ". ")?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
