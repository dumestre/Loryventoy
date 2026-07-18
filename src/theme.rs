use eframe::egui::{
    Color32,
    Context,
    Visuals,
};



pub fn apply_theme(
    ctx: &Context,
) {


    let mut visuals =
        Visuals::dark();



    visuals.window_fill =
        Color32::from_rgb(
            55,
            55,
            68
        );


    visuals.panel_fill =
        Color32::from_rgb(
            50,
            50,
            62
        );


    visuals.extreme_bg_color =
        Color32::from_rgb(
            18,
            18,
            24
        );



    visuals.widgets.noninteractive.bg_fill =
        Color32::from_rgb(
            45,
            45,
            55
        );


    visuals.widgets.inactive.bg_fill =
        Color32::from_rgb(
            50,
            50,
            62
        );


    visuals.widgets.hovered.bg_fill =
        Color32::from_rgb(
            65,
            65,
            80
        );



    ctx.set_visuals(
        visuals
    );

}