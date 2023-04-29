use crate::prelude::*;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tooltip>()
            .add_system(render_active_tooltip.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource, Default)]
pub struct Tooltip {
    pub value: Option<String>,
}

fn render_active_tooltip(
    tooltip_value: Res<Tooltip>,
    mut text_query: Query<(&Name, &Node, Option<&mut Text>, &mut Visibility)>,
) {
    for (name, node, mut text, mut visibility) in text_query.iter_mut() {
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
