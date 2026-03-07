#![allow(dead_code)]
use iced::Color;

// ─── Color Palette ───────────────────────────────────────────────────────────
pub const BG_DARK: Color = Color::BLACK;
pub const ORBITAL_GRAY: Color = Color {
    r: 0.102,
    g: 0.102,
    b: 0.102,
    a: 1.0,
}; // #1a1a1a
pub const ORBITAL_BORDER: Color = Color {
    r: 0.2,
    g: 0.2,
    b: 0.2,
    a: 1.0,
}; // #333333
pub const TEXT_PRIMARY: Color = Color::WHITE;
pub const TEXT_MUTED: Color = Color {
    r: 0.533,
    g: 0.533,
    b: 0.533,
    a: 1.0,
}; // #888888
pub const GREEN_ACCENT: Color = Color {
    r: 0.133,
    g: 0.773,
    b: 0.369,
    a: 1.0,
}; // #22c55e
pub const RED_ACCENT: Color = Color {
    r: 0.85,
    g: 0.22,
    b: 0.22,
    a: 1.0,
};
pub const BORDER_DIM: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.08,
}; // white/8
pub const BORDER_DIMMER: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.015,
}; // white/1.5 — barely visible
pub const HOVER_BG: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.1,
}; // white/10
pub const SIDEBAR_ACTIVE_BG: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.1,
}; // white/10
pub const DROPDOWN_BG: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.05,
    a: 1.0,
};

// ─── Style helpers ───────────────────────────────────────────────────────────
use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Shadow, Theme};

// --- Container styles ---
pub fn container_dark(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_DARK)),
        text_color: Some(TEXT_PRIMARY),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn container_header(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_DARK)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_DIM,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_sidebar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_DARK)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_DIM,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_breadcrumb(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_DARK)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_DIM,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_gray(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ORBITAL_GRAY)),
        text_color: Some(TEXT_MUTED),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn container_footer(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_DARK)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_DIM,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_progress_track(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 0.15,
            g: 0.15,
            b: 0.15,
            a: 1.0,
        })),
        text_color: None,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn container_progress_fill(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(TEXT_PRIMARY)),
        text_color: None,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn container_cli_input_area(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ORBITAL_GRAY)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: BORDER_DIM,
            width: 1.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_chevron(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.05,
        })),
        text_color: None,
        border: Border {
            color: BORDER_DIMMER,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_dropdown(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(DROPDOWN_BG)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.12,
            },
            width: 1.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn container_modal_overlay(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.82,
        })),
        text_color: Some(TEXT_PRIMARY),
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn container_modal_box(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(BG_DARK)),
        text_color: Some(TEXT_PRIMARY),
        border: Border {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.25,
            },
            width: 1.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

// --- Button styles ---
pub fn button_nav(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_MUTED,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_nav_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: TEXT_PRIMARY,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn button_sidebar(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(HOVER_BG)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: TEXT_PRIMARY,
                width: 0.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_MUTED,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_sidebar_active(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let _ = status;
    button::Style {
        background: Some(Background::Color(SIDEBAR_ACTIVE_BG)),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: TEXT_PRIMARY,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn button_breadcrumb(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(HOVER_BG)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(ORBITAL_GRAY)),
            text_color: TEXT_MUTED,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_breadcrumb_active(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let _ = status;
    button::Style {
        background: Some(Background::Color(TEXT_PRIMARY)),
        text_color: BG_DARK,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn button_file_row(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15,
            })),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_MUTED,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_dir_row(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15,
            })),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_exec_row(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15,
            })),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: GREEN_ACCENT,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_selected_row(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.12,
        })),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.25,
            },
            width: 1.0,
            radius: 0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn button_column_header(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_MUTED,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_column_header_active(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    let _ = status;
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: TEXT_PRIMARY,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn button_hovered_drop_target(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color {
            r: 0.133,
            g: 0.773,
            b: 0.369,
            a: 0.3,
        })),
        text_color: TEXT_PRIMARY,
        border: Border {
            color: GREEN_ACCENT,
            width: 1.0,
            radius: 4.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn button_context_menu(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.05,
            })),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_context_danger(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 0.85,
                g: 0.22,
                b: 0.22,
                a: 0.1,
            })),
            text_color: RED_ACCENT,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: RED_ACCENT,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_context_disabled(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color {
            r: 0.4,
            g: 0.4,
            b: 0.4,
            a: 1.0,
        },
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn button_context_backdrop(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::TRANSPARENT,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn button_exec(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 0.85,
                g: 0.85,
                b: 0.85,
                a: 1.0,
            })),
            text_color: BG_DARK,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(TEXT_PRIMARY)),
            text_color: BG_DARK,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_settings(theme: &Theme, status: button::Status) -> button::Style {
    let _ = theme;
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(TEXT_PRIMARY)),
            text_color: BG_DARK,
            border: Border {
                color: BORDER_DIM,
                width: 1.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(ORBITAL_GRAY)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: BORDER_DIM,
                width: 1.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
    }
}

