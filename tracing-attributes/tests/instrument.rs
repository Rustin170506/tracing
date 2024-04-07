use tracing::collect::with_default;
use tracing::Level;
use tracing_attributes::instrument;
use tracing_mock::*;

// Reproduces a compile error when an instrumented function body contains inner
// attributes (https://github.com/tokio-rs/tracing/issues/2294).
#[deny(unused_variables)]
#[instrument]
fn repro_2294() {
    #![allow(unused_variables)]
    let i = 42;
}

#[test]
fn override_everything() {
    #[instrument(target = "my_target", level = "debug")]
    fn my_fn() {}

    #[instrument(level = Level::DEBUG, target = "my_target")]
    fn my_other_fn() {}

    let span = expect::span()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let span2 = expect::span()
        .named("my_other_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let (collector, handle) = collector::mock()
        .new_span(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .new_span(span2.clone())
        .enter(span2.clone())
        .exit(span2.clone())
        .drop_span(span2)
        .only()
        .run_with_handle();

    with_default(collector, || {
        my_fn();
        my_other_fn();
    });

    handle.assert_finished();
}

#[test]
fn fields() {
    #[instrument(target = "my_target", level = "debug")]
    fn my_fn(arg1: usize, arg2: bool, arg3: String) {}

    let span = expect::span()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");

    let span2 = expect::span()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let (collector, handle) = collector::mock()
        .new_span(
            span.clone().with_fields(
                expect::field("arg1")
                    .with_value(&2usize)
                    .and(expect::field("arg2").with_value(&false))
                    .and(expect::field("arg3").with_value(&"Cool".to_string()))
                    .only(),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .new_span(
            span2.clone().with_fields(
                expect::field("arg1")
                    .with_value(&3usize)
                    .and(expect::field("arg2").with_value(&true))
                    .and(expect::field("arg3").with_value(&"Still Cool".to_string()))
                    .only(),
            ),
        )
        .enter(span2.clone())
        .exit(span2.clone())
        .drop_span(span2)
        .only()
        .run_with_handle();

    with_default(collector, || {
        my_fn(2, false, "Cool".to_string());
        my_fn(3, true, "Still Cool".to_string());
    });

    handle.assert_finished();
}

#[test]
fn skip() {
    struct UnDebug;

    #[instrument(target = "my_target", level = "debug", skip(_arg2, _arg3))]
    fn my_fn(arg1: usize, _arg2: UnDebug, _arg3: UnDebug) {}

    let span = expect::span()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");

    let span2 = expect::span()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let (collector, handle) = collector::mock()
        .new_span(
            span.clone()
                .with_fields(expect::field("arg1").with_value(&2usize).only()),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .new_span(
            span2
                .clone()
                .with_fields(expect::field("arg1").with_value(&3usize).only()),
        )
        .enter(span2.clone())
        .exit(span2.clone())
        .drop_span(span2)
        .only()
        .run_with_handle();

    with_default(collector, || {
        my_fn(2, UnDebug, UnDebug);
        my_fn(3, UnDebug, UnDebug);
    });

    handle.assert_finished();
}

#[test]
fn generics() {
    #[derive(Debug)]
    struct Foo;

    #[instrument]
    fn my_fn<S, T: std::fmt::Debug>(arg1: S, arg2: T)
    where
        S: std::fmt::Debug,
    {
    }

    let span = expect::span().named("my_fn");

    let (collector, handle) = collector::mock()
        .new_span(
            span.clone().with_fields(
                expect::field("arg1")
                    .with_value(&format_args!("Foo"))
                    .and(expect::field("arg2").with_value(&format_args!("false"))),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .only()
        .run_with_handle();

    with_default(collector, || {
        my_fn(Foo, false);
    });

    handle.assert_finished();
}

#[test]
fn methods() {
    #[derive(Debug)]
    struct Foo;

    impl Foo {
        #[instrument]
        fn my_fn(&self, arg1: usize) {}
    }

    let span = expect::span().named("my_fn");

    let (collector, handle) = collector::mock()
        .new_span(
            span.clone().with_fields(
                expect::field("self")
                    .with_value(&format_args!("Foo"))
                    .and(expect::field("arg1").with_value(&42usize)),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .only()
        .run_with_handle();

    with_default(collector, || {
        let foo = Foo;
        foo.my_fn(42);
    });

    handle.assert_finished();
}

#[test]
fn impl_trait_return_type() {
    #[instrument]
    fn returns_impl_trait(x: usize) -> impl Iterator<Item = usize> {
        0..x
    }

    let span = expect::span().named("returns_impl_trait");

    let (collector, handle) = collector::mock()
        .new_span(
            span.clone()
                .with_fields(expect::field("x").with_value(&10usize).only()),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .only()
        .run_with_handle();

    with_default(collector, || {
        for _ in returns_impl_trait(10) {
            // nop
        }
    });

    handle.assert_finished();
}
