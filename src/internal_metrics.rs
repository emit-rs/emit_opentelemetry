use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

macro_rules! metrics {
    (
        $pub_container:ty {
            $field:ident: $internal_container:ident {
                $(
                    $(#[$meta:meta])*
                    $metric:ident: $ty:ident -> $pub_ty:ident,
                )*
            }
        }
    ) => {
        #[derive(Default)]
        pub(crate) struct $internal_container {
            $(
                $(#[$meta])*
                pub(crate) $metric: $ty,
            )*
        }

        impl $internal_container {
            pub fn sample1(&self) -> impl Iterator<Item = emit1::metric::Metric<'static, emit::empty::Empty>> + 'static {
                let $internal_container { $($metric),* } = self;

                [$(
                    emit1::metric::Metric::new(
                        emit::pkg!(),
                        stringify!($metric),
                        <$ty>::AGG,
                        emit::empty::Empty,
                        $metric.sample(),
                        emit::empty::Empty,
                    ),
                )*]
                .into_iter()
            }
        }

        impl emit::metric::Source for $internal_container {
            fn sample_metrics<S: emit::metric::Sampler>(&self, sampler: S) {
                let $internal_container { $($metric),* } = self;

                $(
                    sampler.metric(emit::metric::Metric::new(
                        emit::pkg!(),
                        emit::Empty,
                        emit::props! {
                            metric_name: stringify!($metric),
                            metric_agg: <$ty>::AGG,
                            metric_value: $metric.sample(),
                        },
                    ));
                )*
            }
        }

        impl $pub_container {
            $(
                $(#[$meta])*
                pub fn $metric(&self) -> $pub_ty {
                    self.$field.$metric.sample()
                }
            )*
        }
    };
}

#[derive(Default)]
pub(crate) struct Counter(AtomicUsize);

impl Counter {
    const AGG: &'static str = emit::well_known::METRIC_AGG_COUNT;

    pub fn increment(&self) {
        self.increment_by(1);
    }

    pub fn increment_by(&self, by: usize) {
        self.0.fetch_add(by, Ordering::Relaxed);
    }

    pub fn sample(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

metrics!(
    EmitOpenTelemetryMetrics {
        metrics: InternalMetrics {
            /**
            An OpenTelemetry SDK span created through `emit` was closed without its corresponding [`emit::Event`] being emitted.

            The span was still closed, but will be missing properties.
            */
            span_unexpected_close: Counter -> usize,
            /**
            An [`emit::Event`] for a span was emitted that didn't correspond to the currently active OpenTelemetry SDK span created through `emit`.

            The span was emitted as an OpenTelemetry SDK log record instead of as a span.
            */
            span_unexpected_emit: Counter -> usize,
        }
    }
);

/**
Metrics produced by the `emit` to the OpenTelemetry SDK integration itself.

This type doesn't collect any OTLP metrics you emit, it includes metrics about this library's own activity.

You can enumerate the metrics using the [`emit::metric::Source`] implementation. See [`emit::metric`] for details.
*/
pub struct EmitOpenTelemetryMetrics {
    pub(crate) metrics: Arc<InternalMetrics>,
}

impl emit1::metric::Source for EmitOpenTelemetryMetrics {
    fn sample_metrics<S: emit1::metric::sampler::Sampler>(&self, sampler: S) {
        for metric in self.metrics.sample1() {
            sampler.metric(metric);
        }
    }
}

impl emit::metric::Source for EmitOpenTelemetryMetrics {
    fn sample_metrics<S: emit::metric::Sampler>(&self, sampler: S) {
        self.metrics.sample_metrics(sampler)
    }
}
