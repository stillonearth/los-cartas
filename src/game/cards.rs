use bevy::{app::App, prelude::*};
use bevy_la_mesa::events::{
    AlignCardsInHand, AlignChipsOnTable, PlaceCardOffTable, PlaceCardOnTable,
};
use bevy_la_mesa::{Card, CardMetadata, CardOnTable, Chip, ChipArea, Deck};

use std::fmt::Debug;
use std::marker::Send;

use crate::GameCamera;

#[derive(Resource)]
pub struct PhaseTimer(pub Timer);

#[derive(Clone, Copy, Debug, Default)]
pub enum CardType {
    #[default]
    Attack,
    BigDeal,
    Cocaine,
    Cannabis,
    Drought,
    Export,
    LocalMarket,
    Train,
    Truck,
}

#[derive(Component)]
pub struct ActiveEventCard {
    pub player: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ChipType {
    #[default]
    Cocaine,
    Cannabis,
}

#[derive(Default, Clone, Debug)]
pub struct Kard {
    pub card_type: CardType,
    pub price: i32,
    pub filename: String,
}

impl CardMetadata for Kard {
    type Output = Kard;

    fn filename(&self) -> String {
        self.filename.clone()
    }
}

pub fn load_playing_deck(num_players: usize) -> Vec<Kard> {
    let attack = Kard {
        card_type: CardType::Attack,
        price: 300,
        filename: "tarjetas/attack.png".to_string(),
    };

    let cocaine = Kard {
        card_type: CardType::Cocaine,
        price: 1000,
        filename: "tarjetas/cocaine.png".to_string(),
    };

    let export = Kard {
        card_type: CardType::Export,
        price: 0,
        filename: "tarjetas/export.png".to_string(),
    };

    let local_market = Kard {
        card_type: CardType::LocalMarket,
        price: 0,
        filename: "tarjetas/local-market.png".to_string(),
    };

    let marijuana = Kard {
        card_type: CardType::Cannabis,
        price: 500,
        filename: "tarjetas/marijuana.png".to_string(),
    };

    let train = Kard {
        card_type: CardType::Train,
        price: 600,
        filename: "tarjetas/train.png".to_string(),
    };

    let truck = Kard {
        card_type: CardType::Truck,
        price: 300,
        filename: "tarjetas/truck.png".to_string(),
    };

    let mut deck: Vec<Kard> = vec![];
    for _ in 0..num_players {
        deck.push(cocaine.clone());
        deck.push(cocaine.clone());

        deck.push(marijuana.clone());
        deck.push(marijuana.clone());

        deck.push(truck.clone());
        deck.push(truck.clone());

        deck.push(train.clone());

        deck.push(local_market.clone());
        deck.push(local_market.clone());

        deck.push(export.clone());

        deck.push(attack.clone());
    }

    deck
}

pub fn load_event_deck(num_players: usize) -> Vec<Kard> {
    let big_deal = Kard {
        card_type: CardType::BigDeal,
        price: 0,
        filename: "tarjetas/big-deal.png".to_string(),
    };

    let drought = Kard {
        card_type: CardType::Drought,
        price: 0,
        filename: "tarjetas/drought.png".to_string(),
    };

    let mut deck: Vec<Kard> = vec![];
    for _ in 0..num_players {
        deck.push(drought.clone());
        deck.push(big_deal.clone());
    }

    deck
}

#[derive(Default, Debug, PartialEq)]
pub enum TurnPhase {
    #[default]
    Prepare,
    PlaceCardsOnTable,
    DrawEventCard,
    ApplyEventCard,
    ApplyProductionCards,
    ApplyTransportationCards,
    ApplySalesCards,
    ApplyActionCards,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectType {
    Drought,
    Attack,
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub effect_type: EffectType,
    pub player: usize,
    pub turn_number: usize,
    pub duration: usize,
}

#[derive(Resource)]
pub struct GameState {
    pub turn_number: usize,
    pub effects: Vec<Effect>,
    pub phase: TurnPhase,
    pub player: usize,
    pub bank: Vec<i32>,
    num_players: usize,
}

impl GameState {
    pub fn advance(&mut self) {
        self.phase = match self.phase {
            TurnPhase::Prepare => TurnPhase::PlaceCardsOnTable,
            TurnPhase::PlaceCardsOnTable => TurnPhase::DrawEventCard,
            TurnPhase::DrawEventCard => TurnPhase::ApplyEventCard,
            TurnPhase::ApplyEventCard => TurnPhase::ApplyProductionCards,
            TurnPhase::ApplyProductionCards => TurnPhase::ApplyTransportationCards,
            TurnPhase::ApplyTransportationCards => TurnPhase::ApplySalesCards,
            TurnPhase::ApplySalesCards => TurnPhase::ApplyActionCards,
            TurnPhase::ApplyActionCards => TurnPhase::End,
            TurnPhase::End => {
                if self.player == self.num_players {
                    self.turn_number += 1;
                    self.player = 1;
                } else {
                    self.player += 1;
                }
                TurnPhase::Prepare
            }
        };
        self.remove_expired_effects();
    }

