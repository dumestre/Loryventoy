use eframe::egui::{
    Color32,
    CursorIcon,
    Response,
    Sense,
    Ui,
    Vec2,
};


pub struct VerticalSplitter {

    pub size: f32,

}


impl VerticalSplitter {


    pub fn new(
        size: f32
    ) -> Self {

        Self {
            size
        }

    }



    pub fn show(
        &self,
        ui: &mut Ui,
    ) -> Response {


        let (rect, response) =
            ui.allocate_exact_size(
                Vec2::new(
                    ui.available_width(),
                    self.size
                ),
                Sense::drag()
            );



        let painter =
            ui.painter();



        let color =
            if response.hovered() || response.dragged() {

                Color32::from_gray(160)

            } else {

                Color32::from_rgb(45, 45, 55)

            };


        painter.rect_filled(
            rect,
            0.0,
            color
        );


        response.on_hover_cursor(
            CursorIcon::ResizeVertical
        )

    }

}