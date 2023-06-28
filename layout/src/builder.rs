//! A convinient way to build an UI using `cuicui_layout`

fn foo() {
    layout! {
        // `row` and `col` start a container with given `Flow`. By default
        // they have Distribution::Start. and Alignment::Start.
        row {
            space(100 px);
            // main_margin here actually creates 2 nested containers
            // The outer one has same distr as child and align Start, the inner one has the
            // declared distr and align
            let menu = col(Distribution::FillParent, main_margin: 100 px) {
                image(game_logo);
                // fix accepts a
                fix(continue_button);
                fix(new_game_button);
                fix(load_button);
                fix(settings_button);
                fix(additional_button);
                fix(credits_button);
                fix(quit_button);
            }
        }
    }
}
