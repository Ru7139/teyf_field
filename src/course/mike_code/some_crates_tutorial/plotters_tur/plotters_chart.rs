mod project {
    use plotters::prelude::*;

    #[test]
    fn draw_a_png() -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(
            "src/course/mike_code/some_crates_tutorial/plotters_tur/test.png",
            (1920, 1200),
        )
        .into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Test", ("sans-serif", 50).into_font())
            .margin(30)
            .x_label_area_size(20)
            .y_label_area_size(20)
            .build_cartesian_2d(-3f32..3f32, -3f32..3f32)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(
                (-1i32..=1i32).map(|x| x as f32).map(|x| (x, x)),
                // (-1f32..=1f32).map(|x| (x,x))
                // RangeInclusive<f32>` is not an iterator
                &RED,
            ))?
            .label("y = x")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&YELLOW)
            .border_style(&BLACK)
            .draw()?;

        root.present()?;

        Ok(())
    }
}
