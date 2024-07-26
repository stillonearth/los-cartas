use bevy::prelude::*;
use bevy_la_mesa::{
    events::{DeckShuffle, DrawHand},
    Chip, ChipArea, DeckArea,
};

use super::{
    cards::{AdvancePhase, ChipType, DropChip, GameState, MoveChip, SwitchPlayer, TurnPhase},
    spawn::ui::CardGameUIAction,
};
use crate::ui::prelude::InteractionQuery;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (handle_gameplay_action, handle_labels));
}

fn handle_gameplay_action(
    mut button_query: InteractionQuery<&CardGameUIAction>,
    mut ew_shuffle: EventWriter<DeckShuffle>,
    mut ew_draw: EventWriter<DrawHand>,
    mut ew_advance_phase: EventWriter<AdvancePhase>,
    mut ew_drop_chip: EventWriter<DropChip>,
    mut ew_move_chip: EventWriter<MoveChip>,
    mut ew_switch_player: EventWriter<SwitchPlayer>,
    decks: Query<(Entity, &DeckArea)>,
    chips: Query<(Entity, &Transform, &Chip<ChipType>, &ChipArea)>,
    state: Res<GameState>,
) {
    for (interaction, action) in &mut button_query {
        let deck_entity = decks.iter().next().unwrap().0;

        if matches!(interaction, Interaction::Pressed) {
            match action {
                CardGameUIAction::ButtonShuffleDeck => {
                    let event = DeckShuffle { deck_marker: 1 };
                    ew_shuffle.send(event);
                }
                CardGameUIAction::ButtonDrawHand => {
                    let event = DrawHand {
                        deck_marker: 1,
                        num_cards: 5,
                        player: state.player,
                    };
                    ew_draw.send(event);
                    ew_advance_phase.send(AdvancePhase);
                }
                CardGameUIAction::ButtonDropChip => {
                    let event = DropChip {
                        chip_type: ChipType::Cannabis,
                        area: 1,
                        player: state.player,
                    };
                    ew_drop_chip.send(event);
                }
                CardGameUIAction::ButtonMoveChip => {
                    // 1. find highest chip in area
                    let chips_in_area = chips
                        .iter()
                        .filter(|(_, _, chip, area)| {
                            chip.data == ChipType::Cannabis && area.marker == 1
                        })
                        .count();

                    if chips_in_area == 0 {
                        continue;
                    }

                    let chip_entity = chips
                        .iter()
                        .filter(|(_, _, chip, area)| {
                            chip.data == ChipType::Cannabis && area.marker == 1
                        })
                        .max_by_key(|(_, transform, _, _)| {
                            (transform.translation.z * 100.0) as usize
                        })
                        .map(|(entity, _, _, _)| entity)
                        .unwrap();

                    let event = MoveChip {
                        entity: chip_entity,
                        area: 2,
                        player: state.player,
                    };
                    ew_move_chip.send(event);
                }
                CardGameUIAction::ButtonAdvancePhase => {
                    ew_advance_phase.send(AdvancePhase);
                }
                CardGameUIAction::ButtonSwitchPlayer => {
                    ew_switch_player.send(SwitchPlayer {
                        player: match state.player {
                            1 => 2,
                            2 => 1,
                            _ => 1,
                        },
                    });
                }
                _ => {}
            }
        }
    }
}

fn handle_labels(
    mut label_query: Query<(Entity, &mut Visibility, &CardGameUIAction)>,
    mut text_query: Query<(&Parent, &mut Text)>,
    state: Res<GameState>,
) {
    for (entity, mut visibility, ui_element) in &mut label_query {
        for (parent, mut text) in text_query.iter_mut() {
            if parent.index() != entity.index() {
                continue;
            }
            match ui_element {
                CardGameUIAction::LabelTurnNumber => {
                    text.sections[0].value = format!("Turn number: {}", state.turn_number);
                }
                CardGameUIAction::LabelTurnPhase => {
                    text.sections[0].value = format!("Turn phase: {:?}", state.phase);
                }
                CardGameUIAction::ButtonShuffleDeck => {
                    if state.phase == TurnPhase::Prepare {
                        *visibility = Visibility::Visible;
                    } else {
                        *visibility = Visibility::Hidden;
                    }
                }
                CardGameUIAction::ButtonDrawHand => {
                    if state.phase == TurnPhase::Prepare {
                        *visibility = Visibility::Visible;
                    } else {
                        *visibility = Visibility::Hidden;
                    }
                }
                CardGameUIAction::LabelPhaseDescription => {
                    text.sections[0].value = match state.phase {
                        TurnPhase::Prepare => {
                            "You may shuffle the deck and draw 5 cards".to_string()
                        }
                        TurnPhase::PlaceCardsOnTable => {
                            "You may play cards from your hand or draw".to_string()
                        }
                        TurnPhase::Event => "Draw a card from event deck and play it".to_string(),
                        TurnPhase::End => "Update your counters and pass turn".to_string(),
                        TurnPhase::ApplyProductionCards => "Applying Production Cards".to_string(),
                        TurnPhase::ApplyTransportationCards => {
                            "Applying Transportation Cards".to_string()
                        }
                        TurnPhase::ApplySalesCards => "Applying Sales Cards".to_string(),
                    };
                }
                CardGameUIAction::ButtonDropChip => {}
                CardGameUIAction::ButtonMoveChip => {}
                CardGameUIAction::ButtonAdvancePhase => {}
                CardGameUIAction::LabelPlayerNumber => {
                    text.sections[0].value = format!("Player number: {}", state.player)
                }
                CardGameUIAction::ButtonSwitchPlayer => {}
                CardGameUIAction::LabelBank => {
                    text.sections[0].value = format!("Bank: ${}", state.get_balance(state.player));
                }
            }
        }
    }
}
