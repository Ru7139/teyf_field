mod project {
    // use freya::prelude::*;

    #[test]
    fn main() {
        // launch(app);

        let mut c = Counter::new();
        c.increment_one();
        assert_eq!(c, Counter(1i64));
        c.decrement_one();
        assert_eq!(c, Counter(0i64));
    }

    #[derive(Debug, PartialEq)]
    struct Counter(i64);
    impl Counter {
        fn new() -> Counter {
            Counter(0i64)
        }
        fn increment_one(&mut self) {
            self.0 += 1
        }
        fn decrement_one(&mut self) {
            self.0 -= 1
        }
    }

    // #[allow(unused)]
    // fn app() -> Element {
    //     let mut count = use_signal(|| 0i64);

    //     rsx!(
    //         rect {
    //             width: "100%",
    //             height: "50%",
    //             main_align: "center",
    //             cross_align: "center",
    //             background: "blue",
    //             color: "white",
    //             label { font_size: "70%", "{count}"}
    //         }
    //         rect {
    //             width: "100%",
    //             height: "50%",
    //             main_align: "center",
    //             cross_align: "center",
    //             direction: "horizontal",
    //             Button {
    //                 onclick: move |_| count += 1,
    //                 label {"Increase"}
    //             }
    //             Button {
    //                 onclick: move |_| count -= 1,
    //                 label { "Decrease" }
    //             }
    //         }
    //     )
    // }
}
