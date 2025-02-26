use eframe::egui::{Ui, Context, SidePanel, ScrollArea};

use crate::{*, library::LibraryEntry, util::{RIGHT_PANEL_WIDTH, MIN_LIBRARY_WIDTH, DEFAULT_LIBRARY_WIDTH}};

use super::{GdSfx, Tab, Sorting};

pub fn render(gdsfx: &mut GdSfx, ctx: &Context) {
    SidePanel::left("left_panel")
    .min_width(MIN_LIBRARY_WIDTH)
    .max_width(RIGHT_PANEL_WIDTH)
    .default_width(DEFAULT_LIBRARY_WIDTH)
    .show(ctx, |ui| {
        if let Tab::Library | Tab::Favourites = gdsfx.tab {
            add_search_area(ui, gdsfx);
        }
        
        ScrollArea::vertical().show(ui, |ui| {
            if let Some(sfx_library) = &gdsfx.sfx_library {
                let mut library = sfx_library.sound_effects.clone();
                filter_sounds(gdsfx, &mut library);
                match gdsfx.tab {
                    Tab::Library => library::gui::render(ui, gdsfx, library),
                    Tab::Favourites => favorites::gui::render(ui, gdsfx, library),
                    Tab::Tools => tools::gui::render(ui, gdsfx, ctx),
                    Tab::Settings => settings::gui::render(ui, gdsfx),
                    Tab::Stats => stats::gui::render(ui, gdsfx),
                    Tab::Credits => credits::gui::render(ui, gdsfx),
                }
            }
        });
    });
}

fn add_search_area(ui: &mut Ui, gdsfx: &mut GdSfx) {
    ui.heading(t!("search"));
    ui.text_edit_singleline(&mut gdsfx.search_query);

    ui.menu_button(t!("sort.button"), |ui| {
        for (alternative, text) in [
            (Sorting::Default,   t!("sort.default")),
            (Sorting::NameInc,   t!("sort.name.ascending")),
            (Sorting::NameDec,   t!("sort.name.descending")),
            (Sorting::LengthInc, t!("sort.length.ascending")),
            (Sorting::LengthDec, t!("sort.length.descending")),
            (Sorting::IdDec,     t!("sort.id.ascending")),  // this is not a bug, in gd, the id sorting is reversed,
            (Sorting::IdInc,     t!("sort.id.descending")), // in-game it's `ID+ => 9 - 0; ID- => 0 - 9`
            (Sorting::SizeInc,   t!("sort.size.ascending")),
            (Sorting::SizeDec,   t!("sort.size.descending")),
        ] {
            let response = ui.radio_value(&mut gdsfx.sorting, alternative, text);
            if response.clicked() {
                ui.close_menu();
            }
        }
    });

    ui.separator();
}

fn filter_sounds(gdsfx: &mut GdSfx, node: &mut LibraryEntry) {
    match node {
        LibraryEntry::Sound { .. } => {
            node.set_enabled(gdsfx.matches_query(node));
        }
        LibraryEntry::Category { children, .. } => {
            for child in children.iter_mut() {
                filter_sounds(gdsfx, child);
            }

            let any_enabled = children.iter().any(LibraryEntry::is_enabled);
            node.set_enabled(any_enabled);
        }
    }
}
