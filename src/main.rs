use eframe::{egui, epi};
use egui::Key;
use std::collections::HashMap;

use std::sync::atomic::{AtomicUsize, Ordering};

const QUESTS: &[u8] = include_bytes!("../quests.json");

static UP: AtomicUsize = AtomicUsize::new(0);
static DOWN: AtomicUsize = AtomicUsize::new(0);
static ADD_NOD: AtomicUsize = AtomicUsize::new(0);
static SUB_NOD: AtomicUsize = AtomicUsize::new(0);

#[derive(serde::Deserialize)]
struct Quest {
    level: u32,
    name: String,
}

#[derive(serde::Deserialize)]
struct QuestChain {
    name: String,
    quests: Vec<Quest>,
}

struct QuestInfo {
    chains: Vec<QuestChain>,
}

impl Default for QuestInfo {
    fn default() -> Self {
        QuestInfo {
            chains: serde_json::from_slice(QUESTS).unwrap(),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Nodders {
    // quest name -> nod counter
    nods: HashMap<String, usize>,
    selected_quest: Option<(usize, usize)>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    filter: String,

    #[cfg_attr(feature = "persistence", serde(skip))]
    quest_info: QuestInfo,
}

impl Default for Nodders {
    fn default() -> Self {
        Self {
            nods: HashMap::default(),
            selected_quest: None,
            filter: String::new(),
            quest_info: QuestInfo::default(),
        }
    }
}

impl epi::App for Nodders {
    fn name(&self) -> &str {
        "nodders"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, "nodders").unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, "nodders", self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut advanced = false;

            let input = ctx.input();
            if input.key_pressed(Key::W)
                || input.key_pressed(egui::Key::ArrowUp)
                || UP.load(Ordering::SeqCst) != 0
            {
                let (chain_idx, quest_idx) = self.selected_quest.unwrap_or_default();
                let is_start_of_chain = quest_idx == 0;

                if is_start_of_chain {
                    let previous_chain_len = self
                        .quest_info
                        .chains
                        .get(chain_idx.saturating_sub(1))
                        .map(|chain| chain.quests.len())
                        .unwrap_or(1);
                    self.selected_quest = Some((
                        chain_idx.saturating_sub(1),
                        previous_chain_len.saturating_sub(1),
                    ));
                } else {
                    self.selected_quest = Some((chain_idx, quest_idx.saturating_sub(1)));
                }

                advanced = true;
                UP.store(0, Ordering::SeqCst);
            }

            if input.key_pressed(Key::S)
                || input.key_pressed(Key::ArrowDown)
                || DOWN.load(Ordering::SeqCst) != 0
            {
                let (chain_idx, quest_idx) = self.selected_quest.unwrap_or_default();
                let is_end_of_chain = self
                    .quest_info
                    .chains
                    .get(chain_idx)
                    .and_then(|chain| chain.quests.get(quest_idx + 1))
                    .is_none();

                if is_end_of_chain {
                    self.selected_quest = Some((chain_idx + 1, 0));
                } else {
                    self.selected_quest = Some((chain_idx, quest_idx + 1));
                }

                advanced = true;
                DOWN.store(0, Ordering::SeqCst);
            }

            ui.horizontal(|ui| {
                ui.label("filter:");
                ui.text_edit_singleline(&mut self.filter);
                ui.label("Total nods:");
                let total_nods: usize = self.nods.values().sum();
                ui.label(format_args!("{}", total_nods));
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([true; 2])
                .show_viewport(ui, |ui, _viewport| {
                    let filter = self.filter.to_lowercase();

                    for (chain_idx, chain) in self.quest_info.chains.iter().enumerate() {
                        ui.spacing();
                        ui.heading(&chain.name);
                        ui.separator();
                        for (quest_idx, quest) in chain.quests.iter().enumerate() {
                            if !quest.name.to_lowercase().contains(&filter) {
                                continue;
                            }

                            ui.horizontal(|ui| {
                                ui.label(format_args!("{}", quest.level));
                                ui.spacing();

                                let quest_button = ui.add(
                                    egui::Button::new(&quest.name).fill(egui::Color32::TRANSPARENT),
                                );

                                if quest_button.clicked() {
                                    self.selected_quest = Some((chain_idx, quest_idx));
                                }

                                if self.selected_quest == Some((chain_idx, quest_idx)) {
                                    if advanced && !ui.clip_rect().contains_rect(quest_button.rect)
                                    {
                                        quest_button.scroll_to_me(egui::Align::Center);
                                    }

                                    let plus = ui.add(
                                        egui::Button::new("+").text_style(egui::TextStyle::Heading),
                                    );
                                    if plus.clicked()
                                        || input.key_pressed(Key::A)
                                        || input.key_pressed(Key::ArrowLeft)
                                        || ADD_NOD.load(Ordering::SeqCst) != 0
                                    {
                                        let nods = self.nods.entry(quest.name.clone()).or_default();
                                        *nods += 1;
                                        ADD_NOD.store(0, Ordering::SeqCst);
                                    }
                                    ui.spacing();
                                    let minus = ui.add(
                                        egui::Button::new("-").text_style(egui::TextStyle::Heading),
                                    );

                                    if minus.clicked()
                                        || input.key_pressed(Key::D)
                                        || input.key_pressed(Key::ArrowRight)
                                        || SUB_NOD.load(Ordering::SeqCst) != 0
                                    {
                                        let nods = self.nods.entry(quest.name.clone()).or_default();
                                        if *nods == 0 {
                                            self.nods.remove(&quest.name);
                                        } else {
                                            *nods -= 1;
                                        }
                                        SUB_NOD.store(0, Ordering::SeqCst);
                                    }
                                }

                                if let Some(nods) = self.nods.get(&quest.name) {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::RIGHT),
                                        |ui| {
                                            ui.label(format_args!("{}", nods));
                                        },
                                    );
                                }
                            });
                        }
                    }
                });
        });
    }
}

fn main() {
    inputbot::KeybdKey::Numpad8Key.bind(|| {
        UP.fetch_add(1, Ordering::SeqCst);
    });

    inputbot::KeybdKey::Numpad2Key.bind(|| {
        DOWN.fetch_add(1, Ordering::SeqCst);
    });

    inputbot::KeybdKey::Numpad4Key.bind(|| {
        ADD_NOD.fetch_add(1, Ordering::SeqCst);
    });

    inputbot::KeybdKey::Numpad6Key.bind(|| {
        SUB_NOD.fetch_add(1, Ordering::SeqCst);
    });

    std::thread::spawn(inputbot::handle_input_events);

    let app = Nodders::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
