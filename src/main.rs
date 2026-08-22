use draft::{
    app::App,
    expression,
    geom::{self},
    slint_gen,
};
use slint::{
    ComponentHandle,
    wgpu_29::{WGPUSettings, wgpu},
};
use std::{
    cell::RefCell,
    error::Error,
    f64::consts::{FRAC_PI_2, FRAC_PI_8},
    rc::Rc,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut settings = WGPUSettings::default();
    settings.device_required_limits = wgpu::Limits::defaults();

    slint::BackendSelector::new()
        .require_wgpu_29(slint::wgpu_29::WGPUConfiguration::Automatic(settings))
        .select()?;

    let main_window = slint_gen::MainWindow::new()?;

    let app = Rc::new(RefCell::new(App::new(main_window.as_weak())));
    App::init_callbacks(app.clone())?;

    {
        let mut app = app.borrow_mut();
        let arena = app.arena_mut();

        let p = arena.add_point_free(geom::point2(200., 300.))?;

        let q = arena.add_point_dist_angle(p, expression::length(200.), expression::angle(0.))?;

        let r = arena.add_point_dist_angle(
            p,
            expression::length(200.),
            expression::angle(FRAC_PI_2),
        )?;

        let c = arena.add_curve(
            p,
            q,
            geom::polar(150., -FRAC_PI_2),
            geom::polar(100., -FRAC_PI_8),
        )?;

        arena.add_point_on_line(p, r, expression::length(50.))?;

        arena.add_point_on_curve(c, expression::length(90.))?;

        arena.evaluate_all()?;
    }

    main_window.run()?;
    Ok(())
}
