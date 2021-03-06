mod destinations;
mod kml;
pub mod mapping;
mod polygon;
mod scenario;
mod story;

use crate::app::App;
use crate::common::CityPicker;
use crate::game::{DrawBaselayer, State, Transition, WizardState};
use crate::helpers::nice_map_name;
use abstutil::Timer;
use ezgui::{
    hotkey, Btn, Composite, EventCtx, GfxCtx, HorizontalAlignment, Key, Line, Outcome, TextExt,
    VerticalAlignment, Widget, Wizard,
};
use geom::LonLat;

pub struct DevToolsMode {
    composite: Composite,
}

impl DevToolsMode {
    pub fn new(ctx: &mut EventCtx, app: &App) -> Box<dyn State> {
        Box::new(DevToolsMode {
            composite: Composite::new(Widget::col(vec![
                Widget::row(vec![
                    Line("Internal dev tools").small_heading().draw(ctx),
                    Btn::text_fg("X")
                        .build(ctx, "close", hotkey(Key::Escape))
                        .align_right(),
                ]),
                Widget::row(vec![
                    "Change map:".draw_text(ctx),
                    Btn::text_fg(format!("{} ↓", nice_map_name(app.primary.map.get_name()))).build(
                        ctx,
                        "change map",
                        None,
                    ),
                ]),
                Widget::custom_row(vec![
                    Btn::text_fg("edit a polygon").build_def(ctx, hotkey(Key::E)),
                    Btn::text_fg("draw a polygon").build_def(ctx, hotkey(Key::P)),
                    Btn::text_fg("load scenario").build_def(ctx, hotkey(Key::W)),
                    Btn::text_fg("view KML").build_def(ctx, hotkey(Key::K)),
                    Btn::text_fg("story maps").build_def(ctx, hotkey(Key::S)),
                ])
                .flex_wrap(ctx, 60),
            ]))
            .aligned(HorizontalAlignment::Center, VerticalAlignment::Top)
            .build(ctx),
        })
    }
}

impl State for DevToolsMode {
    fn event(&mut self, ctx: &mut EventCtx, app: &mut App) -> Transition {
        match self.composite.event(ctx) {
            Some(Outcome::Clicked(x)) => match x.as_ref() {
                "close" => {
                    return Transition::Pop;
                }
                "edit a polygon" => {
                    return Transition::Push(WizardState::new(Box::new(choose_polygon)));
                }
                "draw a polygon" => {
                    return Transition::Push(polygon::PolygonEditor::new(
                        ctx,
                        "name goes here".to_string(),
                        Vec::new(),
                    ));
                }
                "load scenario" => {
                    return Transition::Push(WizardState::new(Box::new(load_scenario)));
                }
                "view KML" => {
                    return Transition::Push(WizardState::new(Box::new(choose_kml)));
                }
                "story maps" => {
                    return Transition::Push(story::StoryMapEditor::new(ctx));
                }
                "change map" => {
                    return Transition::Push(CityPicker::new(
                        ctx,
                        app,
                        Box::new(|ctx, app| {
                            Transition::PopThenReplace(DevToolsMode::new(ctx, app))
                        }),
                    ));
                }
                _ => unreachable!(),
            },
            None => {}
        }

        Transition::Keep
    }

    fn draw_baselayer(&self) -> DrawBaselayer {
        DrawBaselayer::Custom
    }

    fn draw(&self, g: &mut GfxCtx, app: &App) {
        g.clear(app.cs.grass);
        self.composite.draw(g);
    }
}

fn load_scenario(wiz: &mut Wizard, ctx: &mut EventCtx, app: &mut App) -> Option<Transition> {
    let map_name = app.primary.map.get_name().to_string();
    let s = wiz.wrap(ctx).choose_string("Load which scenario?", || {
        abstutil::list_all_objects(abstutil::path_all_scenarios(&map_name))
    })?;
    let scenario = abstutil::read_binary(
        abstutil::path_scenario(&map_name, &s),
        &mut Timer::throwaway(),
    );
    Some(Transition::Replace(Box::new(
        scenario::ScenarioManager::new(scenario, ctx, app),
    )))
}

fn choose_polygon(wiz: &mut Wizard, ctx: &mut EventCtx, _: &mut App) -> Option<Transition> {
    // TODO Sorry, Seattle only right now
    let name = wiz.wrap(ctx).choose_string("Edit which polygon?", || {
        abstutil::list_all_objects(abstutil::path("input/seattle/polygons/"))
    })?;
    match LonLat::read_osmosis_polygon(abstutil::path(format!(
        "input/seattle/polygons/{}.poly",
        name
    ))) {
        Ok(pts) => Some(Transition::Replace(polygon::PolygonEditor::new(
            ctx, name, pts,
        ))),
        Err(err) => {
            println!("Bad polygon {}: {}", name, err);
            Some(Transition::Pop)
        }
    }
}

fn choose_kml(wiz: &mut Wizard, ctx: &mut EventCtx, app: &mut App) -> Option<Transition> {
    let path = wiz.wrap(ctx).choose_string("View what KML dataset?", || {
        abstutil::list_dir(std::path::Path::new(&abstutil::path(format!(
            "input/{}/",
            app.primary.map.get_city_name()
        ))))
        .into_iter()
        .filter(|x| x.ends_with(".bin") && !x.ends_with("popdat.bin"))
        .collect()
    })?;
    Some(Transition::Replace(kml::ViewKML::new(ctx, app, path)))
}