    pub fn new(num_players: usize) -> Self {
        Self {
            turn_number: 1,
            phase: TurnPhase::Prepare,
            player: 1,
            bank: vec![5000; num_players],
            num_players,
            effects: vec![],
        }
    }

    pub fn reset(&mut self) {
        self.turn_number = 1;
        self.phase = TurnPhase::Prepare;
        self.player = 1;
        self.bank = vec![0; self.num_players];
    }

    pub fn change_balance(&mut self, player: usize, amount: i32) {
        self.bank[player - 1] += amount;
    }

    pub fn draw_bank(&mut self, player: usize, amount: i32) {
        self.bank[player - 1] -= amount;
    }

    pub fn get_balance(&self, player: usize) -> i32 {
        self.bank[player - 1]
    }

    pub fn add_effect(&mut self, effect_type: EffectType, duration: usize, player: usize) {
        self.effects
            .retain(|effect| !(effect.effect_type == effect_type && effect.player == player));

        self.effects.push(Effect {
            effect_type: effect_type,
            player: player,
            turn_number: self.turn_number,
            duration,
        });
    }

    pub fn get_effects(&self, player: usize) -> Vec<Effect> {
        self.effects
            .iter()
            .filter(|effect| effect.player == player)
            .cloned()
            .collect()
    }

