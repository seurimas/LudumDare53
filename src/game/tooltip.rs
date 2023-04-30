use crate::prelude::*;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tooltip>()
            .add_system(set_simple_helper_tooltips.run_if(in_state(GameState::Playing)))
            .add_system(render_active_tooltip.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct SimpleTooltip {
    pub value: String,
}

impl SimpleTooltip {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Resource, Default)]
pub struct Tooltip {
    pub value: Option<String>,
}

fn set_simple_helper_tooltips(
    mut tooltip_control: Local<bool>,
    mut tooltip_value: ResMut<Tooltip>,
    text_query: Query<(&SimpleTooltip, &ComputedVisibility, &RelativeCursorPosition)>,
) {
    let mut tooltip = None;
    for (simple_tooltip, _, _) in text_query
        .iter()
        .filter(|(_, visibility, rcp)| visibility.is_visible() && rcp.mouse_over())
    {
        tooltip = Some(simple_tooltip.value.clone());
    }
    if let Some(tooltip) = tooltip {
        tooltip_value.value = Some(tooltip);
        *tooltip_control = true;
    } else if *tooltip_control {
        *tooltip_control = false;
        tooltip_value.value = None;
    }
}

fn render_active_tooltip(
    tooltip_value: Res<Tooltip>,
    mut text_query: Query<(&Name, Option<&mut Text>, &mut Visibility)>,
) {
    for (name, text, mut visibility) in text_query.iter_mut() {
        if let Some(value) = tooltip_value.value.as_ref() {
            if name.eq_ignore_ascii_case("tooltip") {
                text.unwrap().sections[0].value = value.clone();
            } else if name.eq_ignore_ascii_case("tooltip_parent") {
                *visibility = Visibility::Visible;
            }
        } else {
            if name.eq_ignore_ascii_case("tooltip") {
                text.unwrap().sections[0].value = "".to_string();
            } else if name.eq_ignore_ascii_case("tooltip_parent") {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
