use super::types::{Bitfield, PieceIndex};

pub struct PiecePicker {
    // pieces we have
    own_pieces: Bitfield,
    //
    pieces: Vec<Piece>,
    // count of pieces we do not have yet, but may have picked
    missing_count: usize,
    // number of pieces we can still pick
    free_count: usize,
}

pub struct Piece {
    // will be important later for optimizations
    // such as starting with the rarest pieces
    pub frequency: usize,
    pub is_pending: bool,
}

impl PiecePicker {
    pub fn new(own_pieces: Bitfield) -> Self {
        let pieces = vec![own_pieces.len(), Piece::default];
        let missing_count = own_pieces.count_zeros();
        Self {
            own_pieces,
            pieces,
            missing_count,
            free_count: missing_count,
        }
    }

    pub fn pick_piece(&mut self) -> Option<PieceIndex> {
        for index in 0..self.pieces.len() {
            let piece = &mut self.pieces[index];
            if !self.own_pieces[index] && piece.frequency > 0 && !piece.is_pending {
                piece.is_pending = true;
                self.free_count -= 1;
                return Some(index);
            }
        }
        None
    }

    pub fn register_peer_piece(&mut self, index: PieceIndex) -> bool {
        let is_interested = self.own_pieces.get(index).expect("invalid piece index");
        self.pieces[index].frequency += 1;
        *is_interested
    }

    pub fn received_piece(&mut self, index: PieceIndex) {
        let mut have_piece = self.own_pieces.get_mut(index).expect("invalid piece index");
        *have_piece = true;
        self.missing_count -= 1;
        let piece = &mut self.pieces[index];
        if !piece.is_pending {
            self.free_count -= 1;
            piece.is_pending = false;
        }
    }
}