    pub fn remove_expired_effects(&mut self) {
        self.effects
            .retain(|effect| effect.turn_number + effect.duration > self.turn_number);
    }
}

// Events

#[derive(Event)]
pub struct AdvancePhase;

#[derive(Event)]
pub struct SwitchPlayer {
    pub player: usize,
}

#[derive(Event)]
pub struct DropChip {
    pub chip_type: ChipType,
    pub area: usize,
    pub player: usize,
}

#[derive(Debug, Event)]
pub struct MoveChip {
    pub entity: Entity,
    pub area: usize,
    pub player: usize,
}

#[derive(Debug, Event)]
pub struct DiscardChip {
    pub entity: Entity,
}

#[derive(Event)]
pub struct GameOver {
    pub player_won: usize,
}

// Plugin

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(GameState::new(2))
        .add_event::<AdvancePhase>()
        .add_event::<DropChip>()
        .add_event::<MoveChip>()
        .add_event::<DiscardChip>()
        .add_event::<SwitchPlayer>()
        .add_event::<GameOver>()
        .insert_resource(PhaseTimer(Timer::from_seconds(0.3, TimerMode::Once)))
        .add_systems(
            Update,
            (
                apply_card_effects,
                handle_next_phase,
                handle_drop_chip,
                handle_move_chip,
                handle_switch_player,
                check_game_over_conditions,
            ),
        );
}

pub fn apply_card_effects(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    cards_on_table: Query<(Entity, &Card<Kard>, &CardOnTable)>,
    cards_in_deck: Query<(Entity, &Transform, &Card<Kard>, &Deck)>,
    chips_on_table: Query<(Entity, &Transform, &Chip<ChipType>, &ChipArea)>,
    event_cards_on_table: Query<(Entity, &Card<Kard>, &ActiveEventCard)>,
    mut ew_place_card_off_table: EventWriter<PlaceCardOffTable>,
    mut ew_place_card_on_table: EventWriter<PlaceCardOnTable>,
    mut ew_drop_chip: EventWriter<DropChip>,
    mut ew_move_chip: EventWriter<MoveChip>,
    mut ew_discard_chip: EventWriter<DiscardChip>,
    mut ew_advance_phase: EventWriter<AdvancePhase>,
) {
    let player = state.player;

    // Apply Cards in Play Area
    for (entity, card, _) in cards_on_table
        .iter()
        .filter(|(_, _, card_on_table)| card_on_table.player == player)
    {
        match state.phase {
            TurnPhase::ApplyProductionCards => match card.data.card_type {
                CardType::Cocaine | CardType::Cannabis => {
                    let production_power = 5;
                    let discount = state.get_effects(player).iter().fold(0, |acc, effect| {
                        match effect.effect_type {
                            EffectType::Attack => acc + 1,
                            EffectType::Drought => acc + 1,
                        }
                    });

                    for _ in 0..(production_power - discount).max(0) {
                        let event = DropChip {
                            chip_type: match card.data.card_type {
                                CardType::Cocaine => ChipType::Cocaine,
                                CardType::Cannabis => ChipType::Cannabis,
                                _ => ChipType::Cocaine,
                            },
                            area: 1,
                            player,
                        };
                        ew_drop_chip.send(event);
                    }

                    ew_place_card_off_table.send(PlaceCardOffTable {
                        card_entity: entity,
                        deck_marker: 1,
                    });
                }
                _ => {}
            },
            TurnPhase::ApplyTransportationCards => match card.data.card_type {
                CardType::Truck | CardType::Train => {
                    let mut cannabis_chips_on_table = chips_on_table
                        .iter()
                        .filter(|(_, _, chip, area)| {
                            chip.data == ChipType::Cannabis
                                && area.marker == 1
                                && area.player == player
                                && chip.turn_activation_1 < state.turn_number
                        })
                        .collect::<Vec<_>>();
                    cannabis_chips_on_table.sort_by(|(_, t1, _, _), (_, t2, _, _)| {
                        t1.translation.z.partial_cmp(&t2.translation.z).unwrap()
                    });

                    let mut cocaine_chips_on_table = chips_on_table
                        .iter()
                        .filter(|(_, _, chip, area)| {
                            chip.data == ChipType::Cocaine
                                && area.marker == 1
                                && area.player == player
                                && chip.turn_activation_1 < state.turn_number
                        })
                        .collect::<Vec<_>>();

                    cocaine_chips_on_table.sort_by(|(_, t1, _, _), (_, t2, _, _)| {
                        t1.translation.z.partial_cmp(&t2.translation.z).unwrap()
                    });

                    let mut entities_to_move: Vec<Entity> = vec![];
                    let mut chip_value = match card.data.card_type {
                        CardType::Truck => 20,
                        CardType::Train => 50,
                        _ => 0,
                    };

                    let common_chips =
                        std::cmp::min(cannabis_chips_on_table.len(), cocaine_chips_on_table.len());

                    for i in 0..common_chips {
                        let cannabis_chip = cannabis_chips_on_table[i].0;
                        let cocaine_chip = cocaine_chips_on_table[i].0;

                        entities_to_move.push(cannabis_chip);
                        entities_to_move.push(cocaine_chip);
                    }

                    if common_chips < cannabis_chips_on_table.len() {
                        for chip in cannabis_chips_on_table.iter().skip(common_chips) {
                            entities_to_move.push(chip.0);
                        }
                    }

                    if common_chips < cocaine_chips_on_table.len() {
                        for chip in cocaine_chips_on_table.iter().skip(common_chips) {
                            entities_to_move.push(chip.0);
                        }
                    }

                    for entity in entities_to_move {
                        if chip_value <= 0 {
                            break;
                        }

                        let event = MoveChip {
                            entity,
                            area: 2,
                            player,
                        };
                        ew_move_chip.send(event);
                        chip_value -= 2;
                    }

                    ew_place_card_off_table.send(PlaceCardOffTable {
                        card_entity: entity,
                        deck_marker: 1,
                    });
                }
                _ => {}
            },
            TurnPhase::ApplySalesCards => match card.data.card_type {
                CardType::Export | CardType::LocalMarket => {
                    let mut cannabis_chips_on_table = chips_on_table
                        .iter()
                        .filter(|(_, _, chip, area)| {
                            chip.data == ChipType::Cannabis
                                && area.marker == 2
                                && area.player == player
                                && chip.turn_activation_2 < state.turn_number
                                && chip.turn_activation_2 != 0
                        })
                        .collect::<Vec<_>>();
                    cannabis_chips_on_table.sort_by(|(_, t1, _, _), (_, t2, _, _)| {
                        t2.translation.z.partial_cmp(&t1.translation.z).unwrap()
                    });

                    let mut cocaine_chips_on_table = chips_on_table
                        .iter()
                        .filter(|(_, _, chip, area)| {
                            chip.data == ChipType::Cocaine
                                && area.marker == 2
                                && area.player == player
                                && chip.turn_activation_2 < state.turn_number
                                && chip.turn_activation_2 != 0
                        })
                        .collect::<Vec<_>>();

                    cocaine_chips_on_table.sort_by(|(_, t1, _, _), (_, t2, _, _)| {
                        t2.translation.z.partial_cmp(&t1.translation.z).unwrap()
                    });

                    let mut entities_to_discard: Vec<Entity> = vec![];
                    let mut chip_value = match card.data.card_type {
                        CardType::Export => 50,
                        CardType::LocalMarket => 20,
                        _ => 0,
                    };

                    let common_chips =
                        std::cmp::min(cannabis_chips_on_table.len(), cocaine_chips_on_table.len());

                    for i in 0..common_chips {
                        let cannabis_chip = cannabis_chips_on_table[i].0;
                        let cocaine_chip = cocaine_chips_on_table[i].0;

                        entities_to_discard.push(cannabis_chip);
                        entities_to_discard.push(cocaine_chip);
                    }

                    if common_chips < cannabis_chips_on_table.len() {
                        for i in common_chips..cannabis_chips_on_table.len() {
                            entities_to_discard.push(cannabis_chips_on_table[i].0);
                        }
                    }

                    if common_chips < cocaine_chips_on_table.len() {
                        for i in common_chips..cocaine_chips_on_table.len() {
                            entities_to_discard.push(cocaine_chips_on_table[i].0);
                        }
                    }

                    for entity in entities_to_discard {
                        if chip_value <= 0 {
                            break;
                        }

                        let event = DiscardChip { entity };

                        ew_discard_chip.send(event);
                        chip_value -= 2;

                        let cost: i32 = match card.data.card_type {
                            CardType::Cocaine => 800,
                            CardType::Cannabis => 400,
                            _ => 0,
                        };

                        state.change_balance(player, cost);
                    }

                    ew_place_card_off_table.send(PlaceCardOffTable {
                        card_entity: entity,
                        deck_marker: 1,
                    });
                }
                _ => {}
            },
            TurnPhase::ApplyActionCards => match card.data.card_type {
                CardType::Attack => {
                    state.add_effect(
                        EffectType::Attack,
                        2,
                        match player {
                            1 => 2,
                            2 => 1,
                            _ => 1,
                        },
                    );

                    ew_place_card_off_table.send(PlaceCardOffTable {
                        card_entity: entity,
                        deck_marker: 1,
                    });
                }
                _ => {}
            },
            _ => {}
        }
    }

    // Apply Effect Cards
    match state.phase {
        TurnPhase::DrawEventCard => {
            let n_event_cards_from = event_cards_on_table
                .iter()
                .filter(|(_, _, active_event_card)| active_event_card.player == player)
                .count();

            if n_event_cards_from == 0 {
                let mut event_cards = cards_in_deck
                    .iter()
                    .filter(|(_, _, _, deck)| deck.marker == 2)
                    .collect::<Vec<_>>();
                event_cards.sort_by(|(_, t1, _, _), (_, t2, _, _)| {
                    t2.translation.z.partial_cmp(&t1.translation.z).unwrap()
                });
                let entity = event_cards[0].0;
                commands
                    .entity(entity)
                    .insert(ActiveEventCard { player })
                    .remove::<Deck>();
                ew_place_card_on_table.send(PlaceCardOnTable {
                    card_entity: event_cards[0].0,
                    marker: 6,
                    player,
                });
            }
        }
        TurnPhase::End => {
            if state.player == state.num_players {
                for (entity, _, _) in event_cards_on_table.iter() {
                    commands.entity(entity).remove::<ActiveEventCard>();
                    ew_place_card_off_table.send(PlaceCardOffTable {
                        card_entity: entity,
                        deck_marker: 2,
                    });
                }
            }
        }
        TurnPhase::ApplyEventCard => {
            let event_cards = event_cards_on_table
                .iter()
                .filter(|(_, _, active_event_card)| active_event_card.player == player)
                .collect::<Vec<_>>();

            for (_, card, _) in event_cards {
                let card_type = card.data.card_type;
                match card_type {
                    CardType::Drought => {
                        state.add_effect(EffectType::Drought, 3, player);
                    }

                    _ => {}
                }
            }

            state.advance();
        }
        _ => {}
    }

    match state.phase {
        TurnPhase::ApplyProductionCards
        | TurnPhase::ApplyTransportationCards
        | TurnPhase::ApplySalesCards
        | TurnPhase::DrawEventCard
        | TurnPhase::ApplyEventCard
        | TurnPhase::ApplyActionCards
        | TurnPhase::End => {
            ew_advance_phase.send(AdvancePhase);
        }
        _ => {}
    }
}

pub fn handle_next_phase(
    mut er_next_phase: EventReader<AdvancePhase>,
    mut ew_align_cards_in_hand: EventWriter<AlignCardsInHand>,
    mut ew_align_chips_on_table: EventWriter<AlignChipsOnTable<ChipType>>,
    mut game_state: ResMut<GameState>,
    mut ew_switch_player: EventWriter<SwitchPlayer>,
    mut phase_timer: ResMut<PhaseTimer>,
    time: Res<Time>,
) {
    if !phase_timer.0.finished() {
        phase_timer.0.tick(time.delta());
    }

    for _ in er_next_phase.read() {
        if !phase_timer.0.finished() {
            continue;
        }

        match game_state.phase {
            TurnPhase::PlaceCardsOnTable => {
                ew_align_cards_in_hand.send(AlignCardsInHand {
                    player: game_state.player,
                });
            }

            TurnPhase::End => {
                ew_align_chips_on_table.send(AlignChipsOnTable::<ChipType> {
                    chip_area: ChipArea {
                        marker: 1,
                        player: 1,
                    },
                    chip_type: ChipType::Cocaine,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable::<ChipType> {
                    chip_area: ChipArea {
                        marker: 1,
                        player: 1,
                    },
                    chip_type: ChipType::Cannabis,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable {
                    chip_area: ChipArea {
                        marker: 2,
                        player: 1,
                    },
                    chip_type: ChipType::Cocaine,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable {
                    chip_area: ChipArea {
                        marker: 2,
                        player: 1,
                    },
                    chip_type: ChipType::Cannabis,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable::<ChipType> {
                    chip_area: ChipArea {
                        marker: 1,
                        player: 2,
                    },
                    chip_type: ChipType::Cocaine,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable::<ChipType> {
                    chip_area: ChipArea {
                        marker: 1,
                        player: 2,
                    },
                    chip_type: ChipType::Cannabis,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable {
                    chip_area: ChipArea {
                        marker: 2,
                        player: 2,
                    },
                    chip_type: ChipType::Cocaine,
                });
                ew_align_chips_on_table.send(AlignChipsOnTable {
                    chip_area: ChipArea {
                        marker: 2,
                        player: 2,
                    },
                    chip_type: ChipType::Cannabis,
                });
            }
            _ => {}
        }

        let previous_player = game_state.player;
        game_state.advance();
        let next_player = game_state.player;

        if previous_player != next_player {
            ew_switch_player.send(SwitchPlayer {
                player: next_player,
            });
        }
        phase_timer.0.reset();
    }
}

pub fn handle_drop_chip(mut er_drop_chip: EventReader<DropChip>) {
    for _drop_chip in er_drop_chip.read() {
        // println!("Dropping chip: {:?}", drop_chip.chip_type);
    }
}

pub fn handle_move_chip(
    mut er_drop_chip: EventReader<MoveChip>,
    mut query: Query<(Entity, &mut Chip<ChipType>)>,
    state: Res<GameState>,
) {
    for move_chip in er_drop_chip.read() {
        let (_, mut chip) = query.get_mut(move_chip.entity).unwrap();
        chip.turn_activation_2 = state.turn_number;
    }
}

pub fn handle_switch_player(
    mut er_switch_player: EventReader<SwitchPlayer>,
    mut game_state: ResMut<GameState>,
    mut query: Query<(&mut Transform, &GameCamera)>,
) {
    for event in er_switch_player.read() {
        game_state.player = event.player;

        for (mut transform, _) in query.iter_mut() {
            if game_state.player == 1 {
                *transform = Transform::from_xyz(0.0, 12.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y);
            } else {
                *transform = Transform::from_xyz(-3.0, 12.0, -15.0)
                    .looking_at(Vec3::ZERO + Vec3::new(-3.0, 0.0, 0.0), Vec3::Y);
            }
        }
    }
}

pub fn check_game_over_conditions(
    mut er_advance_phase: EventReader<AdvancePhase>,
    game_state: Res<GameState>,
    mut ew_game_over_event: EventWriter<GameOver>,
) {
    for _ in er_advance_phase.read() {
        let funds_per_player: Vec<i32> = (1..=game_state.num_players)
            .map(|player| game_state.get_balance(player))
            .collect();

        let lowest_funds = *funds_per_player.iter().min().unwrap();

        let highest_funds = *funds_per_player.iter().max().unwrap();
        let index_highest_funds = funds_per_player
            .iter()
            .position(|&r| r == highest_funds)
            .unwrap();
        let total_players = game_state.num_players;

        if total_players == 2 && lowest_funds <= 0 {
            ew_game_over_event.send(GameOver {
                player_won: index_highest_funds + 1,
            });
            return;
        }

        if highest_funds >= 50000 {
            ew_game_over_event.send(GameOver {
                player_won: index_highest_funds + 1,
            });
            return;
        }

        if game_state.turn_number >= 20 {
            ew_game_over_event.send(GameOver {
                player_won: index_highest_funds + 1,
            });
            return;
        }
    }
}
