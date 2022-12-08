use super::*;

pub fn intersection_bound_trait_to_object_pool(
    ir_type_impl_traits_pool: HashSet<IrTypeImplTrait>,
    trait_to_impl_pool: &HashMap<String, Vec<Impl>>,
) -> HashMap<Vec<String>, HashSet<String>> {
    ir_type_impl_traits_pool
        .iter()
        .flat_map(|ty| &ty.trait_bounds)
        .for_each(|trait_| {
            if !trait_to_impl_pool.contains_key(trait_) {
                panic!("loss impl {} for some self_ty", trait_);
            }
        });

    ir_type_impl_traits_pool
        .into_iter()
        .map(|ty| ty.trait_bounds)
        .map(|trait_bounds| {
            let sets = trait_bounds.iter().map(|trait_| {
                let impls = trait_to_impl_pool.get(trait_).unwrap();
                let iter = impls.iter().map(|impl_| impl_.self_ty.to_string());
                HashSet::from_iter(iter)
            });

            let mut iter = sets;

            let intersection_set = iter
                .next()
                .map(|set: HashSet<String>| iter.fold(set, |set1, set2| &set1 & &set2))
                .unwrap();
            (trait_bounds, intersection_set)
        })
        .collect()
}
