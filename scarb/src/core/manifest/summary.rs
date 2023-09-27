use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

use once_cell::sync::Lazy;
use typed_builder::TypedBuilder;

#[cfg(doc)]
use crate::core::Manifest;
use crate::core::{
    DepKind, DependencyVersionReq, ManifestDependency, PackageId, PackageName, SourceId, TargetKind,
};

/// Subset of a [`Manifest`] that contains only the most important information about a package.
/// See [`SummaryInner`] for public fields reference.
/// Construct using [`Summary::builder`].
#[derive(Clone, Debug)]
pub struct Summary(Arc<SummaryInner>);

#[derive(TypedBuilder, Debug)]
#[builder(builder_type(name = SummaryBuilder))]
#[builder(builder_method(vis = ""))]
#[builder(build_method(into = Summary))]
#[non_exhaustive]
pub struct SummaryInner {
    pub package_id: PackageId,
    #[builder(default)]
    pub dependencies: Vec<ManifestDependency>,
    pub target_kinds: HashSet<TargetKind>,
    #[builder(default = false)]
    pub no_core: bool,
}

impl Deref for Summary {
    type Target = SummaryInner;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[doc(hidden)]
impl From<SummaryInner> for Summary {
    fn from(data: SummaryInner) -> Self {
        Self(Arc::new(data))
    }
}

impl Summary {
    pub fn builder() -> SummaryBuilder {
        SummaryInner::builder()
    }

    pub fn full_dependencies(&self) -> impl Iterator<Item = &ManifestDependency> {
        self.dependencies.iter().chain(self.implicit_dependencies())
    }

    pub fn implicit_dependencies(&self) -> impl Iterator<Item = &ManifestDependency> {
        static CORE_DEPENDENCY: Lazy<ManifestDependency> = Lazy::new(|| {
            // NOTE: Pin `core` to exact version, because we know that's the only one we have.
            let cairo_version = crate::version::get().cairo.version.parse().unwrap();
            ManifestDependency::builder()
                .name(PackageName::CORE)
                .version_req(DependencyVersionReq::exact(&cairo_version))
                .build()
        });

        static TEST_PLUGIN_DEPENDENCY: Lazy<ManifestDependency> = Lazy::new(|| {
            // NOTE: Pin test plugin to exact version, because we know that's the only one we have.
            let cairo_version = crate::version::get().cairo.version.parse().unwrap();
            ManifestDependency::builder()
                .kind(DepKind::Target(TargetKind::TEST))
                .name(PackageName::TEST_PLUGIN)
                .source_id(SourceId::default())
                .version_req(DependencyVersionReq::exact(&cairo_version))
                .build()
        });

        let mut deps: Vec<&ManifestDependency> = Vec::new();

        if !self.no_core {
            deps.push(&CORE_DEPENDENCY);
            if self.target_kinds.contains(&TargetKind::TEST) {
                deps.push(&TEST_PLUGIN_DEPENDENCY);
            }
        }

        deps.into_iter()
    }
}