pub fn button_menu_item(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(HOVER_BG)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_menu_item_active(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(HOVER_BG)),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.04,
            })),
            text_color: TEXT_PRIMARY,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_menu_item_danger(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                r: 0.85,
                g: 0.15,
                b: 0.15,
                a: 0.2,
            })),
            text_color: RED_ACCENT,
            border: Border::default(),
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: RED_ACCENT,
            border: Border::default(),
            shadow: Shadow::default(),
        },
    }
}

pub fn button_confirm(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(TEXT_PRIMARY)),
            text_color: BG_DARK,
            border: Border {
                color: TEXT_PRIMARY,
                width: 1.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: TEXT_MUTED,
                width: 1.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
    }
}

pub fn button_abort(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(TEXT_PRIMARY)),
            text_color: BG_DARK,
            border: Border {
                color: TEXT_PRIMARY,
                width: 1.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
        _ => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: TEXT_MUTED,
                width: 1.0,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
        },
    }
}

pub fn button_menu_backdrop(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::TRANSPARENT,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

// --- Text input styles ---
pub fn text_input_dark(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let _ = theme;
    let border_color = match status {
        text_input::Status::Focused => Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.5,
        },
        _ => BORDER_DIMMER,
    };
    text_input::Style {
        background: Background::Color(Color {
            r: 0.102,
            g: 0.102,
            b: 0.102,
            a: 0.5,
        }),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 0.into(),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.3,
        },
    }
}

pub fn text_input_cli(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let _ = theme;
    let _ = status;
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.into(),
        },
        icon: TEXT_MUTED,
        placeholder: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.2,
        },
        value: TEXT_PRIMARY,
        selection: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.3,
        },
    }
}

pub fn text_input_rename(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let _ = theme;
    let border_color = match status {
        text_input::Status::Focused => TEXT_PRIMARY,
        _ => TEXT_MUTED,
    };
    text_input::Style {
        background: Background::Color(Color {
            r: 0.08,
            g: 0.08,
            b: 0.08,
            a: 1.0,
        }),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 0.into(),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_MUTED,
        value: TEXT_PRIMARY,
        selection: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.3,
        },
    }
}

// --- Scrollable style ---
pub fn scrollable_dark(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let _ = theme;
    let rail_style = |hovered: bool| scrollable::Rail {
        background: Some(Background::Color(BG_DARK)),
        border: Border::default(),
        scroller: scrollable::Scroller {
            color: if hovered {
                Color {
                    r: 0.33,
                    g: 0.33,
                    b: 0.33,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.2,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                }
            },
            border: Border {
                color: BG_DARK,
                width: 1.0,
                radius: 0.into(),
            },
        },
    };
    match status {
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
        } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail_style(is_vertical_scrollbar_hovered),
            horizontal_rail: rail_style(is_horizontal_scrollbar_hovered),
            gap: None,
        },
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
        } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail_style(is_vertical_scrollbar_dragged),
            horizontal_rail: rail_style(is_horizontal_scrollbar_dragged),
            gap: None,
        },
        _ => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail_style(false),
            horizontal_rail: rail_style(false),
            gap: None,
        },
    }
}
