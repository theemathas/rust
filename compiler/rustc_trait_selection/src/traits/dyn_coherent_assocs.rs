use rustc_middle::query::Providers;
use rustc_middle::ty::{Ty, TyCtxt};

fn does_dyn_have_coherent_assocs(_tcx: TyCtxt<'_>, _ty: Ty<'_>) -> bool {
    true
}

pub(crate) fn provide(providers: &mut Providers) {
    *providers = Providers { does_dyn_have_coherent_assocs, ..*providers };
}
