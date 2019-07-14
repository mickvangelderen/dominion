use rand::seq::SliceRandom;
use rand::Rng;

pub mod protocol;

#[derive(Debug, Clone)]
pub struct Player {
    pub play: Vec<CardKind>,
    pub hand: Vec<CardKind>,
    pub draw: Vec<CardKind>,
    pub discard: Vec<CardKind>,
}

impl Player {
    pub fn new(deck: Vec<CardKind>) -> Self {
        Player {
            draw: deck,
            hand: vec![],
            play: vec![],
            discard: vec![],
        }
    }

    pub fn draw(&mut self, rng: &mut impl Rng, count: usize) {
        let target_count = self.hand.len() + count;
        loop {
            while self.hand.len() < target_count {
                match self.draw.pop() {
                    Some(card) => {
                        self.hand.push(card);
                    }
                    None => {
                        break;
                    }
                }
            }

            if self.discard.len() == 0 {
                break;
            }

            std::mem::swap(&mut self.draw, &mut self.discard);
            self.draw.shuffle(rng);
        }
    }

    pub fn play(&mut self, index: HandIndex) -> Option<CardKind> {
        if index < self.hand.len() {
            Some(self.hand.remove(index))
        } else {
            None
        }
    }

    pub fn points(&self) -> usize {
        let deck_size = self.play.len() + self.hand.len() + self.draw.len() + self.discard.len();

        self.play
            .iter()
            .chain(
                self.hand
                    .iter()
                    .chain(self.draw.iter().chain(self.discard.iter())),
            )
            .map(|card| card.victory_points(deck_size))
            .sum()
    }
}

pub type PlayerIndex = usize;
pub type HandIndex = usize;

#[derive(Debug)]
pub struct Pile {
    pub card: CardKind,
    pub count: usize,
}

impl Pile {
    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

pub type PileIndex = usize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CardType {
    Action,
    Attack,
    Reaction,
    Treasure,
    Victory,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CardKind {
    Copper,
    Silver,
    Gold,
    Estate,
    Duchy,
    Province,
    Gardens,
    Cellar,
    Market,
    Merchant,
    Militia,
    Mine,
    Moat,
    Remodel,
    Smithy,
    Village,
    Workshop,
}

impl CardKind {
    pub fn victory_points(self, deck_size: usize) -> usize {
        match self {
            CardKind::Estate => 1,
            CardKind::Duchy => 3,
            CardKind::Province => 6,
            CardKind::Gardens => deck_size / 10,
            _ => 0,
        }
    }

    pub fn is_action(self) -> bool {
        if let CardKind::Cellar
        | CardKind::Market
        | CardKind::Merchant
        | CardKind::Militia
        | CardKind::Mine
        | CardKind::Moat
        | CardKind::Remodel
        | CardKind::Smithy
        | CardKind::Village
        | CardKind::Workshop = self
        {
            true
        } else {
            false
        }
    }

    pub fn initial_count(self, player_count: usize) -> usize {
        fn initial_victory_count(player_count: usize) -> usize {
            if player_count <= 2 {
                8
            } else {
                12
            }
        }

        match self {
            CardKind::Copper => 60 - 7 * player_count,
            CardKind::Silver => 40,
            CardKind::Gold => 30,
            CardKind::Estate => initial_victory_count(player_count),
            CardKind::Duchy => initial_victory_count(player_count),
            CardKind::Province => initial_victory_count(player_count),
            _ => 10,
        }
    }
}

pub enum CardEffect {
    GainCopper,
    GainAction,
    GainBuy,
}

#[derive(Debug)]
pub enum TurnPhase {
    Action,
    Buy,
}

#[derive(Debug)]
pub enum Event {
    EndPhase,
    PlayCard(HandIndex),
    BuyCard(PileIndex),
}

#[derive(Debug)]
pub struct Game<R> {
    pub rng: R,
    pub piles: Vec<Pile>,
    pub players: Vec<Player>,
    pub turn: usize,
    pub current_phase: TurnPhase,
    pub current_player: PlayerIndex,
    pub actions_remaining: usize,
    pub buys_remaining: usize,
    pub winner: Option<PlayerIndex>,
}

impl<R> Game<R>
where
    R: rand::Rng,
{
    pub fn new(rng: R, piles: Vec<Pile>, players: Vec<Player>) -> Self {
        Game {
            rng,
            piles,
            players,
            turn: 1,
            current_phase: TurnPhase::Action,
            current_player: 0,
            actions_remaining: 1,
            buys_remaining: 1,
            winner: None,
        }
    }

    pub fn process_event(&mut self, event: Event) {
        if self.winner.is_some() {
            panic!("Game is already over!");
        }

        match event {
            Event::EndPhase => {
                self.current_phase = match self.current_phase {
                    TurnPhase::Action => TurnPhase::Buy,
                    TurnPhase::Buy => {
                        // End the turn.
                        self.turn += 1;
                        self.current_player += 1;
                        if self.current_player >= self.players.len() {
                            // End the round.
                            self.current_player = 0;
                        }
                        self.winner = self.winner();
                        TurnPhase::Action
                    }
                };
            },
            Event::PlayCard(hand_index) => {
                let player = &mut self.players[self.current_player];
                match player.play(hand_index) {
                    Some(card) => {
                        if card.is_action() {
                            assert!(self.actions_remaining > 0, "No actions remaining!");
                            self.actions_remaining -= 1;
                        }

                        println!("Player {} plays {:?}", self.current_player, card);

                        match self.current_phase {
                            TurnPhase::Action => {
                                match card {
                                    CardKind::Village => {
                                        self.actions_remaining += 1;
                                        player.draw(&mut self.rng, 1);
                                    },
                                    _ => {
                                        unimplemented!();
                                    }
                                }
                            },
                            TurnPhase::Buy => {
                                unimplemented!();
                            },
                        }
                    },
                    None => {
                        panic!("Received an invalid hand index.");
                    }
                }
            }
            _ => {
                unimplemented!();
            }
        }
    }

    fn winner(&self) -> Option<PlayerIndex> {
        // Check game end.
        let game_has_ended = {
            let empty_pile_count = self.piles.iter().filter(|&pile| pile.is_empty()).count();
            let provinces_empty = self
                .piles
                .iter()
                .find(|pile| pile.card == CardKind::Province)
                .unwrap()
                .is_empty();
            empty_pile_count > 3 || provinces_empty
        };

        if game_has_ended {
            let points = self.players.iter().map(Player::points);

            Some(
                points
                    .enumerate()
                    .max_by_key(|&(_, p): &(PlayerIndex, usize)| p)
                    .map(|(i, _)| i)
                    .expect("Game has no players!"),
            )
        } else {
            None
        }
    }
}